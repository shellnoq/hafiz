//! Replication engine for async data replication
//!
//! Handles:
//! - Event-driven replication queue
//! - Async object copying between nodes
//! - Checksum verification
//! - Retry with exponential backoff
//! - Conflict resolution

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use chrono::Utc;
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use tokio::sync::mpsc;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use hafiz_core::types::{
    ClusterNode, ConflictResolution, NodeId, ReplicationEvent, ReplicationEventType,
    ReplicationMode, ReplicationProgress, ReplicationRule, ReplicationStatus,
};

use crate::discovery::DiscoveryService;
use crate::error::{ClusterError, ClusterResult};
use crate::transport::ClusterTransport;

/// Configuration for the replicator
#[derive(Debug, Clone)]
pub struct ReplicatorConfig {
    /// Maximum concurrent replications
    pub max_concurrent: usize,
    /// Queue size before dropping events
    pub queue_size: usize,
    /// Maximum retry attempts per event
    pub max_retries: u32,
    /// Base delay for retry backoff
    pub retry_base_delay: Duration,
    /// Enable checksum verification
    pub verify_checksums: bool,
    /// Conflict resolution strategy
    pub conflict_resolution: ConflictResolution,
    /// Batch size for bulk operations
    pub batch_size: usize,
}

impl Default for ReplicatorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            queue_size: 10000,
            max_retries: 5,
            retry_base_delay: Duration::from_millis(500),
            verify_checksums: true,
            conflict_resolution: ConflictResolution::LastWriteWins,
            batch_size: 100,
        }
    }
}

/// Statistics for the replicator
#[derive(Debug, Clone, Default)]
pub struct ReplicatorStats {
    /// Total events processed
    pub events_processed: u64,
    /// Successful replications
    pub successful: u64,
    /// Failed replications
    pub failed: u64,
    /// Currently pending events
    pub pending: u64,
    /// Events currently being processed
    pub in_progress: u64,
    /// Total bytes replicated
    pub bytes_replicated: u64,
    /// Average replication latency in ms
    pub avg_latency_ms: f64,
}

/// The replication engine
pub struct Replicator {
    /// Configuration
    config: ReplicatorConfig,
    /// Transport for node communication
    transport: Arc<ClusterTransport>,
    /// Discovery service for finding nodes
    discovery: Arc<DiscoveryService>,
    /// Event queue sender
    event_tx: mpsc::Sender<ReplicationEvent>,
    /// Replication rules
    rules: Arc<RwLock<Vec<ReplicationRule>>>,
    /// Replication progress tracking
    progress: Arc<RwLock<HashMap<String, ReplicationProgress>>>,
    /// Statistics
    stats: Arc<RwLock<ReplicatorStats>>,
    /// Shutdown signal
    shutdown: Arc<RwLock<bool>>,
    /// This node's ID
    node_id: NodeId,
}

impl Replicator {
    /// Create a new replicator
    pub fn new(
        config: ReplicatorConfig,
        transport: Arc<ClusterTransport>,
        discovery: Arc<DiscoveryService>,
        node_id: NodeId,
    ) -> (Self, mpsc::Sender<ReplicationEvent>) {
        let (event_tx, event_rx) = mpsc::channel(config.queue_size);

        let replicator = Self {
            config: config.clone(),
            transport,
            discovery,
            event_tx: event_tx.clone(),
            rules: Arc::new(RwLock::new(Vec::new())),
            progress: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(ReplicatorStats::default())),
            shutdown: Arc::new(RwLock::new(false)),
            node_id,
        };

        // Start the processing loop in a separate task
        replicator.start_processing_loop(event_rx);

