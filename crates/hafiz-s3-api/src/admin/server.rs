//! Server information and health check endpoints

use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::time::Instant;

use crate::server::AppState;

/// Server information response
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub s3_endpoint: String,
    pub admin_endpoint: String,
    pub storage_backend: String,
    pub database_type: String,
    pub uptime: String,
    pub features: ServerFeatures,
}

/// Server features
#[derive(Debug, Serialize)]
pub struct ServerFeatures {
    pub versioning: bool,
    pub multipart_upload: bool,
    pub server_side_encryption: bool,
    pub customer_encryption: bool,
    pub lifecycle: bool,
    pub tagging: bool,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub status: String,
    pub checks: HealthChecks,
    pub timestamp: String,
}

/// Individual health checks
#[derive(Debug, Serialize)]
pub struct HealthChecks {
    pub storage: HealthStatus,
    pub database: HealthStatus,
    pub memory: HealthStatus,
}

/// Health status for a component
#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub message: Option<String>,
    pub latency_ms: Option<u64>,
}

/// Get server information
pub async fn get_server_info(
    State(state): State<AppState>,
) -> Result<Json<ServerInfo>, (StatusCode, String)> {
    // Calculate uptime
    let uptime = format_uptime(state.start_time.elapsed());
    
    // Storage backend is always local filesystem for now
    let storage_backend = "Local Filesystem".to_string();
    
    // Determine database type
    let database_type = if state.config.database.url.contains("postgres") {
        "PostgreSQL".to_string()
    } else if state.config.database.url.contains("sqlite") || 
              state.config.database.url.ends_with(".db") {
        "SQLite".to_string()
    } else {
        "Unknown".to_string()
    };
    
    Ok(Json(ServerInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        s3_endpoint: format!("http://{}:{}", state.config.server.bind_address, state.config.server.port),
        admin_endpoint: format!("http://{}:{}/api/v1", state.config.server.bind_address, state.config.server.port),
        storage_backend,
        database_type,
        uptime,
        features: ServerFeatures {
            versioning: true,
            multipart_upload: true,
            server_side_encryption: state.config.encryption.enabled,
            customer_encryption: state.config.encryption.sse_c_enabled,
            lifecycle: true,
            tagging: true,
        },
    }))
}

/// Health check endpoint
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<HealthCheck>, (StatusCode, String)> {
    let mut overall_status = "healthy";
    
    // Check storage
    let storage_check = check_storage(&state).await;
    if storage_check.status != "ok" {
        overall_status = "degraded";
    }
    
    // Check database
    let database_check = check_database(&state).await;
    if database_check.status != "ok" {
        overall_status = "degraded";
    }
    
    // Check memory
    let memory_check = check_memory();
    if memory_check.status != "ok" {
        overall_status = "degraded";
    }
    
    Ok(Json(HealthCheck {
        status: overall_status.to_string(),
        checks: HealthChecks {
            storage: storage_check,
            database: database_check,
            memory: memory_check,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
}

/// Check storage health
async fn check_storage(state: &AppState) -> HealthStatus {
    let start = Instant::now();
    
    // Try to access storage
    let storage = &state.storage;
    
    // Simple check - verify we can get storage info
    match storage.health_check().await {
        Ok(_) => HealthStatus {
            status: "ok".to_string(),
            message: None,
            latency_ms: Some(start.elapsed().as_millis() as u64),
        },
        Err(e) => HealthStatus {
            status: "error".to_string(),
            message: Some(e.to_string()),
            latency_ms: Some(start.elapsed().as_millis() as u64),
        },
    }
}

/// Check database health
async fn check_database(state: &AppState) -> HealthStatus {
    let start = Instant::now();
    
    let metadata = &state.metadata;
    
    // Try a simple query
    match metadata.list_buckets().await {
        Ok(_) => HealthStatus {
            status: "ok".to_string(),
            message: None,
            latency_ms: Some(start.elapsed().as_millis() as u64),
        },
        Err(e) => HealthStatus {
            status: "error".to_string(),
            message: Some(e.to_string()),
            latency_ms: Some(start.elapsed().as_millis() as u64),
        },
    }
}

/// Check memory health
fn check_memory() -> HealthStatus {
    // Simple memory check - in production would use proper memory metrics
    HealthStatus {
        status: "ok".to_string(),
        message: None,
        latency_ms: None,
    }
}

/// Format uptime duration
fn format_uptime(duration: std::time::Duration) -> String {
    let total_secs = duration.as_secs();
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;
    
    if days > 0 {
        format!("{}d {}h {}m", days, hours, mins)
    } else if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}
