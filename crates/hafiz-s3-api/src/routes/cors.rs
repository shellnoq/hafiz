//! Bucket CORS Configuration handlers
//!
//! S3-compatible CORS (Cross-Origin Resource Sharing) configuration.
//! Enables web browsers to make cross-origin requests to S3 buckets.
//!
//! Endpoints:
//! - GET /{bucket}?cors - Get bucket CORS configuration
//! - PUT /{bucket}?cors - Set bucket CORS configuration
//! - DELETE /{bucket}?cors - Delete bucket CORS configuration
//! - OPTIONS /{bucket}/{key} - CORS preflight request

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, Method, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use hafiz_core::{
    types::{CorsConfiguration, CorsResponseHeaders},
    utils::generate_request_id,
    Error,
};
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
        .status(StatusCode::NO_CONTENT)
        .header("x-amz-request-id", request_id)
        .body(Body::empty())
        .unwrap()
}

fn cors_error_response(code: &str, message: &str, request_id: &str) -> Response {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Error>
<Code>{}</Code>
<Message>{}</Message>
<RequestId>{}</RequestId>
</Error>"#,
        code, message, request_id
    );
    
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("Content-Type", "application/xml")
        .header("x-amz-request-id", request_id)
        .body(Body::from(xml))
        .unwrap()
}

// ============================================================================
// CORS Configuration Handlers
// ============================================================================

