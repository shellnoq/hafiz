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
