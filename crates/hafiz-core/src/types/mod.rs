//! Core types for Hafiz

mod acl;
mod bucket;
mod common;
mod cors;
mod lifecycle;
mod notification;
mod object;
mod object_lock;
mod policy;
mod presigned;
mod replication;
mod storage;
mod user;

// Re-export everything except modules with duplicates
pub use acl::*;
pub use bucket::*;
pub use common::*;
pub use cors::*;
pub use lifecycle::*;
pub use notification::*;
pub use object::*;
pub use object_lock::*;
pub use policy::*;
pub use presigned::*;
pub use storage::*;

// Re-export from replication (except NodeStatus which conflicts with storage)
pub use replication::{
    ClusterConfig, ClusterMessage, ClusterNode, ClusterStats, ConflictResolution, ConsistencyLevel,
    NodeRole, NodeStats, ReplicationConfig, ReplicationDestination, ReplicationEvent,
    ReplicationEventType, ReplicationMode, ReplicationProgress, ReplicationRule, ReplicationStatus,
};

// Re-export from user (except Owner which conflicts with acl)
pub use user::{Credentials, User, UserQuota};
