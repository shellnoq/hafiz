//! Storage types for nodes and disks

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

/// Storage node in the cluster
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode {
    /// Unique identifier
    pub id: Uuid,

    /// Hostname
    pub hostname: String,

    /// IP address
    pub ip_address: IpAddr,

    /// gRPC port for internal communication
    pub grpc_port: u16,

    /// Node status
    pub status: NodeStatus,

    /// Availability zone
    pub zone: String,

    /// Rack identifier (for failure domain)
    pub rack: Option<String>,

    /// Total capacity in bytes
    pub capacity_bytes: i64,

    /// Used space in bytes
    pub used_bytes: i64,

    /// Number of disks
    pub disk_count: i32,

    /// Disk information
    pub disks: Vec<DiskInfo>,

    /// Hafiz version
    pub version: Option<String>,

    /// Supported features
    pub features: Vec<String>,

    /// Last heartbeat timestamp
    pub last_heartbeat: DateTime<Utc>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modification timestamp
    pub updated_at: DateTime<Utc>,
}

impl StorageNode {
    /// Create a new storage node
    pub fn new(hostname: String, ip_address: IpAddr, grpc_port: u16) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            hostname,
            ip_address,
            grpc_port,
            status: NodeStatus::Joining,
            zone: "default".to_string(),
            rack: None,
            capacity_bytes: 0,
            used_bytes: 0,
            disk_count: 0,
            disks: Vec::new(),
            version: Some(crate::VERSION.to_string()),
            features: Vec::new(),
            last_heartbeat: now,
            created_at: now,
            updated_at: now,
        }
    }

    /// Get available space in bytes
    pub fn available_bytes(&self) -> i64 {
        self.capacity_bytes - self.used_bytes
    }

    /// Get usage percentage
    pub fn usage_percent(&self) -> f64 {
        if self.capacity_bytes == 0 {
            return 0.0;
        }
        (self.used_bytes as f64 / self.capacity_bytes as f64) * 100.0
    }

    /// Check if node is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, NodeStatus::Active)
    }

    /// Get gRPC endpoint
    pub fn grpc_endpoint(&self) -> String {
        format!("{}:{}", self.ip_address, self.grpc_port)
    }
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    /// Node is joining the cluster
    #[default]
    Joining,
    /// Node is active and healthy
    Active,
    /// Node is being drained (no new data)
    Draining,
    /// Node is offline
    Offline,
    /// Node is being decommissioned
    Decommissioning,
}

impl std::fmt::Display for NodeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeStatus::Joining => write!(f, "joining"),
            NodeStatus::Active => write!(f, "active"),
            NodeStatus::Draining => write!(f, "draining"),
            NodeStatus::Offline => write!(f, "offline"),
            NodeStatus::Decommissioning => write!(f, "decommissioning"),
        }
    }
}

/// Disk information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    /// Disk identifier (e.g., /dev/sda)
    pub id: String,

    /// Mount path
    pub mount_path: String,

    /// Disk status
    pub status: DiskStatus,

    /// Total capacity in bytes
    pub capacity_bytes: i64,

    /// Used space in bytes
    pub used_bytes: i64,

    /// Filesystem type
    pub filesystem: String,

    /// Disk model (if available)
    pub model: Option<String>,

    /// Serial number (if available)
    pub serial: Option<String>,

    /// Is SSD?
    pub is_ssd: bool,

    /// SMART health status
    pub smart_status: Option<SmartStatus>,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,
}

impl DiskInfo {
    /// Get available space
    pub fn available_bytes(&self) -> i64 {
        self.capacity_bytes - self.used_bytes
    }

    /// Check if disk is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.status, DiskStatus::Online)
    }
}

/// Disk status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DiskStatus {
    #[default]
    Online,
    Offline,
    Degraded,
    Failed,
    Initializing,
}

/// SMART health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SmartStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Erasure coding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureConfig {
    /// Number of data shards
    pub data_shards: u8,

    /// Number of parity shards
    pub parity_shards: u8,

    /// Block size in bytes
    pub block_size: u64,
}

impl ErasureConfig {
    /// Create standard EC configuration (4+2)
    pub fn standard() -> Self {
        Self {
            data_shards: 4,
            parity_shards: 2,
            block_size: 1024 * 1024, // 1 MiB
        }
    }

    /// Create high redundancy EC configuration (4+4)
    pub fn high_redundancy() -> Self {
        Self {
            data_shards: 4,
            parity_shards: 4,
            block_size: 1024 * 1024,
        }
    }

