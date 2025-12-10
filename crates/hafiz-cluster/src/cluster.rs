//! Cluster manager - the main coordinator for cluster operations
//!
//! Responsibilities:
//! - Initialize and coordinate all cluster components
//! - Handle cluster API requests
//! - Manage cluster state
//! - Coordinate failover

use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use hafiz_core::types::{
    ClusterConfig, ClusterMessage, ClusterNode, ClusterStats, NodeId, NodeStats, NodeStatus,
    ReplicationEvent, ReplicationRule,
};

use crate::discovery::{DiscoveryEvent, DiscoveryService};
use crate::error::{ClusterError, ClusterResult};
use crate::replicator::{Replicator, ReplicatorConfig, ReplicatorStats};
use crate::transport::{ClusterTransport, TransportConfig};

/// The main cluster manager
pub struct ClusterManager {
    /// Cluster configuration
    config: ClusterConfig,
    /// Discovery service
    discovery: Arc<DiscoveryService>,
    /// Replicator
    replicator: Arc<Replicator>,
    /// Event sender for replication
    replication_tx: mpsc::Sender<ReplicationEvent>,
    /// Transport layer
    transport: Arc<ClusterTransport>,
    /// Whether cluster mode is enabled
    enabled: bool,
}

impl ClusterManager {
    /// Create a new cluster manager
    pub fn new(config: ClusterConfig) -> ClusterResult<Self> {
        // Check if cluster mode should be enabled
        let enabled =
            !config.seed_nodes.is_empty() || config.advertise_endpoint != "http://localhost:9000";

        if !enabled {
            info!("Cluster mode disabled (no seed nodes configured)");
        }

        // Create transport
        let transport_config = TransportConfig {
            verify_tls: config.cluster_tls_enabled,
            ca_cert_path: config.cluster_ca_cert.clone(),
            client_cert_path: config.cluster_tls_cert.clone(),
            client_key_path: config.cluster_tls_key.clone(),
            ..Default::default()
        };
        let transport = Arc::new(ClusterTransport::new(transport_config)?);

        // Create discovery service event channel
        let (discovery_tx, discovery_rx) = mpsc::channel(1000);

        // Create discovery service
        let discovery = Arc::new(DiscoveryService::new(
            config.clone(),
            Arc::clone(&transport),
            discovery_tx,
        ));

        // Create replicator
        let replicator_config = ReplicatorConfig::default();
        let (replicator, replication_tx) = Replicator::new(
            replicator_config,
            Arc::clone(&transport),
            Arc::clone(&discovery),
            config.node_id.clone(),
        );
        let replicator = Arc::new(replicator);

        // Start listening for discovery events
        Self::handle_discovery_events(discovery_rx, Arc::clone(&replicator));

        Ok(Self {
            config,
            discovery,
            replicator,
            replication_tx,
            transport,
            enabled,
        })
    }

    /// Start the cluster manager
    pub async fn start(&self) -> ClusterResult<()> {
        if !self.enabled {
            info!("Cluster manager running in standalone mode");
            return Ok(());
        }

        info!("Starting cluster manager for '{}'", self.config.name);

        // Start discovery
        self.discovery.start().await?;

        // Start replicator
        self.replicator.start().await?;

        info!("Cluster manager started successfully");
        Ok(())
    }

    /// Stop the cluster manager
    pub async fn stop(&self) -> ClusterResult<()> {
        if !self.enabled {
            return Ok(());
        }

        info!("Stopping cluster manager");

        // Send leave notification to other nodes
        let local_node = self.discovery.local_node();
        let leave_msg = ClusterMessage::LeaveNotification {
            node_id: local_node.id.clone(),
            reason: "Node shutting down".to_string(),
        };

        for node in self.discovery.healthy_nodes() {
            if node.id != local_node.id {
                let _ = self.transport.send_message(&node, &leave_msg).await;
            }
        }

        // Stop components
        self.replicator.stop();
        self.discovery.stop();

        info!("Cluster manager stopped");
        Ok(())
    }

    /// Check if cluster mode is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the local node
    pub fn local_node(&self) -> ClusterNode {
        self.discovery.local_node()
    }

    /// Get all nodes in the cluster
    pub fn nodes(&self) -> Vec<ClusterNode> {
        self.discovery.nodes()
    }

    /// Get healthy nodes
    pub fn healthy_nodes(&self) -> Vec<ClusterNode> {
        self.discovery.healthy_nodes()
    }

    /// Get a specific node
    pub fn get_node(&self, node_id: &str) -> Option<ClusterNode> {
        self.discovery.get_node(node_id)
    }

    /// Get cluster statistics
    pub fn stats(&self) -> ClusterStats {
        let nodes = self.discovery.nodes();
        let replicator_stats = self.replicator.stats();

        ClusterStats {
            total_nodes: nodes.len() as u32 + 1, // Include local node
            healthy_nodes: nodes.iter().filter(|n| n.is_healthy()).count() as u32 + 1,
            primary_nodes: nodes.iter().filter(|n| n.can_accept_writes()).count() as u32 + 1,
            replica_nodes: nodes
                .iter()
                .filter(|n| matches!(n.role, hafiz_core::types::NodeRole::Replica))
                .count() as u32,
            total_objects: 0,       // TODO: Get from metadata
            total_storage_bytes: 0, // TODO: Get from storage
            pending_replications: replicator_stats.pending,
            failed_replications: replicator_stats.failed,
            replication_lag_secs: 0, // TODO: Calculate
        }
    }

