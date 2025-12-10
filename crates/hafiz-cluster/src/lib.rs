//! Hafiz Cluster - Multi-node replication and cluster management
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Hafiz Cluster                            │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                             │
//! │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐   │
//! │  │ ClusterManager│  │   Discovery   │  │  Replicator   │   │
//! │  │               │  │               │  │               │   │
//! │  │ - Node mgmt   │  │ - Seed nodes  │  │ - Event queue │   │
//! │  │ - Health check│  │ - Heartbeats  │  │ - Async copy  │   │
//! │  │ - State sync  │  │ - Auto-join   │  │ - Retry logic │   │
//! │  └───────┬───────┘  └───────┬───────┘  └───────┬───────┘   │
//! │          │                  │                  │           │
//! │          └──────────────────┼──────────────────┘           │
//! │                             │                               │
//! │                    ┌────────┴────────┐                      │
//! │                    │    Transport    │                      │
//! │                    │   (HTTP/gRPC)   │                      │
//! │                    └─────────────────┘                      │
//! │                                                             │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Features
//!
//! - **Automatic Discovery**: Nodes find each other via seed nodes
//! - **Async Replication**: Non-blocking object replication
//! - **Consistency Levels**: One, Quorum, or All
//! - **Conflict Resolution**: Last-write-wins, first-write-wins, etc.
//! - **Health Monitoring**: Automatic failure detection
//! - **TLS Support**: Encrypted cluster communication

mod cluster;
mod discovery;
mod error;
mod replicator;
mod transport;

pub use cluster::ClusterManager;
pub use discovery::DiscoveryService;
pub use error::{ClusterError, ClusterResult};
pub use replicator::Replicator;
pub use transport::ClusterTransport;

// Re-export types from core
pub use hafiz_core::types::{
    ClusterConfig, ClusterMessage, ClusterNode, ClusterStats, ConflictResolution, ConsistencyLevel,
    NodeId, NodeRole, NodeStats, NodeStatus, ReplicationEvent, ReplicationEventType,
    ReplicationMode, ReplicationProgress, ReplicationRule, ReplicationStatus,
};
