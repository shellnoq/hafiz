//! Bucket Policy and ACL handlers
//!
//! S3-compatible policy and ACL management endpoints.

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use hafiz_core::{
    types::{
        AccessControlPolicy, AclHeaders, CannedAcl, Grant, Grantee, Owner, Permission,
        PolicyDocument,
    },
    utils::generate_request_id,
    Error,
};
use tracing::{debug, error, info};

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

fn success_response(status: StatusCode, body: String, request_id: &str) -> Response {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/xml")
        .header("x-amz-request-id", request_id)
        .body(Body::from(body))
        .unwrap()
}

fn success_response_json(status: StatusCode, body: String, request_id: &str) -> Response {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("x-amz-request-id", request_id)
        .body(Body::from(body))
        .unwrap()
}

fn no_content_response(request_id: &str) -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("x-amz-request-id", request_id)
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// Bucket Policy Handlers
// ============================================================================

/// GET /{bucket}?policy - Get bucket policy
pub async fn get_bucket_policy(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetBucketPolicy bucket={} request_id={}", bucket, request_id);

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

    // Get bucket policy from metadata
    match state.metadata.get_bucket_policy(&bucket).await {
        Ok(Some(policy_json)) => {
            success_response_json(StatusCode::OK, policy_json, &request_id)
        }
        Ok(None) => {
            error_response(Error::NoSuchBucketPolicy, &request_id)
        }
        Err(e) => {
            error!("Error getting bucket policy: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// PUT /{bucket}?policy - Put bucket policy
pub async fn put_bucket_policy(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutBucketPolicy bucket={} request_id={}", bucket, request_id);

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

    // Parse and validate policy JSON
    let policy_json = match String::from_utf8(body.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            return error_response(
                Error::MalformedPolicy("Invalid UTF-8 in policy document".into()),
                &request_id,
            );
        }
    };

    // Validate policy structure
    match serde_json::from_str::<PolicyDocument>(&policy_json) {
        Ok(policy) => {
            // Additional validation
            if policy.statement.is_empty() {
                return error_response(
                    Error::MalformedPolicy("Policy must contain at least one statement".into()),
                    &request_id,
                );
            }

            info!("Valid policy with {} statements", policy.statement.len());
        }
        Err(e) => {
            return error_response(
                Error::MalformedPolicy(format!("Invalid policy JSON: {}", e)),
                &request_id,
            );
        }
    }

    // Store bucket policy
    match state.metadata.put_bucket_policy(&bucket, &policy_json).await {
        Ok(_) => {
            info!("Bucket policy set for {}", bucket);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error setting bucket policy: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// DELETE /{bucket}?policy - Delete bucket policy
pub async fn delete_bucket_policy(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("DeleteBucketPolicy bucket={} request_id={}", bucket, request_id);

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

    // Delete bucket policy
    match state.metadata.delete_bucket_policy(&bucket).await {
        Ok(_) => {
            info!("Bucket policy deleted for {}", bucket);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error deleting bucket policy: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============================================================================
// Bucket ACL Handlers
// ============================================================================

/// GET /{bucket}?acl - Get bucket ACL
pub async fn get_bucket_acl(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetBucketAcl bucket={} request_id={}", bucket, request_id);

    // Check if bucket exists
    let bucket_info = match state.metadata.get_bucket(&bucket).await {
        Ok(Some(b)) => b,
        Ok(None) => {
            return error_response(Error::NoSuchBucketNamed(bucket), &request_id);
        }
        Err(e) => {
            error!("Error checking bucket: {}", e);
            return error_response(e, &request_id);
        }
    };

    // Get ACL from metadata or return default
    let acl = match state.metadata.get_bucket_acl(&bucket).await {
        Ok(Some(acl_xml)) => acl_xml,
        Ok(None) => {
            // Return default private ACL
            let owner = Owner::with_name(&bucket_info.owner, &bucket_info.owner);
            AccessControlPolicy::from_canned(owner, CannedAcl::Private).to_xml()
        }
        Err(e) => {
            error!("Error getting bucket ACL: {}", e);
            return error_response(e, &request_id);
        }
    };

    success_response(StatusCode::OK, acl, &request_id)
}

/// PUT /{bucket}?acl - Put bucket ACL
pub async fn put_bucket_acl(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutBucketAcl bucket={} request_id={}", bucket, request_id);

    // Check if bucket exists
    let bucket_info = match state.metadata.get_bucket(&bucket).await {
        Ok(Some(b)) => b,
        Ok(None) => {
            return error_response(Error::NoSuchBucketNamed(bucket), &request_id);
        }
        Err(e) => {
            error!("Error checking bucket: {}", e);
            return error_response(e, &request_id);
        }
    };

    let owner = Owner::with_name(&bucket_info.owner, &bucket_info.owner);

    // Check for canned ACL header
    let acl_xml = if let Some(canned) = headers
        .get("x-amz-acl")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<CannedAcl>().ok())
    {
        AccessControlPolicy::from_canned(owner, canned).to_xml()
    } else if !body.is_empty() {
        // Parse ACL from body (XML)
        // For now, just store it directly after basic validation
        let acl_str = match String::from_utf8(body.to_vec()) {
            Ok(s) => s,
            Err(_) => {
                return error_response(
                    Error::MalformedACL("Invalid UTF-8 in ACL document".into()),
                    &request_id,
                );
            }
        };

        // Basic validation - check it looks like XML
        if !acl_str.contains("<AccessControlPolicy") {
            return error_response(
                Error::MalformedACL("Invalid ACL XML".into()),
                &request_id,
            );
        }

        acl_str
    } else {
        // Check for grant headers
        let acl_headers = AclHeaders {
            canned_acl: headers
                .get("x-amz-acl")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            grant_read: headers
                .get("x-amz-grant-read")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_write: headers
                .get("x-amz-grant-write")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_read_acp: headers
                .get("x-amz-grant-read-acp")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_write_acp: headers
                .get("x-amz-grant-write-acp")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_full_control: headers
                .get("x-amz-grant-full-control")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
        };

        if acl_headers.has_acl_headers() {
            acl_headers.build_acl(owner).to_xml()
        } else {
            // Default to private
            AccessControlPolicy::from_canned(owner, CannedAcl::Private).to_xml()
        }
    };

    // Store bucket ACL
    match state.metadata.put_bucket_acl(&bucket, &acl_xml).await {
        Ok(_) => {
            info!("Bucket ACL set for {}", bucket);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error setting bucket ACL: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============================================================================
// Object ACL Handlers
// ============================================================================

/// GET /{bucket}/{key}?acl - Get object ACL
pub async fn get_object_acl(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    version_id: Option<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetObjectAcl bucket={} key={} request_id={}", bucket, key, request_id);

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

    // Check if object exists
    let object = match state.metadata.get_object(&bucket, &key, version_id.as_deref()).await {
        Ok(Some(obj)) => obj,
        Ok(None) => {
            return error_response(Error::NoSuchKeyNamed(key), &request_id);
        }
        Err(e) => {
            error!("Error checking object: {}", e);
            return error_response(e, &request_id);
        }
    };

    // Get ACL from metadata or return default
    let acl = match state.metadata.get_object_acl(&bucket, &key, version_id.as_deref()).await {
        Ok(Some(acl_xml)) => acl_xml,
        Ok(None) => {
            // Return default private ACL
            let owner = Owner::with_name(&object.owner, &object.owner);
            AccessControlPolicy::from_canned(owner, CannedAcl::Private).to_xml()
        }
        Err(e) => {
            error!("Error getting object ACL: {}", e);
            return error_response(e, &request_id);
        }
    };

    success_response(StatusCode::OK, acl, &request_id)
}

/// PUT /{bucket}/{key}?acl - Put object ACL
pub async fn put_object_acl(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    headers: axum::http::HeaderMap,
    version_id: Option<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutObjectAcl bucket={} key={} request_id={}", bucket, key, request_id);

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

    // Check if object exists
    let object = match state.metadata.get_object(&bucket, &key, version_id.as_deref()).await {
        Ok(Some(obj)) => obj,
        Ok(None) => {
            return error_response(Error::NoSuchKeyNamed(key), &request_id);
        }
        Err(e) => {
            error!("Error checking object: {}", e);
            return error_response(e, &request_id);
        }
    };

    let owner = Owner::with_name(&object.owner, &object.owner);

    // Check for canned ACL header
    let acl_xml = if let Some(canned) = headers
        .get("x-amz-acl")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<CannedAcl>().ok())
    {
        AccessControlPolicy::from_canned(owner, canned).to_xml()
    } else if !body.is_empty() {
        // Parse ACL from body (XML)
        let acl_str = match String::from_utf8(body.to_vec()) {
            Ok(s) => s,
            Err(_) => {
                return error_response(
                    Error::MalformedACL("Invalid UTF-8 in ACL document".into()),
                    &request_id,
                );
            }
        };

        if !acl_str.contains("<AccessControlPolicy") {
            return error_response(
                Error::MalformedACL("Invalid ACL XML".into()),
                &request_id,
            );
        }

        acl_str
    } else {
        // Check for grant headers
        let acl_headers = AclHeaders {
            canned_acl: headers
                .get("x-amz-acl")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            grant_read: headers
                .get("x-amz-grant-read")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_write: headers
                .get("x-amz-grant-write")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_read_acp: headers
                .get("x-amz-grant-read-acp")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_write_acp: headers
                .get("x-amz-grant-write-acp")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            grant_full_control: headers
                .get("x-amz-grant-full-control")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
        };

        if acl_headers.has_acl_headers() {
            acl_headers.build_acl(owner).to_xml()
        } else {
            return error_response(
                Error::MalformedACL("No ACL specified".into()),
                &request_id,
            );
        }
    };

    // Store object ACL
    match state.metadata.put_object_acl(&bucket, &key, version_id.as_deref(), &acl_xml).await {
        Ok(_) => {
            info!("Object ACL set for {}/{}", bucket, key);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error setting object ACL: {}", e);
            error_response(e, &request_id)
        }
    }
}
