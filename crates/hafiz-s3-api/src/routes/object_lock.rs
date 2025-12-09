//! Object Lock / WORM handlers
//!
//! S3-compatible Object Lock for regulatory compliance.
//!
//! Bucket-level endpoints:
//! - GET /{bucket}?object-lock - Get bucket Object Lock configuration
//! - PUT /{bucket}?object-lock - Put bucket Object Lock configuration
//!
//! Object-level endpoints:
//! - GET /{bucket}/{key}?retention - Get object retention
//! - PUT /{bucket}/{key}?retention - Put object retention
//! - GET /{bucket}/{key}?legal-hold - Get object legal hold
//! - PUT /{bucket}/{key}?legal-hold - Put object legal hold

use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use hafiz_core::{
    types::{
        ObjectLockConfiguration, ObjectRetention, ObjectLegalHold,
        RetentionMode, LegalHoldStatus, ObjectLockError,
    },
    utils::generate_request_id,
    Error,
};
use serde::Deserialize;
use tracing::{debug, error, info, warn};

use crate::server::AppState;

// ============================================================================
// Response Helpers
// ============================================================================

fn error_response(err: Error, request_id: &str) -> Response {
    let status = StatusCode::from_u16(err.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let s3_error = hafiz_core::error::S3Error::from(err).with_request_id(request_id);

    Response::builder()
        .status(status)
        .header("Content-Type", "application/xml")
        .header("x-amz-request-id", request_id)
        .body(Body::from(s3_error.to_xml()))
        .unwrap()
}

fn success_response_xml(status: StatusCode, body: String, request_id: &str) -> Response {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/xml")
        .header("x-amz-request-id", request_id)
        .body(Body::from(body))
        .unwrap()
}

fn no_content_response(request_id: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("x-amz-request-id", request_id)
        .body(Body::empty())
        .unwrap()
}

fn object_lock_error_response(code: &str, message: &str, request_id: &str) -> Response {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<e>
<Code>{}</Code>
<Message>{}</Message>
<RequestId>{}</RequestId>
</e>"#,
        code, message, request_id
    );

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/xml")
        .header("x-amz-request-id", request_id)
        .body(Body::from(xml))
        .unwrap()
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize, Default)]
pub struct RetentionQuery {
    #[serde(rename = "versionId")]
    pub version_id: Option<String>,
}

// ============================================================================
// Bucket Object Lock Configuration
// ============================================================================

