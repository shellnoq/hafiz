//! Replication types for multi-node Hafiz cluster
//!
//! Supports async replication with configurable consistency levels,
//! conflict resolution, and bucket-level replication rules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a cluster node
pub type NodeId = String;

/// Replication mode for bucket data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReplicationMode {
    /// No replication - single node only
    #[default]
    None,
    /// Asynchronous replication - eventual consistency
    Async,
    /// Synchronous replication - strong consistency (slower)
    Sync,
}

/// Consistency level for read operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConsistencyLevel {
    /// Read from any node (fastest, eventual consistency)
    #[default]
    One,
    /// Read from quorum of nodes
    Quorum,
    /// Read from all nodes (slowest, strong consistency)
    All,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    /// Last write wins based on timestamp
    #[default]
    LastWriteWins,
    /// First write wins (immutable after creation)
    FirstWriteWins,
    /// Highest version number wins
    HighestVersion,
    /// Custom resolver (requires plugin)
    Custom,
}

/// Node status in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    /// Node is starting up
    #[default]
    Starting,
    /// Node is healthy and accepting requests
    Healthy,
    /// Node is degraded but operational
    Degraded,
    /// Node is unreachable
    Unreachable,
    /// Node is being drained for maintenance
    Draining,
    /// Node has left the cluster
    Left,
}

/// Node role in the cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeRole {
    /// Can accept writes and reads
    #[default]
    Primary,
    /// Read-only replica
    Replica,
    /// Witness node for quorum (no data)
    Witness,
}

/// Information about a cluster node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    /// Unique node identifier
    pub id: NodeId,
    /// Human-readable node name
    pub name: String,
    /// Node's API endpoint (e.g., "https://node1.hafiz.local:9000")
    pub endpoint: String,
    /// Internal cluster communication endpoint
    pub cluster_endpoint: String,
    /// Node role
    pub role: NodeRole,
    /// Current status
    pub status: NodeStatus,
    /// Node region/zone for locality-aware routing
    pub region: Option<String>,
    /// Node zone within region
    pub zone: Option<String>,
    /// Weight for load balancing (higher = more traffic)
    pub weight: u32,
    /// Node metadata/labels
    pub labels: HashMap<String, String>,
    /// When the node joined the cluster
    pub joined_at: DateTime<Utc>,
    /// Last heartbeat received
    pub last_heartbeat: DateTime<Utc>,
    /// Node version
    pub version: String,
}

impl ClusterNode {
    pub fn new(id: NodeId, name: String, endpoint: String, cluster_endpoint: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            endpoint,
            cluster_endpoint,
            role: NodeRole::Primary,
            status: NodeStatus::Starting,
            region: None,
            zone: None,
            weight: 100,
            labels: HashMap::new(),
            joined_at: now,
            last_heartbeat: now,
            version: crate::VERSION.to_string(),
        }
    }

    pub fn is_healthy(&self) -> bool {
        matches!(self.status, NodeStatus::Healthy | NodeStatus::Degraded)
    }

    pub fn can_accept_writes(&self) -> bool {
        self.is_healthy() && matches!(self.role, NodeRole::Primary)
    }

    pub fn can_accept_reads(&self) -> bool {
        self.is_healthy() && !matches!(self.role, NodeRole::Witness)
    }
}

/// Cluster configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterConfig {
    /// Cluster name (must match across all nodes)
    pub name: String,
    /// This node's ID
    pub node_id: NodeId,
    /// This node's name
    pub node_name: String,
    /// This node's API endpoint
    pub advertise_endpoint: String,
    /// This node's cluster communication endpoint
    pub cluster_endpoint: String,
    /// Seed nodes for discovery
    pub seed_nodes: Vec<String>,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Node timeout in seconds (consider dead after this)
    pub node_timeout_secs: u64,
    /// Default replication mode for new buckets
    pub default_replication_mode: ReplicationMode,
    /// Default replication factor
    pub default_replication_factor: u32,
    /// Default consistency level for reads
    pub default_consistency_level: ConsistencyLevel,
    /// Enable TLS for cluster communication
    pub cluster_tls_enabled: bool,
    /// Path to cluster TLS certificate
    pub cluster_tls_cert: Option<String>,
    /// Path to cluster TLS key
    pub cluster_tls_key: Option<String>,
    /// Path to cluster CA certificate
    pub cluster_ca_cert: Option<String>,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            name: "hafiz-cluster".to_string(),
            node_id: Uuid::new_v4().to_string(),
            node_name: hostname::get()
                .map(|h| h.to_string_lossy().to_string())
                .unwrap_or_else(|_| "hafiz-node".to_string()),
            advertise_endpoint: "http://localhost:9000".to_string(),
            cluster_endpoint: "http://localhost:9001".to_string(),
            seed_nodes: Vec::new(),
            heartbeat_interval_secs: 5,
            node_timeout_secs: 30,
            default_replication_mode: ReplicationMode::Async,
            default_replication_factor: 2,
            default_consistency_level: ConsistencyLevel::One,
            cluster_tls_enabled: false,
            cluster_tls_cert: None,
            cluster_tls_key: None,
            cluster_ca_cert: None,
        }
    }
}

