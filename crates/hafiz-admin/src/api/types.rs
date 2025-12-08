//! API response types

use serde::{Deserialize, Serialize};

/// Dashboard statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashboardStats {
    pub total_buckets: i64,
    pub total_objects: i64,
    pub total_size: i64,
    pub total_users: i64,
    pub recent_buckets: Vec<BucketInfo>,
}

/// Bucket information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BucketInfo {
    pub name: String,
    pub object_count: i64,
    pub size: i64,
    pub created_at: String,
    pub versioning_enabled: bool,
    pub encryption_enabled: bool,
}

/// Object information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObjectInfo {
    pub key: String,
    pub size: i64,
    pub etag: String,
    pub content_type: String,
    pub last_modified: String,
    pub version_id: Option<String>,
    pub encryption: Option<String>,
}

/// Object listing response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObjectListing {
    pub objects: Vec<ObjectInfo>,
    pub common_prefixes: Vec<String>,
    pub is_truncated: bool,
    pub next_marker: Option<String>,
}

/// User information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub name: String,
    pub access_key: String,
    pub email: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

/// Server information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub s3_endpoint: String,
    pub admin_endpoint: String,
    pub storage_backend: String,
    pub database_type: String,
    pub uptime: String,
}

/// Health check status
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub storage_ok: bool,
    pub database_ok: bool,
    pub timestamp: String,
}

/// API error response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

// ============================================================================
// Cluster Types
// ============================================================================

/// Cluster status response
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ClusterStatus {
    pub enabled: bool,
    pub cluster_name: String,
    pub local_node: NodeInfo,
    pub stats: ClusterStats,
}

/// Node information
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct NodeInfo {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub role: String,
    pub status: String,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub joined_at: String,
    pub last_heartbeat: String,
    pub version: String,
}

/// Cluster statistics
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ClusterStats {
    pub total_nodes: u32,
    pub healthy_nodes: u32,
    pub primary_nodes: u32,
    pub replica_nodes: u32,
    pub total_objects: u64,
    pub total_storage_bytes: u64,
    pub pending_replications: u64,
    pub failed_replications: u64,
    pub replication_lag_secs: u64,
}

/// Nodes list response
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct NodesList {
    pub nodes: Vec<NodeInfo>,
    pub total: usize,
    pub healthy: usize,
}

/// Replication rule
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ReplicationRule {
    pub id: String,
    pub enabled: bool,
    pub source_bucket: String,
    pub destination_bucket: String,
    pub target_nodes: Vec<String>,
    pub prefix_filter: Option<String>,
    pub mode: String,
    pub priority: i32,
    pub replicate_deletes: bool,
    pub replicate_existing: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Replication rules list
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ReplicationRulesList {
    pub rules: Vec<ReplicationRule>,
    pub total: usize,
}

/// Replication statistics
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ReplicationStats {
    pub events_processed: u64,
    pub successful: u64,
    pub failed: u64,
    pub pending: u64,
    pub in_progress: u64,
    pub bytes_replicated: u64,
    pub avg_latency_ms: f64,
}

/// Create replication rule request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateReplicationRuleRequest {
    pub source_bucket: String,
    pub destination_bucket: Option<String>,
    pub target_nodes: Option<Vec<String>>,
    pub prefix_filter: Option<String>,
    pub mode: Option<String>,
    pub replicate_deletes: Option<bool>,
}

/// Cluster health response
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ClusterHealth {
    pub status: String,
    pub cluster_enabled: bool,
    pub node_count: usize,
    pub timestamp: String,
}

/// Pre-signed URL request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresignedUrlRequest {
    pub method: String,
    pub bucket: String,
    pub key: String,
    pub expires_in: u64,
}

/// Pre-signed URL response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PresignedUrlResponse {
    pub url: String,
    pub method: String,
    pub expires_at: String,
}