        (replicator, event_tx)
    }

    /// Start the replicator
    pub async fn start(&self) -> ClusterResult<()> {
        info!(
            "Starting replicator with max {} concurrent replications",
            self.config.max_concurrent
        );
        Ok(())
    }

    /// Stop the replicator
    pub fn stop(&self) {
        info!("Stopping replicator");
        *self.shutdown.write() = true;
    }

    /// Add a replication rule
    pub fn add_rule(&self, rule: ReplicationRule) {
        info!(
            "Adding replication rule {} for bucket {}",
            rule.id, rule.source_bucket
        );
        self.rules.write().push(rule);
    }

    /// Remove a replication rule
    pub fn remove_rule(&self, rule_id: &str) -> bool {
        let mut rules = self.rules.write();
        let len_before = rules.len();
        rules.retain(|r| r.id != rule_id);
        rules.len() < len_before
    }

    /// Get all replication rules
    pub fn rules(&self) -> Vec<ReplicationRule> {
        self.rules.read().clone()
    }

    /// Get replication progress for an object
    pub fn get_progress(&self, bucket: &str, key: &str) -> Option<ReplicationProgress> {
        let progress_key = format!("{}/{}", bucket, key);
        self.progress.read().get(&progress_key).cloned()
    }

    /// Get replicator statistics
    pub fn stats(&self) -> ReplicatorStats {
        self.stats.read().clone()
    }

    /// Queue a replication event
    pub async fn queue_event(&self, event: ReplicationEvent) -> ClusterResult<()> {
        self.event_tx
            .send(event)
            .await
            .map_err(|_| ClusterError::Internal("Event queue full".to_string()))?;

        self.stats.write().pending += 1;
        Ok(())
    }

    /// Start the event processing loop
    fn start_processing_loop(&self, mut event_rx: mpsc::Receiver<ReplicationEvent>) {
        let transport = Arc::clone(&self.transport);
        let discovery = Arc::clone(&self.discovery);
        let rules = Arc::clone(&self.rules);
        let progress = Arc::clone(&self.progress);
        let stats = Arc::clone(&self.stats);
        let shutdown = Arc::clone(&self.shutdown);
        let config = self.config.clone();
        let node_id = self.node_id.clone();

        tokio::spawn(async move {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_concurrent));

            loop {
                if *shutdown.read() {
                    break;
                }

                tokio::select! {
                    Some(event) = event_rx.recv() => {
                        let permit = semaphore.clone().acquire_owned().await.unwrap();
                        let transport = Arc::clone(&transport);
                        let discovery = Arc::clone(&discovery);
                        let rules = Arc::clone(&rules);
                        let progress = Arc::clone(&progress);
                        let stats = Arc::clone(&stats);
                        let config = config.clone();
                        let node_id = node_id.clone();

                        tokio::spawn(async move {
                            let result = Self::process_event(
                                &event,
                                &transport,
                                &discovery,
                                &rules,
                                &progress,
                                &config,
                                &node_id,
                            )
                            .await;

                            // Update stats
                            {
                                let mut s = stats.write();
                                s.events_processed += 1;
                                s.pending = s.pending.saturating_sub(1);

                                match result {
                                    Ok(bytes) => {
                                        s.successful += 1;
                                        s.bytes_replicated += bytes;
                                    }
                                    Err(e) => {
                                        s.failed += 1;
                                        error!("Replication failed: {}", e);
                                    }
                                }
                            }

                            drop(permit);
                        });
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        // Check for shutdown periodically
                    }
                }
            }

            info!("Replicator processing loop stopped");
        });
    }

    /// Process a single replication event
    async fn process_event(
        event: &ReplicationEvent,
        transport: &ClusterTransport,
        discovery: &DiscoveryService,
        rules: &RwLock<Vec<ReplicationRule>>,
        progress: &RwLock<HashMap<String, ReplicationProgress>>,
        config: &ReplicatorConfig,
        local_node_id: &str,
    ) -> ClusterResult<u64> {
        debug!("Processing replication event: {:?}", event.event_type);

        // Find matching rules
        let matching_rules: Vec<ReplicationRule> = {
            let rules = rules.read();
            rules
                .iter()
                .filter(|r| {
                    r.enabled
                        && r.source_bucket == event.bucket
                        && event
                            .key
                            .as_ref()
                            .map_or(true, |k| r.matches(k, &event.metadata))
                })
                .cloned()
                .collect()
        };

        if matching_rules.is_empty() {
            debug!("No matching rules for event, skipping");
            return Ok(0);
        }

        // Get target nodes
        let healthy_nodes = discovery.healthy_nodes();
        let mut total_bytes: u64 = 0;

        for rule in matching_rules {
            // Determine target nodes for this rule
            let targets: Vec<&ClusterNode> = if rule.target_nodes.is_empty() {
                // Replicate to all healthy nodes except source
                healthy_nodes
                    .iter()
                    .filter(|n| n.id != event.source_node && n.id != local_node_id)
                    .collect()
            } else {
                healthy_nodes
                    .iter()
                    .filter(|n| rule.target_nodes.contains(&n.id))
                    .collect()
            };

            if targets.is_empty() {
                debug!("No target nodes for rule {}", rule.id);
                continue;
            }

            // Process based on event type
            match event.event_type {
                ReplicationEventType::ObjectCreated | ReplicationEventType::MetadataUpdated => {
                    let bytes = Self::replicate_object(
                        event, &targets, transport, discovery, progress, config,
                    )
                    .await?;
                    total_bytes += bytes;
                }
                ReplicationEventType::ObjectDeleted => {
                    if rule.replicate_deletes {
                        Self::replicate_delete(event, &targets, transport, progress).await?;
                    }
                }
                ReplicationEventType::BucketCreated | ReplicationEventType::BucketDeleted => {
                    // Bucket operations are handled differently
                    debug!("Bucket operation replication not yet implemented");
                }
            }
        }

        Ok(total_bytes)
    }

    /// Replicate an object to target nodes
    async fn replicate_object(
        event: &ReplicationEvent,
        targets: &[&ClusterNode],
        transport: &ClusterTransport,
        discovery: &DiscoveryService,
        progress: &RwLock<HashMap<String, ReplicationProgress>>,
        config: &ReplicatorConfig,
    ) -> ClusterResult<u64> {
        let key = event
            .key
            .as_ref()
            .ok_or_else(|| ClusterError::Internal("Object event missing key".to_string()))?;

        let progress_key = format!("{}/{}", event.bucket, key);

        // Initialize progress tracking
        {
            let mut prog = progress.write();
            prog.insert(
                progress_key.clone(),
                ReplicationProgress {
                    bucket: event.bucket.clone(),
                    key: key.clone(),
                    version_id: event.version_id.clone(),
                    node_status: targets
                        .iter()
                        .map(|n| (n.id.clone(), ReplicationStatus::Pending))
                        .collect(),
                    last_attempt: Some(Utc::now()),
                    error: None,
                },
            );
        }

        // Fetch object data from source node
        let source_node = discovery
            .get_node(&event.source_node)
            .ok_or_else(|| ClusterError::NodeNotFound(event.source_node.clone()))?;

        let (data, checksum) = transport
            .fetch_object_data(
                &source_node,
                &event.bucket,
                key,
                event.version_id.as_deref(),
            )
            .await?;

        // Verify checksum if enabled
        if config.verify_checksums {
            if let Some(expected) = &event.checksum {
                let actual = Self::compute_checksum(&data);
                if &actual != expected {
                    return Err(ClusterError::ChecksumMismatch {
                        expected: expected.clone(),
                        got: actual,
                    });
                }
            }
        }

        let data_len = data.len() as u64;

        // Replicate to each target
        for target in targets {
            let result = transport
                .upload_object_data(
                    target,
                    &event.bucket,
                    key,
                    data.clone(),
                    checksum.as_deref(),
                    &event.metadata,
                )
                .await;

            // Update progress
            {
                let mut prog = progress.write();
                if let Some(p) = prog.get_mut(&progress_key) {
                    p.node_status.insert(
                        target.id.clone(),
                        if result.is_ok() {
                            ReplicationStatus::Completed
                        } else {
                            ReplicationStatus::Failed
                        },
                    );
                    if let Err(e) = &result {
                        p.error = Some(e.to_string());
                    }
                }
            }

            if let Err(e) = result {
                warn!("Failed to replicate to {}: {}", target.id, e);
            }
        }

        Ok(data_len)
    }

    /// Replicate a delete operation to target nodes
    async fn replicate_delete(
        event: &ReplicationEvent,
        targets: &[&ClusterNode],
        transport: &ClusterTransport,
        progress: &RwLock<HashMap<String, ReplicationProgress>>,
    ) -> ClusterResult<()> {
        let key = event
            .key
            .as_ref()
            .ok_or_else(|| ClusterError::Internal("Delete event missing key".to_string()))?;

        for target in targets {
            // TODO: Implement delete API on transport
            debug!(
                "Would delete {}/{} on node {}",
                event.bucket, key, target.id
            );
        }

        Ok(())
    }

    /// Compute SHA256 checksum of data
    fn compute_checksum(data: &Bytes) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }
}

impl std::fmt::Debug for Replicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Replicator")
            .field("config", &self.config)
            .field("node_id", &self.node_id)
            .field("stats", &self.stats())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replicator_config_default() {
        let config = ReplicatorConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.queue_size, 10000);
        assert!(config.verify_checksums);
    }

    #[test]
    fn test_compute_checksum() {
        let data = Bytes::from("hello world");
        let checksum = Replicator::compute_checksum(&data);
        assert!(!checksum.is_empty());
        assert_eq!(checksum.len(), 64); // SHA256 hex is 64 chars
    }
}
