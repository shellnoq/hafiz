//! Pre-signed URL API endpoints
//!
//! Provides API for generating pre-signed URLs for temporary object access.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use hafiz_auth::generate_presigned_url;
use hafiz_core::types::{PresignedLimits, PresignedMethod, PresignedRequest, PresignedUrl};

use crate::server::AppState;

/// Request body for generating a pre-signed URL
#[derive(Debug, Deserialize)]
pub struct GeneratePresignedUrlRequest {
    /// HTTP method (GET, PUT, DELETE, HEAD)
    pub method: String,
    /// Bucket name
    pub bucket: String,
    /// Object key
    pub key: String,
    /// Expiration time in seconds (default: 3600, max: 604800)
    #[serde(default = "default_expires")]
    pub expires_in: u64,
    /// Content-Type for PUT requests
    pub content_type: Option<String>,
    /// Version ID for versioned objects
    pub version_id: Option<String>,
}

fn default_expires() -> u64 {
    3600
}

/// Response for pre-signed URL generation
#[derive(Debug, Serialize)]
pub struct PresignedUrlResponse {
    /// The pre-signed URL
    pub url: String,
    /// HTTP method to use
    pub method: String,
    /// Expiration timestamp (ISO 8601)
    pub expires_at: String,
    /// Headers to include with the request (for PUT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HeaderPair>>,
}

#[derive(Debug, Serialize)]
pub struct HeaderPair {
    pub name: String,
    pub value: String,
}

/// POST /api/v1/presigned
/// Generate a pre-signed URL
pub async fn generate_presigned(
    State(state): State<AppState>,
    Json(request): Json<GeneratePresignedUrlRequest>,
) -> Result<Json<PresignedUrlResponse>, (StatusCode, String)> {
    // Parse method
    let method: PresignedMethod = request.method.parse().map_err(|e: String| {
        (StatusCode::BAD_REQUEST, e)
    })?;

    // Validate expiration
    let expires_in = PresignedLimits::validate_expires(request.expires_in).map_err(|e| {
        (StatusCode::BAD_REQUEST, e)
    })?;

    // Check if bucket exists
    state.metadata.get_bucket(&request.bucket).await.map_err(|_| {
        (StatusCode::NOT_FOUND, format!("Bucket not found: {}", request.bucket))
    })?;

    // Build the presigned request
    let presigned_request = PresignedRequest {
        method,
        bucket: request.bucket,
        key: request.key,
        expires_in,
        content_type: request.content_type,
        content_md5: None,
        signed_headers: None,
        version_id: request.version_id,
    };

    // Determine the endpoint
    let protocol = if state.config.tls.enabled { "https" } else { "http" };
    let endpoint = format!(
        "{}://{}:{}",
        protocol,
        state.config.server.bind_address,
        state.config.server.port
    );

    // Generate the pre-signed URL
    let presigned = generate_presigned_url(
        &presigned_request,
        &endpoint,
        &state.config.auth.root_access_key,
        &state.config.auth.root_secret_key,
        hafiz_core::DEFAULT_REGION,
    ).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?;

    // Convert headers
    let headers = presigned.headers.map(|h| {
        h.into_iter()
            .map(|(name, value)| HeaderPair { name, value })
            .collect()
    });

    Ok(Json(PresignedUrlResponse {
        url: presigned.url,
        method: presigned.method,
        expires_at: presigned.expires_at.to_rfc3339(),
        headers,
    }))
}

/// POST /api/v1/presigned/download/:bucket/:key
/// Generate a pre-signed download URL (shortcut)
pub async fn generate_presigned_download(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<Json<PresignedUrlResponse>, (StatusCode, String)> {
    let request = GeneratePresignedUrlRequest {
        method: "GET".to_string(),
        bucket,
        key,
        expires_in: 3600,
        content_type: None,
        version_id: None,
    };
    generate_presigned(State(state), Json(request)).await
}

/// POST /api/v1/presigned/upload/:bucket/:key
/// Generate a pre-signed upload URL (shortcut)
pub async fn generate_presigned_upload(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<Json<PresignedUrlResponse>, (StatusCode, String)> {
    let request = GeneratePresignedUrlRequest {
        method: "PUT".to_string(),
        bucket,
        key,
        expires_in: 3600,
        content_type: None,
        version_id: None,
    };
    generate_presigned(State(state), Json(request)).await
}