/// GET /{bucket}?object-lock - Get bucket Object Lock configuration
pub async fn get_bucket_object_lock_config(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetObjectLockConfiguration bucket={} request_id={}", bucket, request_id);

    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return error_response(Error::NoSuchBucketNamed(bucket), &request_id);
        }
        Err(e) => {
            error!("Error checking bucket: {}", e);
            return error_response(e, &request_id);
        }
    }

    // Get Object Lock configuration
    match state.metadata.get_bucket_object_lock_config(&bucket).await {
        Ok(Some(config_xml)) => {
            info!("GetObjectLockConfiguration success bucket={}", bucket);
            success_response_xml(StatusCode::OK, config_xml, &request_id)
        }
        Ok(None) => {
            // Object Lock not configured - return error per S3 spec
            object_lock_error_response(
                "ObjectLockConfigurationNotFoundError",
                "Object Lock configuration does not exist for this bucket",
                &request_id,
            )
        }
        Err(e) => {
            error!("Error getting Object Lock config: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// PUT /{bucket}?object-lock - Put bucket Object Lock configuration
pub async fn put_bucket_object_lock_config(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutObjectLockConfiguration bucket={} request_id={}", bucket, request_id);

    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return error_response(Error::NoSuchBucketNamed(bucket), &request_id);
        }
        Err(e) => {
            error!("Error checking bucket: {}", e);
            return error_response(e, &request_id);
        }
    }

    // Check if Object Lock is already enabled (cannot change once enabled)
    if let Ok(Some(existing_xml)) = state.metadata.get_bucket_object_lock_config(&bucket).await {
        if let Ok(existing) = ObjectLockConfiguration::from_xml(&existing_xml) {
            if existing.is_enabled() {
                // Can only update default retention, not disable
                // For now, allow updates
                debug!("Updating existing Object Lock config for bucket {}", bucket);
            }
        }
    }

    // Parse XML body
    let xml_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => {
            return error_response(
                Error::MalformedXML("Invalid UTF-8 in request body".to_string()),
                &request_id,
            );
        }
    };

    // Parse and validate configuration
    let config = match ObjectLockConfiguration::from_xml(xml_str) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse Object Lock config: {}", e);
            return error_response(Error::MalformedXML(e), &request_id);
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Invalid Object Lock configuration: {}", e);
        return object_lock_error_response(
            "InvalidObjectLockConfiguration",
            &e.to_string(),
            &request_id,
        );
    }

    // Serialize back to clean XML
    let clean_xml = match config.to_xml() {
        Ok(xml) => xml,
        Err(e) => {
            error!("Failed to serialize Object Lock config: {}", e);
            return error_response(Error::InternalError(e), &request_id);
        }
    };

    // Store in metadata
    match state.metadata.put_bucket_object_lock_config(&bucket, &clean_xml).await {
        Ok(_) => {
            info!("PutObjectLockConfiguration success bucket={} enabled={}",
                  bucket, config.is_enabled());
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error storing Object Lock config: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============================================================================
// Object Retention
// ============================================================================

/// GET /{bucket}/{key}?retention - Get object retention
pub async fn get_object_retention(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(query): Query<RetentionQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetObjectRetention bucket={} key={} request_id={}", bucket, key, request_id);

    // Check if bucket exists and has Object Lock enabled
    if let Err(resp) = check_object_lock_enabled(&state, &bucket, &request_id).await {
        return resp;
    }

    // Check if object exists
    let version_id = query.version_id.as_deref();
    if let Err(resp) = check_object_exists(&state, &bucket, &key, version_id, &request_id).await {
        return resp;
    }

    // Get object retention
    match state.metadata.get_object_retention(&bucket, &key, version_id).await {
        Ok(Some(retention_xml)) => {
            info!("GetObjectRetention success bucket={} key={}", bucket, key);
            success_response_xml(StatusCode::OK, retention_xml, &request_id)
        }
        Ok(None) => {
            // No retention set
            object_lock_error_response(
                "NoSuchObjectLockConfiguration",
                "The specified object does not have a retention configuration",
                &request_id,
            )
        }
        Err(e) => {
            error!("Error getting object retention: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// PUT /{bucket}/{key}?retention - Put object retention
pub async fn put_object_retention(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
    Query(query): Query<RetentionQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutObjectRetention bucket={} key={} request_id={}", bucket, key, request_id);

    // Check if bucket exists and has Object Lock enabled
    if let Err(resp) = check_object_lock_enabled(&state, &bucket, &request_id).await {
        return resp;
    }

    // Check if object exists
    let version_id = query.version_id.as_deref();
    if let Err(resp) = check_object_exists(&state, &bucket, &key, version_id, &request_id).await {
        return resp;
    }

    // Check for bypass governance header
    let bypass_governance = headers
        .get("x-amz-bypass-governance-retention")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    // Check existing retention
    if let Ok(Some(existing_xml)) = state.metadata.get_object_retention(&bucket, &key, version_id).await {
        if let Ok(existing) = ObjectRetention::from_xml(&existing_xml) {
            if !existing.can_modify(bypass_governance) {
                warn!("Cannot modify retention: object is locked");
                return object_lock_error_response(
                    "AccessDenied",
                    "Object is locked and cannot be modified",
                    &request_id,
                );
            }
        }
    }

    // Parse XML body
    let xml_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => {
            return error_response(
                Error::MalformedXML("Invalid UTF-8 in request body".to_string()),
                &request_id,
            );
        }
    };

    // Parse retention
    let retention = match ObjectRetention::from_xml(xml_str) {
        Ok(r) => r,
        Err(e) => {
            error!("Failed to parse retention: {}", e);
            return error_response(Error::MalformedXML(e), &request_id);
        }
    };

    // Serialize back to clean XML
    let clean_xml = match retention.to_xml() {
        Ok(xml) => xml,
        Err(e) => {
            error!("Failed to serialize retention: {}", e);
            return error_response(Error::InternalError(e), &request_id);
        }
    };

    // Store retention
    match state.metadata.put_object_retention(&bucket, &key, version_id, &clean_xml).await {
        Ok(_) => {
            info!("PutObjectRetention success bucket={} key={} mode={}",
                  bucket, key, retention.mode);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error storing object retention: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============================================================================
// Object Legal Hold
// ============================================================================

/// GET /{bucket}/{key}?legal-hold - Get object legal hold
pub async fn get_object_legal_hold(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(query): Query<RetentionQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetObjectLegalHold bucket={} key={} request_id={}", bucket, key, request_id);

    // Check if bucket exists and has Object Lock enabled
    if let Err(resp) = check_object_lock_enabled(&state, &bucket, &request_id).await {
        return resp;
    }

    // Check if object exists
    let version_id = query.version_id.as_deref();
    if let Err(resp) = check_object_exists(&state, &bucket, &key, version_id, &request_id).await {
        return resp;
    }

    // Get legal hold
    match state.metadata.get_object_legal_hold(&bucket, &key, version_id).await {
        Ok(Some(hold_xml)) => {
            info!("GetObjectLegalHold success bucket={} key={}", bucket, key);
            success_response_xml(StatusCode::OK, hold_xml, &request_id)
        }
        Ok(None) => {
            // No legal hold - return OFF status
            let hold = ObjectLegalHold::off();
            let xml = hold.to_xml().unwrap_or_default();
            success_response_xml(StatusCode::OK, xml, &request_id)
        }
        Err(e) => {
            error!("Error getting object legal hold: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// PUT /{bucket}/{key}?legal-hold - Put object legal hold
pub async fn put_object_legal_hold(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(query): Query<RetentionQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutObjectLegalHold bucket={} key={} request_id={}", bucket, key, request_id);

    // Check if bucket exists and has Object Lock enabled
    if let Err(resp) = check_object_lock_enabled(&state, &bucket, &request_id).await {
        return resp;
    }

    // Check if object exists
    let version_id = query.version_id.as_deref();
    if let Err(resp) = check_object_exists(&state, &bucket, &key, version_id, &request_id).await {
        return resp;
    }

    // Parse XML body
    let xml_str = match std::str::from_utf8(&body) {
        Ok(s) => s,
        Err(_) => {
            return error_response(
                Error::MalformedXML("Invalid UTF-8 in request body".to_string()),
                &request_id,
            );
        }
    };

    // Parse legal hold
    let hold = match ObjectLegalHold::from_xml(xml_str) {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to parse legal hold: {}", e);
            return error_response(Error::MalformedXML(e), &request_id);
        }
    };

    // Serialize back to clean XML
    let clean_xml = match hold.to_xml() {
        Ok(xml) => xml,
        Err(e) => {
            error!("Failed to serialize legal hold: {}", e);
            return error_response(Error::InternalError(e), &request_id);
        }
    };

    // Store legal hold
    match state.metadata.put_object_legal_hold(&bucket, &key, version_id, &clean_xml).await {
        Ok(_) => {
            info!("PutObjectLegalHold success bucket={} key={} status={}",
                  bucket, key, hold.status);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error storing object legal hold: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Check if Object Lock is enabled on the bucket
async fn check_object_lock_enabled(
    state: &AppState,
    bucket: &str,
    request_id: &str,
) -> Result<(), Response> {
    // Check if bucket exists
    match state.metadata.get_bucket(bucket).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err(error_response(Error::NoSuchBucketNamed(bucket.to_string()), request_id));
        }
        Err(e) => {
            return Err(error_response(e, request_id));
        }
    }

    // Check Object Lock configuration
    match state.metadata.get_bucket_object_lock_config(bucket).await {
        Ok(Some(config_xml)) => {
            match ObjectLockConfiguration::from_xml(&config_xml) {
                Ok(config) if config.is_enabled() => Ok(()),
                _ => Err(object_lock_error_response(
                    "InvalidRequest",
                    "Object Lock is not enabled for this bucket",
                    request_id,
                )),
            }
        }
        Ok(None) => Err(object_lock_error_response(
            "InvalidRequest",
            "Object Lock is not enabled for this bucket",
            request_id,
        )),
        Err(e) => Err(error_response(e, request_id)),
    }
}

/// Check if object exists
async fn check_object_exists(
    state: &AppState,
    bucket: &str,
    key: &str,
    version_id: Option<&str>,
    request_id: &str,
) -> Result<(), Response> {
    let exists = match version_id {
        Some(vid) => state.metadata.get_object_version(bucket, key, vid).await,
        None => state.metadata.get_object(bucket, key).await,
    };

    match exists {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(error_response(
            Error::NoSuchKey(key.to_string()),
            request_id,
        )),
        Err(e) => Err(error_response(e, request_id)),
    }
}

/// Check if object can be deleted considering Object Lock
pub async fn can_delete_object(
    state: &AppState,
    bucket: &str,
    key: &str,
    version_id: Option<&str>,
    bypass_governance: bool,
) -> Result<bool, Error> {
    // Check legal hold
    if let Some(hold_xml) = state.metadata.get_object_legal_hold(bucket, key, version_id).await? {
        if let Ok(hold) = ObjectLegalHold::from_xml(&hold_xml) {
            if hold.is_active() {
                return Ok(false);
            }
        }
    }

    // Check retention
    if let Some(retention_xml) = state.metadata.get_object_retention(bucket, key, version_id).await? {
        if let Ok(retention) = ObjectRetention::from_xml(&retention_xml) {
            if !retention.can_delete(bypass_governance) {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Get Object Lock error message for deletion attempt
pub async fn get_lock_error_message(
    state: &AppState,
    bucket: &str,
    key: &str,
    version_id: Option<&str>,
) -> Option<String> {
    // Check legal hold
    if let Ok(Some(hold_xml)) = state.metadata.get_object_legal_hold(bucket, key, version_id).await {
        if let Ok(hold) = ObjectLegalHold::from_xml(&hold_xml) {
            if hold.is_active() {
                return Some("Object is under legal hold".to_string());
            }
        }
    }

    // Check retention
    if let Ok(Some(retention_xml)) = state.metadata.get_object_retention(bucket, key, version_id).await {
        if let Ok(retention) = ObjectRetention::from_xml(&retention_xml) {
            if !retention.is_expired() {
                return Some(format!(
                    "Object is locked in {} mode until {}",
                    retention.mode, retention.retain_until_date
                ));
            }
        }
    }

    None
}