    /// Create performance-oriented EC configuration (8+2)
    pub fn performance() -> Self {
        Self {
            data_shards: 8,
            parity_shards: 2,
            block_size: 1024 * 1024,
        }
    }

    /// Total number of shards
    pub fn total_shards(&self) -> u8 {
        self.data_shards + self.parity_shards
    }

    /// Storage overhead ratio
    pub fn overhead_ratio(&self) -> f64 {
        self.parity_shards as f64 / self.data_shards as f64
    }

    /// Maximum tolerable failures
    pub fn max_failures(&self) -> u8 {
        self.parity_shards
    }

    /// Minimum shards needed for reconstruction
    pub fn min_shards(&self) -> u8 {
        self.data_shards
    }
}

impl Default for ErasureConfig {
    fn default() -> Self {
        Self::standard()
    }
}

/// Erasure set (group of shards across nodes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErasureSet {
    /// Set identifier
    pub id: String,

    /// Erasure configuration
    pub config: ErasureConfig,

    /// Node IDs in this set
    pub node_ids: Vec<Uuid>,

    /// Set status
    pub status: ErasureSetStatus,

    /// Number of healthy shards
    pub healthy_shards: u8,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

impl ErasureSet {
    /// Check if set can accept writes
    pub fn can_write(&self) -> bool {
        self.healthy_shards >= self.config.total_shards()
    }

    /// Check if set can serve reads
    pub fn can_read(&self) -> bool {
        self.healthy_shards >= self.config.min_shards()
    }

    /// Check if set needs healing
    pub fn needs_healing(&self) -> bool {
        self.healthy_shards < self.config.total_shards()
    }
}

/// Erasure set status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErasureSetStatus {
    Healthy,
    Degraded,
    Healing,
    Critical,
}

/// Storage pool (logical grouping of nodes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePool {
    /// Pool identifier
    pub id: Uuid,

    /// Pool name
    pub name: String,

    /// Node IDs in this pool
    pub node_ids: Vec<Uuid>,

    /// Default erasure configuration
    pub erasure_config: ErasureConfig,

    /// Total capacity
    pub total_capacity: i64,

    /// Used capacity
    pub used_capacity: i64,

    /// Pool status
    pub status: PoolStatus,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Storage pool status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PoolStatus {
    Online,
    Degraded,
    Offline,
    Maintenance,
}

/// Chunk location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkLocation {
    /// Chunk ID
    pub chunk_id: String,

    /// Node ID
    pub node_id: Uuid,

    /// Disk path on the node
    pub disk_path: String,

    /// Chunk type (data or parity)
    pub chunk_type: ChunkType,

    /// Shard index
    pub shard_index: u8,

    /// Chunk size
    pub size: i64,

    /// Checksum
    pub checksum: String,
}

/// Chunk type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChunkType {
    Data,
    Parity,
}

/// Write result from storage
#[derive(Debug, Clone)]
pub struct WriteResult {
    /// Written chunks
    pub chunks: Vec<ChunkLocation>,

    /// Total bytes written
    pub bytes_written: i64,

    /// Checksum of the entire object
    pub checksum: String,
}

/// Read result from storage
#[derive(Debug, Clone)]
pub struct ReadResult {
    /// Data bytes
    pub data: Vec<u8>,

    /// Was reconstruction needed?
    pub reconstructed: bool,

    /// Number of shards read
    pub shards_read: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_storage_node() {
        let node = StorageNode::new(
            "node1.local".to_string(),
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
            9001,
        );

        assert_eq!(node.hostname, "node1.local");
        assert_eq!(node.grpc_endpoint(), "192.168.1.10:9001");
        assert_eq!(node.status, NodeStatus::Joining);
    }

    #[test]
    fn test_erasure_config() {
        let config = ErasureConfig::standard();
        assert_eq!(config.data_shards, 4);
        assert_eq!(config.parity_shards, 2);
        assert_eq!(config.total_shards(), 6);
        assert_eq!(config.max_failures(), 2);
        assert_eq!(config.min_shards(), 4);
        assert!((config.overhead_ratio() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_node_usage() {
        let mut node = StorageNode::new(
            "node1".to_string(),
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            9001,
        );
        node.capacity_bytes = 1000;
        node.used_bytes = 250;

        assert_eq!(node.available_bytes(), 750);
        assert!((node.usage_percent() - 25.0).abs() < 0.01);
    }
}
