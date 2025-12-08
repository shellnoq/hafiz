//! Node discovery and membership management
//!
//! Handles:
//! - Initial cluster join via seed nodes
//! - Heartbeat-based health monitoring
//! - Automatic node failure detection
//! - Cluster state synchronization

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tokio::time::{interval, Instant};
use tracing::{debug, error, info, warn};

use hafiz_core::types::{
    ClusterConfig, ClusterMessage, ClusterNode, NodeId, NodeRole, NodeStats, NodeStatus,
};

use crate::error::{ClusterError, ClusterResult};
use crate::transport::ClusterTransport;

/// Discovery service for cluster membership
pub struct DiscoveryService {
    /// Current node information
    local_node: Arc<RwLock<ClusterNode>>,
    /// All known nodes in the cluster
    nodes: Arc<RwLock<HashMap<NodeId, ClusterNode>>>,
    /// Cluster configuration
    config: ClusterConfig,
    /// Transport for communication
    transport: Arc<ClusterTransport>,
    /// Channel to notify about node changes
    event_tx: mpsc::Sender<DiscoveryEvent>,
    /// Shutdown signal
    shutdown: Arc<RwLock<bool>>,
}

/// Events emitted by the discovery service
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    /// A new node joined the cluster
    NodeJoined(ClusterNode),
    /// A node left the cluster
    NodeLeft(NodeId),
    /// A node became unhealthy
    NodeUnhealthy(NodeId),
    /// A node recovered
    NodeRecovered(NodeId),
    /// Cluster state was synchronized
    StateSynced,
}

impl DiscoveryService {
    /// Create a new discovery service
    pub fn new(
        config: ClusterConfig,
        transport: Arc<ClusterTransport>,
        event_tx: mpsc::Sender<DiscoveryEvent>,
    ) -> Self {
        let local_node = ClusterNode::new(
            config.node_id.clone(),
            config.node_name.clone(),
            config.advertise_endpoint.clone(),
            config.cluster_endpoint.clone(),
        );

        Self {
            local_node: Arc::new(RwLock::new(local_node)),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            config,
            transport,
            event_tx,
            shutdown: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the discovery service
    pub async fn start(&self) -> ClusterResult<()> {
        info!("Starting discovery service for cluster '{}'", self.config.name);

        // Try to join via seed nodes
        if !self.config.seed_nodes.is_empty() {
            self.join_cluster().await?;
        } else {
            info!("No seed nodes configured, starting as standalone node");
            self.local_node.write().status = NodeStatus::Healthy;
        }

        // Start heartbeat loop
        self.start_heartbeat_loop();

        // Start health check loop
        self.start_health_check_loop();

        Ok(())
    }

    /// Stop the discovery service
    pub fn stop(&self) {
        info!("Stopping discovery service");
        *self.shutdown.write() = true;
    }

    /// Get the local node
    pub fn local_node(&self) -> ClusterNode {
        self.local_node.read().clone()
    }

    /// Get all known nodes
    pub fn nodes(&self) -> Vec<ClusterNode> {
        self.nodes.read().values().cloned().collect()
    }

    /// Get healthy nodes
    pub fn healthy_nodes(&self) -> Vec<ClusterNode> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.is_healthy())
            .cloned()
            .collect()
    }

    /// Get a specific node
    pub fn get_node(&self, node_id: &str) -> Option<ClusterNode> {
        self.nodes.read().get(node_id).cloned()
    }