    /// Get replicator statistics
    pub fn replicator_stats(&self) -> ReplicatorStats {
        self.replicator.stats()
    }

    /// Add a replication rule
    pub fn add_replication_rule(&self, rule: ReplicationRule) {
        self.replicator.add_rule(rule);
    }

    /// Remove a replication rule
    pub fn remove_replication_rule(&self, rule_id: &str) -> bool {
        self.replicator.remove_rule(rule_id)
    }

    /// Get all replication rules
    pub fn replication_rules(&self) -> Vec<ReplicationRule> {
        self.replicator.rules()
    }

    /// Queue a replication event
    pub async fn queue_replication(&self, event: ReplicationEvent) -> ClusterResult<()> {
        if !self.enabled {
            return Ok(()); // Silently ignore in standalone mode
        }
        self.replicator.queue_event(event).await
    }

    /// Get the replication event sender (for direct access)
    pub fn replication_sender(&self) -> mpsc::Sender<ReplicationEvent> {
        self.replication_tx.clone()
    }

    /// Handle an incoming cluster message
    pub async fn handle_message(&self, message: ClusterMessage) -> ClusterResult<ClusterMessage> {
        match message {
            ClusterMessage::JoinRequest { node, cluster_name } => {
                self.discovery
                    .handle_join_request(node, &cluster_name)
                    .await
            }
            ClusterMessage::Heartbeat { node, stats } => {
                self.discovery.handle_heartbeat(node, stats).await?;
                Ok(ClusterMessage::Heartbeat {
                    node: self.discovery.local_node(),
                    stats: NodeStats::default(),
                })
            }
            ClusterMessage::LeaveNotification { node_id, reason } => {
                self.discovery.handle_leave(&node_id, &reason).await?;
                Ok(ClusterMessage::LeaveNotification {
                    node_id: self.config.node_id.clone(),
                    reason: "Acknowledged".to_string(),
                })
            }
            ClusterMessage::ReplicationEvent(event) => {
                self.queue_replication(event).await?;
                Ok(ClusterMessage::Heartbeat {
                    node: self.discovery.local_node(),
                    stats: NodeStats::default(),
                })
            }
            ClusterMessage::StateSync {
                nodes,
                replication_rules,
            } => {
                // Apply state sync
                for rule in replication_rules {
                    self.add_replication_rule(rule);
                }
                Ok(ClusterMessage::StateSync {
                    nodes: self.nodes(),
                    replication_rules: self.replication_rules(),
                })
            }
            _ => Err(ClusterError::Internal("Unhandled message type".to_string())),
        }
    }

    /// Handle discovery events
    fn handle_discovery_events(
        mut rx: mpsc::Receiver<DiscoveryEvent>,
        replicator: Arc<Replicator>,
    ) {
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    DiscoveryEvent::NodeJoined(node) => {
                        info!("Node joined cluster: {} ({})", node.name, node.id);
                    }
                    DiscoveryEvent::NodeLeft(node_id) => {
                        warn!("Node left cluster: {}", node_id);
                    }
                    DiscoveryEvent::NodeUnhealthy(node_id) => {
                        warn!("Node became unhealthy: {}", node_id);
                    }
                    DiscoveryEvent::NodeRecovered(node_id) => {
                        info!("Node recovered: {}", node_id);
                    }
                    DiscoveryEvent::StateSynced => {
                        info!("Cluster state synchronized");
                    }
                }
            }
        });
    }
}

impl std::fmt::Debug for ClusterManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClusterManager")
            .field("config", &self.config.name)
            .field("enabled", &self.enabled)
            .field("node_id", &self.config.node_id)
            .finish()
    }
}

/// Builder for ClusterManager
pub struct ClusterManagerBuilder {
    config: ClusterConfig,
}

impl ClusterManagerBuilder {
    /// Create a new builder with default config
    pub fn new() -> Self {
        Self {
            config: ClusterConfig::default(),
        }
    }

    /// Set the cluster name
    pub fn cluster_name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set the node ID
    pub fn node_id(mut self, id: impl Into<String>) -> Self {
        self.config.node_id = id.into();
        self
    }

    /// Set the node name
    pub fn node_name(mut self, name: impl Into<String>) -> Self {
        self.config.node_name = name.into();
        self
    }

    /// Set the advertise endpoint
    pub fn advertise_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.advertise_endpoint = endpoint.into();
        self
    }

    /// Set the cluster endpoint
    pub fn cluster_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.config.cluster_endpoint = endpoint.into();
        self
    }

    /// Add seed nodes
    pub fn seed_nodes(mut self, nodes: Vec<String>) -> Self {
        self.config.seed_nodes = nodes;
        self
    }

    /// Enable cluster TLS
    pub fn enable_tls(mut self, cert: String, key: String, ca: Option<String>) -> Self {
        self.config.cluster_tls_enabled = true;
        self.config.cluster_tls_cert = Some(cert);
        self.config.cluster_tls_key = Some(key);
        self.config.cluster_ca_cert = ca;
        self
    }

    /// Build the cluster manager
    pub fn build(self) -> ClusterResult<ClusterManager> {
        ClusterManager::new(self.config)
    }
}

impl Default for ClusterManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let manager = ClusterManagerBuilder::new()
            .cluster_name("test-cluster")
            .node_id("node-1")
            .node_name("Test Node 1")
            .seed_nodes(vec!["http://seed1:9001".to_string()])
            .build();

        assert!(manager.is_ok());
    }
}