/// Replication rule for a bucket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationRule {
    /// Rule ID
    pub id: String,
    /// Rule is enabled
    pub enabled: bool,
    /// Source bucket
    pub source_bucket: String,
    /// Destination bucket (can be same name on different node)
    pub destination_bucket: String,
    /// Target node IDs (empty = all nodes)
    pub target_nodes: Vec<NodeId>,
    /// Object prefix filter
    pub prefix_filter: Option<String>,
    /// Object tag filters
    pub tag_filters: HashMap<String, String>,
    /// Replication mode for this rule
    pub mode: ReplicationMode,
    /// Priority (lower = higher priority)
    pub priority: i32,
    /// Replicate delete markers
    pub replicate_deletes: bool,
    /// Replicate existing objects (not just new ones)
    pub replicate_existing: bool,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub updated_at: DateTime<Utc>,
}

impl ReplicationRule {
    pub fn new(source_bucket: String, destination_bucket: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            enabled: true,
            source_bucket,
            destination_bucket,
            target_nodes: Vec::new(),
            prefix_filter: None,
            tag_filters: HashMap::new(),
            mode: ReplicationMode::Async,
            priority: 0,
            replicate_deletes: true,
            replicate_existing: false,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if an object matches this rule
    pub fn matches(&self, key: &str, tags: &HashMap<String, String>) -> bool {
        // Check prefix
        if let Some(prefix) = &self.prefix_filter {
            if !key.starts_with(prefix) {
                return false;
            }
        }

        // Check tags
        for (k, v) in &self.tag_filters {
            if tags.get(k) != Some(v) {
                return false;
            }
        }

        true
    }
}

/// Replication event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReplicationEventType {
    /// Object was created/updated
    ObjectCreated,
    /// Object was deleted
    ObjectDeleted,
    /// Object metadata was updated
    MetadataUpdated,
    /// Bucket was created
    BucketCreated,
    /// Bucket was deleted
    BucketDeleted,
}

/// A replication event to be processed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationEvent {
    /// Event ID
    pub id: String,
    /// Event type
    pub event_type: ReplicationEventType,
    /// Source node ID
    pub source_node: NodeId,
    /// Bucket name
    pub bucket: String,
    /// Object key (if applicable)
    pub key: Option<String>,
    /// Object version ID (if applicable)
    pub version_id: Option<String>,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Number of retry attempts
    pub retry_count: u32,
    /// Checksum of the object data
    pub checksum: Option<String>,
    /// Object size in bytes
    pub size: Option<u64>,
    /// Additional event data
    pub metadata: HashMap<String, String>,
}

impl ReplicationEvent {
    pub fn object_created(
        source_node: NodeId,
        bucket: String,
        key: String,
        version_id: Option<String>,
        checksum: Option<String>,
        size: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type: ReplicationEventType::ObjectCreated,
            source_node,
            bucket,
            key: Some(key),
            version_id,
            timestamp: Utc::now(),
            retry_count: 0,
            checksum,
            size: Some(size),
            metadata: HashMap::new(),
        }
    }

    pub fn object_deleted(
        source_node: NodeId,
        bucket: String,
        key: String,
        version_id: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type: ReplicationEventType::ObjectDeleted,
            source_node,
            bucket,
            key: Some(key),
            version_id,
            timestamp: Utc::now(),
            retry_count: 0,
            checksum: None,
            size: None,
            metadata: HashMap::new(),
        }
    }
}