    /// Get nodes that can accept writes
    pub fn writable_nodes(&self) -> Vec<ClusterNode> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.can_accept_writes())
            .cloned()
            .collect()
    }

    /// Get nodes that can accept reads
    pub fn readable_nodes(&self) -> Vec<ClusterNode> {
        self.nodes
            .read()
            .values()
            .filter(|n| n.can_accept_reads())
            .cloned()
            .collect()
    }

    /// Join the cluster via seed nodes
    async fn join_cluster(&self) -> ClusterResult<()> {
        info!("Attempting to join cluster via seed nodes");

        let local = self.local_node.read().clone();
        let join_message = ClusterMessage::JoinRequest {
            node: local.clone(),
            cluster_name: self.config.name.clone(),
        };

        for seed in &self.config.seed_nodes {
            debug!("Trying seed node: {}", seed);

            match self.transport.send_join_request(seed, &join_message).await {
                Ok(ClusterMessage::JoinResponse {
                    accepted,
                    cluster_name,
                    nodes,
                    message,
                }) => {
                    if !accepted {
                        warn!("Join rejected by {}: {:?}", seed, message);
                        continue;
                    }

                    if cluster_name != self.config.name {
                        warn!(
                            "Cluster name mismatch: expected {}, got {}",
                            self.config.name, cluster_name
                        );
                        continue;
                    }

                    info!(
                        "Successfully joined cluster '{}' via {}",
                        cluster_name, seed
                    );

                    // Add all known nodes
                    {
                        let mut nodes_map = self.nodes.write();
                        for node in nodes {
                            if node.id != self.config.node_id {
                                nodes_map.insert(node.id.clone(), node);
                            }
                        }
                    }

                    // Update local node status
                    self.local_node.write().status = NodeStatus::Healthy;

                    // Notify about state sync
                    let _ = self.event_tx.send(DiscoveryEvent::StateSynced).await;

                    return Ok(());
                }
                Ok(_) => {
                    warn!("Unexpected response from seed node {}", seed);
                }
                Err(e) => {
                    warn!("Failed to connect to seed node {}: {}", seed, e);
                }
            }
        }

        // If we couldn't join, start as a new cluster
        warn!("Could not join existing cluster, starting as new primary");
        self.local_node.write().status = NodeStatus::Healthy;
        Ok(())
    }

    /// Handle a join request from another node
    pub async fn handle_join_request(
        &self,
        node: ClusterNode,
        cluster_name: &str,
    ) -> ClusterResult<ClusterMessage> {
        // Verify cluster name
        if cluster_name != self.config.name {
            return Ok(ClusterMessage::JoinResponse {
                accepted: false,
                cluster_name: self.config.name.clone(),
                nodes: vec![],
                message: Some(format!(
                    "Cluster name mismatch: expected {}, got {}",
                    self.config.name, cluster_name
                )),
            });
        }

        // Check if node already exists
        if self.nodes.read().contains_key(&node.id) {
            info!("Node {} rejoining cluster", node.id);
        } else {
            info!("New node {} joining cluster", node.id);
        }

        // Add/update the node
        {
            let mut nodes = self.nodes.write();
            nodes.insert(node.id.clone(), node.clone());
        }

        // Notify about new node
        let _ = self.event_tx.send(DiscoveryEvent::NodeJoined(node)).await;

        // Return current cluster state
        let all_nodes: Vec<ClusterNode> = {
            let nodes = self.nodes.read();
            let mut all = nodes.values().cloned().collect::<Vec<_>>();
            all.push(self.local_node.read().clone());
            all
        };

        Ok(ClusterMessage::JoinResponse {
            accepted: true,
            cluster_name: self.config.name.clone(),
            nodes: all_nodes,
            message: None,
        })
    }

    /// Handle a heartbeat from another node
    pub async fn handle_heartbeat(
        &self,
        node: ClusterNode,
        stats: NodeStats,
    ) -> ClusterResult<()> {
        let mut nodes = self.nodes.write();

        if let Some(existing) = nodes.get_mut(&node.id) {
            // Update existing node
            existing.status = node.status;
            existing.last_heartbeat = Utc::now();

            // Check if node recovered
            if existing.status == NodeStatus::Healthy
                && matches!(
                    existing.status,
                    NodeStatus::Unreachable | NodeStatus::Degraded
                )
            {
                let _ = self
                    .event_tx
                    .send(DiscoveryEvent::NodeRecovered(node.id.clone()))
                    .await;
            }
        } else {
            // New node - add it
            let mut new_node = node.clone();
            new_node.last_heartbeat = Utc::now();
            nodes.insert(node.id.clone(), new_node);

            let _ = self.event_tx.send(DiscoveryEvent::NodeJoined(node)).await;
        }

        Ok(())
    }

    /// Handle a leave notification
    pub async fn handle_leave(&self, node_id: &str, reason: &str) -> ClusterResult<()> {
        info!("Node {} leaving cluster: {}", node_id, reason);

        if let Some(mut node) = self.nodes.write().remove(node_id) {
            node.status = NodeStatus::Left;
            let _ = self
                .event_tx
                .send(DiscoveryEvent::NodeLeft(node_id.to_string()))
                .await;
        }

        Ok(())
    }

    /// Start the heartbeat loop
    fn start_heartbeat_loop(&self) {
        let local_node = Arc::clone(&self.local_node);
        let nodes = Arc::clone(&self.nodes);
        let transport = Arc::clone(&self.transport);
        let shutdown = Arc::clone(&self.shutdown);
        let interval_secs = self.config.heartbeat_interval_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(interval_secs));

            loop {
                ticker.tick().await;

                if *shutdown.read() {
                    break;
                }

                let local = local_node.read().clone();
                let heartbeat = ClusterMessage::Heartbeat {
                    node: local.clone(),
                    stats: NodeStats::default(), // TODO: Collect real stats
                };

                // Send heartbeat to all known nodes
                let target_nodes: Vec<ClusterNode> = nodes.read().values().cloned().collect();

                for node in target_nodes {
                    if node.id == local.id {
                        continue;
                    }

                    let transport = Arc::clone(&transport);
                    let heartbeat = heartbeat.clone();

                    tokio::spawn(async move {
                        if let Err(e) = transport.send_heartbeat(&node, &heartbeat).await {
                            debug!("Failed to send heartbeat to {}: {}", node.id, e);
                        }
                    });
                }
            }

            debug!("Heartbeat loop stopped");
        });
    }

    /// Start the health check loop
    fn start_health_check_loop(&self) {
        let nodes = Arc::clone(&self.nodes);
        let transport = Arc::clone(&self.transport);
        let shutdown = Arc::clone(&self.shutdown);
        let event_tx = self.event_tx.clone();
        let timeout_secs = self.config.node_timeout_secs;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(10));

            loop {
                ticker.tick().await;

                if *shutdown.read() {
                    break;
                }

                let now = Utc::now();
                let timeout = chrono::Duration::seconds(timeout_secs as i64);

                let mut unhealthy_nodes = Vec::new();

                // Check each node
                {
                    let mut nodes_write = nodes.write();
                    for (id, node) in nodes_write.iter_mut() {
                        let since_heartbeat = now - node.last_heartbeat;

                        if since_heartbeat > timeout && node.status == NodeStatus::Healthy {
                            warn!(
                                "Node {} hasn't sent heartbeat in {:?}, marking unhealthy",
                                id, since_heartbeat
                            );
                            node.status = NodeStatus::Unreachable;
                            unhealthy_nodes.push(id.clone());
                        }
                    }
                }

                // Notify about unhealthy nodes
                for node_id in unhealthy_nodes {
                    let _ = event_tx.send(DiscoveryEvent::NodeUnhealthy(node_id)).await;
                }
            }

            debug!("Health check loop stopped");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[test]
    fn test_discovery_events() {
        let node = ClusterNode::new(
            "test".to_string(),
            "Test Node".to_string(),
            "http://localhost:9000".to_string(),
            "http://localhost:9001".to_string(),
        );

        let event = DiscoveryEvent::NodeJoined(node.clone());
        match event {
            DiscoveryEvent::NodeJoined(n) => assert_eq!(n.id, "test"),
            _ => panic!("Wrong event type"),
        }
    }
}