/// GET /{bucket}?cors - Get bucket CORS configuration
pub async fn get_bucket_cors(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetBucketCors bucket={} request_id={}", bucket, request_id);

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

    // Get CORS configuration from metadata
    match state.metadata.get_bucket_cors(&bucket).await {
        Ok(Some(cors_xml)) => {
            info!("GetBucketCors success bucket={}", bucket);
            success_response_xml(StatusCode::OK, cors_xml, &request_id)
        }
        Ok(None) => {
            // No CORS configuration - return error per S3 spec
            cors_error_response(
                "NoSuchCORSConfiguration",
                "The CORS configuration does not exist",
                &request_id,
            )
        }
        Err(e) => {
            error!("Error getting CORS config: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// PUT /{bucket}?cors - Set bucket CORS configuration
pub async fn put_bucket_cors(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutBucketCors bucket={} request_id={}", bucket, request_id);

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

    // Parse and validate CORS configuration
    let config = match CorsConfiguration::from_xml(xml_str) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to parse CORS config: {}", e);
            return error_response(Error::MalformedXML(e), &request_id);
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        error!("Invalid CORS configuration: {}", e);
        return error_response(
            Error::MalformedXML(format!("Invalid CORS configuration: {}", e)),
            &request_id,
        );
    }

    // Serialize back to clean XML
    let clean_xml = match config.to_xml() {
        Ok(xml) => xml,
        Err(e) => {
            error!("Failed to serialize CORS config: {}", e);
            return error_response(Error::InternalError(e), &request_id);
        }
    };

    // Store in metadata
    match state.metadata.put_bucket_cors(&bucket, &clean_xml).await {
        Ok(_) => {
            info!("PutBucketCors success bucket={} rules={}", bucket, config.cors_rules.len());
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error storing CORS config: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// DELETE /{bucket}?cors - Delete bucket CORS configuration
pub async fn delete_bucket_cors(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("DeleteBucketCors bucket={} request_id={}", bucket, request_id);

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

    // Delete CORS configuration
    match state.metadata.delete_bucket_cors(&bucket).await {
        Ok(_) => {
            info!("DeleteBucketCors success bucket={}", bucket);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error deleting CORS config: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============================================================================
// CORS Preflight Handler
// ============================================================================

/// OPTIONS /{bucket}/{key?} - Handle CORS preflight requests
/// 
/// This handler responds to preflight OPTIONS requests from browsers.
/// It checks the Origin header against the bucket's CORS configuration
/// and returns appropriate Access-Control-* headers.
pub async fn handle_cors_preflight(
    State(state): State<AppState>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    
    // Extract bucket from path (first segment)
    let bucket = path.split('/').next().unwrap_or(&path);
    
    debug!(
        "CORS preflight request bucket={} path={} request_id={}",
        bucket, path, request_id
    );

    // Get Origin header (required for CORS)
    let origin = match headers.get(header::ORIGIN) {
        Some(o) => match o.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(Body::empty())
                    .unwrap();
            }
        },
        None => {
            // No Origin header - not a CORS request
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap();
        }
    };

    // Get Access-Control-Request-Method header
    let request_method = headers
        .get(header::ACCESS_CONTROL_REQUEST_METHOD)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("GET");

    // Get Access-Control-Request-Headers header
    let request_headers = headers
        .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
        .and_then(|h| h.to_str().ok());

    // Get bucket's CORS configuration
    let cors_config = match state.metadata.get_bucket_cors(bucket).await {
        Ok(Some(xml)) => match CorsConfiguration::from_xml(&xml) {
            Ok(config) => config,
            Err(e) => {
                warn!("Failed to parse CORS config for bucket {}: {}", bucket, e);
                return cors_forbidden_response(&origin, &request_id);
            }
        },
        Ok(None) => {
            // No CORS configuration - deny cross-origin access
            return cors_forbidden_response(&origin, &request_id);
        }
        Err(e) => {
            error!("Error getting CORS config: {}", e);
            return cors_forbidden_response(&origin, &request_id);
        }
    };

    // Find matching CORS rule
    match cors_config.find_matching_rule(&origin, request_method) {
        Some(rule) => {
            // Check if requested headers are allowed
            if let Some(req_headers) = request_headers {
                for header in req_headers.split(',').map(|h| h.trim()) {
                    if !rule.is_header_allowed(header) {
                        warn!(
                            "CORS header not allowed: {} for origin {} on bucket {}",
                            header, origin, bucket
                        );
                        return cors_forbidden_response(&origin, &request_id);
                    }
                }
            }

            // Build CORS response headers
            let cors_headers = CorsResponseHeaders::for_preflight(rule, &origin, request_headers);
            
            info!(
                "CORS preflight allowed: origin={} method={} bucket={}",
                origin, request_method, bucket
            );

            build_cors_response(StatusCode::OK, cors_headers, &request_id)
        }
        None => {
            warn!(
                "No matching CORS rule for origin={} method={} bucket={}",
                origin, request_method, bucket
            );
            cors_forbidden_response(&origin, &request_id)
        }
    }
}

/// Build CORS response with headers
fn build_cors_response(
    status: StatusCode,
    cors_headers: CorsResponseHeaders,
    request_id: &str,
) -> Response {
    let mut builder = Response::builder()
        .status(status)
        .header("x-amz-request-id", request_id);

    for (name, value) in cors_headers.to_header_vec() {
        builder = builder.header(&name, value);
    }

    builder.body(Body::empty()).unwrap()
}

/// Build CORS forbidden response
fn cors_forbidden_response(origin: &str, request_id: &str) -> Response {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .header("x-amz-request-id", request_id)
        .header("Vary", "Origin, Access-Control-Request-Method, Access-Control-Request-Headers")
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// CORS Middleware Helper
// ============================================================================

/// Add CORS headers to a response based on request origin
/// 
/// This should be called for actual (non-preflight) requests to add
/// the appropriate CORS headers to the response.
pub async fn add_cors_headers_to_response(
    state: &AppState,
    bucket: &str,
    origin: &str,
    method: &str,
    mut response: Response,
) -> Response {
    // Get bucket's CORS configuration
    let cors_config = match state.metadata.get_bucket_cors(bucket).await {
        Ok(Some(xml)) => match CorsConfiguration::from_xml(&xml) {
            Ok(config) => config,
            Err(_) => return response,
        },
        _ => return response,
    };

    // Find matching rule
    if let Some(rule) = cors_config.find_matching_rule(origin, method) {
        let cors_headers = CorsResponseHeaders::for_actual_request(rule, origin);
        
        let headers = response.headers_mut();
        for (name, value) in cors_headers.to_header_vec() {
            if let Ok(header_name) = name.parse::<header::HeaderName>() {
                if let Ok(header_value) = value.parse::<header::HeaderValue>() {
                    headers.insert(header_name, header_value);
                }
            }
        }
    }

    response
}

/// Check if origin is allowed for a bucket
pub async fn is_origin_allowed(
    state: &AppState,
    bucket: &str,
    origin: &str,
    method: &str,
) -> bool {
    match state.metadata.get_bucket_cors(bucket).await {
        Ok(Some(xml)) => match CorsConfiguration::from_xml(&xml) {
            Ok(config) => config.find_matching_rule(origin, method).is_some(),
            Err(_) => false,
        },
        _ => false,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_xml_parsing() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CORSConfiguration>
    <CORSRule>
        <AllowedOrigin>https://example.com</AllowedOrigin>
        <AllowedMethod>GET</AllowedMethod>
        <AllowedMethod>PUT</AllowedMethod>
        <AllowedHeader>*</AllowedHeader>
        <MaxAgeSeconds>3600</MaxAgeSeconds>
    </CORSRule>
</CORSConfiguration>"#;

        let config = CorsConfiguration::from_xml(xml).unwrap();
        assert_eq!(config.cors_rules.len(), 1);
        assert_eq!(config.cors_rules[0].allowed_origins, vec!["https://example.com"]);
        assert_eq!(config.cors_rules[0].allowed_methods.len(), 2);
    }

    #[test]
    fn test_cors_rule_matching() {
        let config = CorsConfiguration {
            cors_rules: vec![
                hafiz_core::types::CorsRule {
                    id: Some("rule1".to_string()),
                    allowed_origins: vec!["https://example.com".to_string()],
                    allowed_methods: vec![
                        hafiz_core::types::CorsMethod::GET,
                        hafiz_core::types::CorsMethod::PUT,
                    ],
                    allowed_headers: vec!["*".to_string()],
                    expose_headers: vec![],
                    max_age_seconds: Some(3600),
                },
            ],
        };

        assert!(config.find_matching_rule("https://example.com", "GET").is_some());
        assert!(config.find_matching_rule("https://example.com", "PUT").is_some());
        assert!(config.find_matching_rule("https://example.com", "DELETE").is_none());
        assert!(config.find_matching_rule("https://other.com", "GET").is_none());
    }
}
