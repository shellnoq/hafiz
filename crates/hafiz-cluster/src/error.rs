//! Cluster error types

use thiserror::Error;

/// Result type for cluster operations
pub type ClusterResult<T> = Result<T, ClusterError>;

/// Cluster-related errors
#[derive(Error, Debug)]
pub enum ClusterError {
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Node unreachable: {0}")]
    NodeUnreachable(String),

    #[error("Cluster name mismatch: expected {expected}, got {got}")]
    ClusterNameMismatch { expected: String, got: String },

    #[error("Join rejected: {0}")]
    JoinRejected(String),

    #[error("Replication failed: {0}")]
    ReplicationFailed(String),

    #[error("Quorum not reached: needed {needed}, got {got}")]
    QuorumNotReached { needed: u32, got: u32 },

    #[error("No healthy nodes available")]
    NoHealthyNodes,

    #[error("Node already exists: {0}")]
    NodeAlreadyExists(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Transport error: {0}")]
    Transport(String),

    #[error("Checksum mismatch: expected {expected}, got {got}")]
    ChecksumMismatch { expected: String, got: String },

    #[error("Conflict detected: {0}")]
    Conflict(String),

    #[error("Storage error: {0}")]
    Storage(#[from] hafiz_core::error::HafizError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}
