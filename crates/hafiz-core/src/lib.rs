//! Hafiz Core Library
//!
//! Core types, traits, and utilities for the Hafiz object storage system.

pub mod config;
pub mod error;
pub mod types;
pub mod utils;

pub use config::HafizConfig;
pub use error::{Error, Result};

/// Hafiz version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default S3 region
pub const DEFAULT_REGION: &str = "us-east-1";

/// Maximum object size (5 TiB)
pub const MAX_OBJECT_SIZE: u64 = 5 * 1024 * 1024 * 1024 * 1024;

/// Maximum number of parts in multipart upload
pub const MAX_PARTS: u32 = 10_000;

/// Minimum part size (5 MiB)
pub const MIN_PART_SIZE: u64 = 5 * 1024 * 1024;

/// Maximum bucket name length
pub const MAX_BUCKET_NAME_LENGTH: usize = 63;

/// Minimum bucket name length
pub const MIN_BUCKET_NAME_LENGTH: usize = 3;

/// Maximum object key length
pub const MAX_KEY_LENGTH: usize = 1024;