/// Replication status for an object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReplicationStatus {
    /// Pending replication
    Pending,
    /// Currently replicating
    InProgress,
    /// Successfully replicated
    Completed,
    /// Replication failed
    Failed,
    /// Replication not applicable
    NotApplicable,
}

/// Replication progress for an object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationProgress {
    /// Bucket name
    pub bucket: String,
    /// Object key
    pub key: String,
    /// Version ID
    pub version_id: Option<String>,
    /// Replication status per target node
    pub node_status: HashMap<NodeId, ReplicationStatus>,
    /// Last attempt timestamp
    pub last_attempt: Option<DateTime<Utc>>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Cluster statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClusterStats {
    /// Total number of nodes
    pub total_nodes: u32,
    /// Number of healthy nodes
    pub healthy_nodes: u32,
    /// Number of primary nodes
    pub primary_nodes: u32,
    /// Number of replica nodes
    pub replica_nodes: u32,
    /// Total objects across cluster
    pub total_objects: u64,
    /// Total storage used (bytes)
    pub total_storage_bytes: u64,
    /// Pending replication events
    pub pending_replications: u64,
    /// Failed replication events
    pub failed_replications: u64,
    /// Replication lag in seconds (max across all nodes)
    pub replication_lag_secs: u64,
}

/// Message types for cluster communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClusterMessage {
    /// Heartbeat from a node
    Heartbeat { node: ClusterNode, stats: NodeStats },
    /// Request to join cluster
    JoinRequest {
        node: ClusterNode,
        cluster_name: String,
    },
    /// Response to join request
    JoinResponse {
        accepted: bool,
        cluster_name: String,
        nodes: Vec<ClusterNode>,
        message: Option<String>,
    },
    /// Node is leaving the cluster
    LeaveNotification { node_id: NodeId, reason: String },
    /// Replication event to be processed
    ReplicationEvent(ReplicationEvent),
    /// Request object data for replication
    ObjectDataRequest {
        request_id: String,
        bucket: String,
        key: String,
        version_id: Option<String>,
    },
    /// Response with object data
    ObjectDataResponse {
        request_id: String,
        success: bool,
        data: Option<Vec<u8>>,
        checksum: Option<String>,
        error: Option<String>,
    },
    /// Cluster state sync
    StateSync {
        nodes: Vec<ClusterNode>,
        replication_rules: Vec<ReplicationRule>,
    },
}

/// Statistics for a single node
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeStats {
    /// Number of buckets
    pub bucket_count: u64,
    /// Number of objects
    pub object_count: u64,
    /// Storage used in bytes
    pub storage_bytes: u64,
    /// Requests per second
    pub requests_per_sec: f64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage percentage
    pub memory_percent: f64,
    /// Disk usage percentage
    pub disk_percent: f64,
    /// Pending replication events
    pub pending_replications: u64,
    /// Uptime in seconds
    pub uptime_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_rule_matches() {
        let rule = ReplicationRule {
            id: "test".to_string(),
            enabled: true,
            source_bucket: "source".to_string(),
            destination_bucket: "dest".to_string(),
            target_nodes: vec![],
            prefix_filter: Some("logs/".to_string()),
            tag_filters: {
                let mut m = HashMap::new();
                m.insert("env".to_string(), "prod".to_string());
                m
            },
            mode: ReplicationMode::Async,
            priority: 0,
            replicate_deletes: true,
            replicate_existing: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Matches prefix and tags
        let mut tags = HashMap::new();
        tags.insert("env".to_string(), "prod".to_string());
        assert!(rule.matches("logs/app.log", &tags));

        // Wrong prefix
        assert!(!rule.matches("data/file.txt", &tags));

        // Wrong tag
        tags.insert("env".to_string(), "dev".to_string());
        assert!(!rule.matches("logs/app.log", &tags));
    }

    #[test]
    fn test_cluster_node_status() {
        let mut node = ClusterNode::new(
            "node1".to_string(),
            "Node 1".to_string(),
            "http://localhost:9000".to_string(),
            "http://localhost:9001".to_string(),
        );

        assert!(!node.is_healthy()); // Starting

        node.status = NodeStatus::Healthy;
        assert!(node.is_healthy());
        assert!(node.can_accept_writes());
        assert!(node.can_accept_reads());

        node.role = NodeRole::Replica;
        assert!(!node.can_accept_writes());
        assert!(node.can_accept_reads());

        node.role = NodeRole::Witness;
        assert!(!node.can_accept_reads());
    }
}
