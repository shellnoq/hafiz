//! S3 API Routes

mod cors;
mod notification;
mod object_lock;
mod policy;

pub use cors::{add_cors_headers_to_response, handle_cors_preflight, is_origin_allowed};
pub use object_lock::{can_delete_object, get_lock_error_message};

use axum::{
    body::Body,
    extract::{Path, Query, RawQuery, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use hafiz_core::{
    types::{Bucket, ByteRange, ListObjectsResult, Object},
    utils::{format_http_datetime, format_s3_datetime, generate_etag, generate_request_id},
    Error,
};
use serde::Deserialize;
use std::collections::BTreeMap;
use tracing::{debug, error, info};

use crate::server::AppState;
use crate::xml;

/// Error response wrapper
pub struct S3Response(pub Response);

impl IntoResponse for S3Response {
    fn into_response(self) -> Response {
        self.0
    }
}

fn error_response(err: Error, request_id: &str) -> Response {
    let status =
        StatusCode::from_u16(err.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
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

// ============= Handler Dispatchers =============

/// Generic query params for dispatching
#[derive(Debug, Deserialize, Default)]
pub struct DispatchQuery {
    uploads: Option<String>,
    #[serde(rename = "uploadId")]
    upload_id: Option<String>,
    #[serde(rename = "partNumber")]
    part_number: Option<i32>,
    delete: Option<String>,
}

/// Bucket GET dispatcher - ListObjects, ListMultipartUploads, GetBucketVersioning, GetBucketLifecycle, ListObjectVersions, GetBucketPolicy, GetBucketAcl, or GetBucketNotification
pub async fn bucket_get_handler(
    state: State<AppState>,
    path: Path<String>,
    raw_query: RawQuery,
    query: Query<ListObjectsQuery>,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    // Check if this is a get bucket versioning request
    if query_str == "versioning" || query_str.starts_with("versioning&") {
        return get_bucket_versioning(state, path).await.into_response();
    }

    // Check if this is a get bucket lifecycle request
    if query_str == "lifecycle" || query_str.starts_with("lifecycle&") {
        return get_bucket_lifecycle(state, path).await.into_response();
    }

    // Check if this is a get bucket policy request
    if query_str == "policy" || query_str.starts_with("policy&") {
        return policy::get_bucket_policy(state, path).await.into_response();
    }

    // Check if this is a get bucket ACL request
    if query_str == "acl" || query_str.starts_with("acl&") {
        return policy::get_bucket_acl(state, path).await.into_response();
    }

    // Check if this is a get bucket notification request
    if query_str == "notification" || query_str.starts_with("notification&") {
        return notification::get_bucket_notification(state, path)
            .await
            .into_response();
    }

    // Check if this is a get bucket CORS request
    if query_str == "cors" || query_str.starts_with("cors&") {
        return cors::get_bucket_cors(state, path).await.into_response();
    }

    // Check if this is a get bucket Object Lock request
    if query_str == "object-lock" || query_str.starts_with("object-lock&") {
        return object_lock::get_bucket_object_lock_config(state, path)
            .await
            .into_response();
    }

    // Check if this is a list object versions request
    if query_str.contains("versions") {
        let params: ListObjectVersionsQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return list_object_versions(state, path, Query(params))
            .await
            .into_response();
    }

    // Check if this is a list multipart uploads request
    if query_str.contains("uploads") && !query_str.contains("uploadId") {
        // Parse as ListMultipartUploadsQuery
        let params: ListMultipartUploadsQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return list_multipart_uploads(state, path, Query(params))
            .await
            .into_response();
    }

    // Default: ListObjects
    get_bucket(state, path, query).await.into_response()
}

/// Bucket PUT dispatcher - CreateBucket, PutBucketVersioning, PutBucketLifecycle, PutBucketPolicy, PutBucketAcl, or PutBucketNotification
pub async fn bucket_put_handler(
    state: State<AppState>,
    path: Path<String>,
    headers: HeaderMap,
    raw_query: RawQuery,
    body: Bytes,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    // Check if this is a put bucket versioning request
    if query_str == "versioning" || query_str.starts_with("versioning&") {
        return put_bucket_versioning(state, path, body)
            .await
            .into_response();
    }

    // Check if this is a put bucket lifecycle request
    if query_str == "lifecycle" || query_str.starts_with("lifecycle&") {
        return put_bucket_lifecycle(state, path, body)
            .await
            .into_response();
    }

    // Check if this is a put bucket policy request
    if query_str == "policy" || query_str.starts_with("policy&") {
        return policy::put_bucket_policy(state, path, body)
            .await
            .into_response();
    }

    // Check if this is a put bucket ACL request
    if query_str == "acl" || query_str.starts_with("acl&") {
        return policy::put_bucket_acl(state, path, headers, body)
            .await
            .into_response();
    }

    // Check if this is a put bucket notification request
    if query_str == "notification" || query_str.starts_with("notification&") {
        return notification::put_bucket_notification(state, path, body)
            .await
            .into_response();
    }

    // Check if this is a put bucket CORS request
    if query_str == "cors" || query_str.starts_with("cors&") {
        return cors::put_bucket_cors(state, path, body)
            .await
            .into_response();
    }

    // Check if this is a put bucket Object Lock request
    if query_str == "object-lock" || query_str.starts_with("object-lock&") {
        return object_lock::put_bucket_object_lock_config(state, path, body)
            .await
            .into_response();
    }

    // Default: CreateBucket
    create_bucket(state, path).await.into_response()
}

/// Bucket DELETE dispatcher - DeleteBucket, DeleteBucketLifecycle, or DeleteBucketPolicy
pub async fn bucket_delete_handler(
    state: State<AppState>,
    path: Path<String>,
    raw_query: RawQuery,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    // Check if this is a delete bucket lifecycle request
    if query_str == "lifecycle" || query_str.starts_with("lifecycle&") {
        return delete_bucket_lifecycle(state, path).await.into_response();
    }

    // Check if this is a delete bucket policy request
    if query_str == "policy" || query_str.starts_with("policy&") {
        return policy::delete_bucket_policy(state, path)
            .await
            .into_response();
    }

    // Check if this is a delete bucket CORS request
    if query_str == "cors" || query_str.starts_with("cors&") {
        return cors::delete_bucket_cors(state, path).await.into_response();
    }

    // Default: DeleteBucket
    delete_bucket(state, path).await.into_response()
}

/// Bucket POST dispatcher - DeleteObjects
pub async fn bucket_post_handler(
    state: State<AppState>,
    path: Path<String>,
    raw_query: RawQuery,
    body: Bytes,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    if query_str.contains("delete") {
        let params: DeleteObjectsQuery = serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return delete_objects(state, path, Query(params), body)
            .await
            .into_response();
    }

    // Unknown POST operation
    let request_id = generate_request_id();
    error_response(
        Error::InvalidRequest("Unknown bucket POST operation".into()),
        &request_id,
    )
}

/// Object GET dispatcher - GetObject, ListParts, GetObjectTagging, or GetObjectAcl
pub async fn object_get_handler(
    state: State<AppState>,
    path: Path<(String, String)>,
    headers: HeaderMap,
    raw_query: RawQuery,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    // Check if this is a get object tagging request
    if query_str == "tagging" || query_str.starts_with("tagging&") || query_str.contains("&tagging")
    {
        let version_id: Option<String> =
            serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(&query_str)
                .ok()
                .and_then(|m| m.get("versionId").cloned());
        return get_object_tagging(state, path, version_id)
            .await
            .into_response();
    }

    // Check if this is a get object ACL request
    if query_str == "acl" || query_str.starts_with("acl&") || query_str.contains("&acl") {
        let version_id: Option<String> =
            serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(&query_str)
                .ok()
                .and_then(|m| m.get("versionId").cloned());
        return policy::get_object_acl(state, path, version_id)
            .await
            .into_response();
    }

    // Check if this is a get object retention request
    if query_str == "retention"
        || query_str.starts_with("retention&")
        || query_str.contains("&retention")
    {
        let query: object_lock::RetentionQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return object_lock::get_object_retention(state, path, Query(query))
            .await
            .into_response();
    }

    // Check if this is a get object legal hold request
    if query_str == "legal-hold"
        || query_str.starts_with("legal-hold&")
        || query_str.contains("&legal-hold")
    {
        let query: object_lock::RetentionQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return object_lock::get_object_legal_hold(state, path, Query(query))
            .await
            .into_response();
    }

    // Check if this is a list parts request
    if query_str.contains("uploadId") && !query_str.contains("partNumber") {
        let params: ListPartsQuery = serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return list_parts(state, path, Query(params)).await.into_response();
    }

    // Check for versionId query param
    let version_id: Option<String> =
        serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(&query_str)
            .ok()
            .and_then(|m| m.get("versionId").cloned());

    // Default: GetObject (with optional version)
    get_object_versioned(state, path, headers, version_id)
        .await
        .into_response()
}

/// Object PUT dispatcher - PutObject, CopyObject, UploadPart, PutObjectTagging, or PutObjectAcl
pub async fn object_put_handler(
    state: State<AppState>,
    path: Path<(String, String)>,
    headers: HeaderMap,
    raw_query: RawQuery,
    body: Bytes,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    // Check if this is a put object tagging request
    if query_str == "tagging" || query_str.starts_with("tagging&") || query_str.contains("&tagging")
    {
        let version_id: Option<String> =
            serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(&query_str)
                .ok()
                .and_then(|m| m.get("versionId").cloned());
        return put_object_tagging(state, path, version_id, body)
            .await
            .into_response();
    }

    // Check if this is a put object ACL request
    if query_str == "acl" || query_str.starts_with("acl&") || query_str.contains("&acl") {
        let version_id: Option<String> =
            serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(&query_str)
                .ok()
                .and_then(|m| m.get("versionId").cloned());
        return policy::put_object_acl(state, path, headers.clone(), version_id, body)
            .await
            .into_response();
    }

    // Check if this is a put object retention request
    if query_str == "retention"
        || query_str.starts_with("retention&")
        || query_str.contains("&retention")
    {
        let query: object_lock::RetentionQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return object_lock::put_object_retention(state, path, headers, Query(query), body)
            .await
            .into_response();
    }

    // Check if this is a put object legal hold request
    if query_str == "legal-hold"
        || query_str.starts_with("legal-hold&")
        || query_str.contains("&legal-hold")
    {
        let query: object_lock::RetentionQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return object_lock::put_object_legal_hold(state, path, Query(query), body)
            .await
            .into_response();
    }

    // Check if this is an upload part request
    if query_str.contains("uploadId") && query_str.contains("partNumber") {
        let params: UploadPartQuery = serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return upload_part(state, path, Query(params), body)
            .await
            .into_response();
    }

    // Check if this is a copy request
    if headers.contains_key("x-amz-copy-source") {
        return copy_object(state, path, headers).await.into_response();
    }

    // Default: PutObject
    put_object(state, path, headers, body).await.into_response()
}

/// Object DELETE dispatcher - DeleteObject, AbortMultipartUpload, or DeleteObjectTagging
pub async fn object_delete_handler(
    state: State<AppState>,
    path: Path<(String, String)>,
    raw_query: RawQuery,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    // Check if this is a delete object tagging request
    if query_str == "tagging" || query_str.starts_with("tagging&") || query_str.contains("&tagging")
    {
        let version_id: Option<String> =
            serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(&query_str)
                .ok()
                .and_then(|m| m.get("versionId").cloned());
        return delete_object_tagging(state, path, version_id)
            .await
            .into_response();
    }

    // Check if this is an abort multipart upload request
    if query_str.contains("uploadId") && !query_str.contains("versionId") {
        let params: AbortMultipartQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return abort_multipart_upload(state, path, Query(params))
            .await
            .into_response();
    }

    // Check for versionId query param
    let version_id: Option<String> =
        serde_urlencoded::from_str::<std::collections::HashMap<String, String>>(&query_str)
            .ok()
            .and_then(|m| m.get("versionId").cloned());

    // Default: DeleteObject (with optional version)
    delete_object_versioned(state, path, version_id)
        .await
        .into_response()
}

/// Object POST dispatcher - CreateMultipartUpload or CompleteMultipartUpload
pub async fn object_post_handler(
    state: State<AppState>,
    path: Path<(String, String)>,
    headers: HeaderMap,
    raw_query: RawQuery,
    body: Bytes,
) -> impl IntoResponse {
    let query_str = raw_query.0.unwrap_or_default();

    // Check if this is a complete multipart upload request
    if query_str.contains("uploadId") {
        let params: CompleteMultipartQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return complete_multipart_upload(state, path, Query(params), body)
            .await
            .into_response();
    }

    // Check if this is a create multipart upload request
    if query_str.contains("uploads") {
        let params: CreateMultipartQuery =
            serde_urlencoded::from_str(&query_str).unwrap_or_default();
        return create_multipart_upload(state, path, headers, Query(params))
            .await
            .into_response();
    }

    // Unknown POST operation
    let request_id = generate_request_id();
    error_response(
        Error::InvalidRequest("Unknown object POST operation".into()),
        &request_id,
    )
}

// ============= Service Operations =============

/// List all buckets
pub async fn list_buckets(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("ListBuckets request_id={}", request_id);

    // Get user from auth (simplified - using root for now)
    let owner_id = "root";

    match state.metadata.list_buckets(owner_id).await {
        Ok(buckets) => {
            let xml = xml::list_buckets_response(&buckets, owner_id);
            success_response(StatusCode::OK, xml, &request_id)
        }
        Err(e) => {
            error!("ListBuckets error: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============= Bucket Operations =============

#[derive(Debug, Deserialize)]
pub struct ListObjectsQuery {
    #[serde(rename = "list-type")]
    list_type: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    #[serde(rename = "max-keys")]
    max_keys: Option<i32>,
    #[serde(rename = "continuation-token")]
    continuation_token: Option<String>,
    marker: Option<String>,
}

/// HEAD bucket - check if bucket exists
pub async fn head_bucket(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("HeadBucket bucket={} request_id={}", bucket, request_id);

    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => Response::builder()
            .status(StatusCode::OK)
            .header("x-amz-request-id", &request_id)
            .body(Body::empty())
            .unwrap(),
        Ok(None) => error_response(Error::NoSuchBucket, &request_id),
        Err(e) => error_response(e, &request_id),
    }
}

/// GET bucket - list objects or get bucket info
pub async fn get_bucket(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(params): Query<ListObjectsQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "GetBucket/ListObjects bucket={} request_id={}",
        bucket, request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    let max_keys = params.max_keys.unwrap_or(1000).min(1000);
    let continuation = params
        .continuation_token
        .as_deref()
        .or(params.marker.as_deref());
    let is_v2 = params.list_type.as_deref() == Some("2");

    match state
        .metadata
        .list_objects(
            &bucket,
            params.prefix.as_deref(),
            params.delimiter.as_deref(),
            max_keys,
            continuation,
        )
        .await
    {
        Ok((objects, common_prefixes, is_truncated, next_token)) => {
            let result = ListObjectsResult {
                name: bucket,
                prefix: params.prefix,
                delimiter: params.delimiter,
                max_keys,
                is_truncated,
                contents: objects,
                common_prefixes,
                continuation_token: params.continuation_token,
                next_continuation_token: next_token,
            };

            let xml = if is_v2 {
                xml::list_objects_v2_response(&result)
            } else {
                xml::list_objects_response(&result)
            };

            success_response(StatusCode::OK, xml, &request_id)
        }
        Err(e) => {
            error!("ListObjects error: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// PUT bucket - create bucket
pub async fn create_bucket(
    State(state): State<AppState>,
    Path(bucket_name): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "CreateBucket bucket={} request_id={}",
        bucket_name, request_id
    );

    // Validate bucket name
    if let Err(e) = Bucket::validate_name(&bucket_name) {
        return error_response(e, &request_id);
    }

    let bucket = Bucket::new(bucket_name.clone(), "root".to_string());

    // Create in metadata
    if let Err(e) = state.metadata.create_bucket(&bucket).await {
        return error_response(e, &request_id);
    }

    // Create storage directory
    if let Err(e) = state.storage.create_bucket(&bucket_name).await {
        error!("Failed to create bucket storage: {}", e);
        // Rollback metadata
        let _ = state.metadata.delete_bucket(&bucket_name).await;
        return error_response(e, &request_id);
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Location", format!("/{}", bucket_name))
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}

/// DELETE bucket
pub async fn delete_bucket(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!("DeleteBucket bucket={} request_id={}", bucket, request_id);

    // Delete from metadata (will check if empty)
    if let Err(e) = state.metadata.delete_bucket(&bucket).await {
        return error_response(e, &request_id);
    }

    // Delete storage
    if let Err(e) = state.storage.delete_bucket(&bucket).await {
        error!("Failed to delete bucket storage: {}", e);
    }

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}

// ============= Object Operations =============

/// HEAD object
pub async fn head_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "HeadObject bucket={} key={} request_id={}",
        bucket, key, request_id
    );

    match state.metadata.get_object(&bucket, &key).await {
        Ok(Some(obj)) => Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", &obj.content_type)
            .header("Content-Length", obj.size.to_string())
            .header("ETag", generate_etag(&obj.etag))
            .header("Last-Modified", format_http_datetime(&obj.last_modified))
            .header("x-amz-request-id", &request_id)
            .body(Body::empty())
            .unwrap(),
        Ok(None) => error_response(Error::NoSuchKey, &request_id),
        Err(e) => error_response(e, &request_id),
    }
}

/// GET object
pub async fn get_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "GetObject bucket={} key={} request_id={}",
        bucket, key, request_id
    );

    // Get metadata
    let obj = match state.metadata.get_object(&bucket, &key).await {
        Ok(Some(obj)) => obj,
        Ok(None) => return error_response(Error::NoSuchKey, &request_id),
        Err(e) => return error_response(e, &request_id),
    };

    // Check for range request
    let range_header = headers.get("range").and_then(|v| v.to_str().ok());

    let (data, status, content_range) = if let Some(range_str) = range_header {
        match ByteRange::parse(range_str) {
            Ok(range) => match range.resolve(obj.size) {
                Ok((start, end)) => {
                    match state.storage.get_range(&bucket, &key, start, end).await {
                        Ok(data) => {
                            let content_range = format!("bytes {}-{}/{}", start, end, obj.size);
                            (data, StatusCode::PARTIAL_CONTENT, Some(content_range))
                        }
                        Err(e) => return error_response(e, &request_id),
                    }
                }
                Err(e) => return error_response(e, &request_id),
            },
            Err(e) => return error_response(e, &request_id),
        }
    } else {
        match state.storage.get(&bucket, &key).await {
            Ok(data) => (data, StatusCode::OK, None),
            Err(e) => return error_response(e, &request_id),
        }
    };

    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", &obj.content_type)
        .header("Content-Length", data.len().to_string())
        .header("ETag", generate_etag(&obj.etag))
        .header("Last-Modified", format_http_datetime(&obj.last_modified))
        .header("Accept-Ranges", "bytes")
        .header("x-amz-request-id", &request_id);

    if let Some(range) = content_range {
        builder = builder.header("Content-Range", range);
    }

    builder.body(Body::from(data)).unwrap()
}

/// PUT object
pub async fn put_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "PutObject bucket={} key={} size={} request_id={}",
        bucket,
        key,
        body.len(),
        request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    // Validate key
    if let Err(e) = Object::validate_key(&key) {
        return error_response(e, &request_id);
    }

    // Get content type
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| {
            mime_guess::from_path(&key)
                .first_or_octet_stream()
                .to_string()
        });

    // Check for SSE headers
    let sse_header = headers
        .get("x-amz-server-side-encryption")
        .and_then(|v| v.to_str().ok());

    let sse_c_key = headers
        .get("x-amz-server-side-encryption-customer-key")
        .and_then(|v| v.to_str().ok());

    let sse_c_key_md5 = headers
        .get("x-amz-server-side-encryption-customer-key-md5")
        .and_then(|v| v.to_str().ok());

    // Determine encryption type
    let encryption_type = if sse_c_key.is_some() {
        hafiz_core::types::EncryptionType::SseC
    } else if sse_header == Some("AES256") || sse_header == Some("aws:kms") {
        hafiz_core::types::EncryptionType::SseS3
    } else {
        hafiz_core::types::EncryptionType::None
    };

    // Build encryption info (actual encryption handled by storage layer)
    let encryption = hafiz_core::types::EncryptionInfo {
        encryption_type,
        encrypted_dek: None,
        dek_nonce: None,
        data_nonce: None,
        sse_customer_key_md5: sse_c_key_md5.map(String::from),
    };

    // Store data
    let etag = match state.storage.put(&bucket, &key, body.clone()).await {
        Ok(etag) => etag,
        Err(e) => return error_response(e, &request_id),
    };

    // Store metadata
    let object = Object::new(
        bucket.clone(),
        key.clone(),
        body.len() as i64,
        etag.clone(),
        content_type,
    )
    .with_encryption(encryption.clone());

    if let Err(e) = state.metadata.put_object(&object).await {
        // Rollback storage
        let _ = state.storage.delete(&bucket, &key).await;
        return error_response(e, &request_id);
    }

    // Build response with SSE headers
    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("ETag", generate_etag(&etag))
        .header("x-amz-request-id", &request_id);

    // Add SSE response headers
    if encryption.encryption_type != hafiz_core::types::EncryptionType::None {
        builder = builder.header(
            "x-amz-server-side-encryption",
            encryption.encryption_type.as_str(),
        );
    }
    if let Some(ref md5) = encryption.sse_customer_key_md5 {
        builder = builder.header("x-amz-server-side-encryption-customer-key-MD5", md5);
    }

    builder.body(Body::empty()).unwrap()
}

/// DELETE object
pub async fn delete_object(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "DeleteObject bucket={} key={} request_id={}",
        bucket, key, request_id
    );

    // Delete from storage
    if let Err(e) = state.storage.delete(&bucket, &key).await {
        error!("Failed to delete object storage: {}", e);
    }

    // Delete from metadata
    if let Err(e) = state.metadata.delete_object(&bucket, &key).await {
        return error_response(e, &request_id);
    }

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}

// ============= Phase 2: Advanced Operations =============

/// COPY object (PUT with x-amz-copy-source header)
pub async fn copy_object(
    State(state): State<AppState>,
    Path((dest_bucket, dest_key)): Path<(String, String)>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let request_id = generate_request_id();

    // Get copy source header
    let copy_source = match headers.get("x-amz-copy-source") {
        Some(v) => v.to_str().unwrap_or(""),
        None => {
            return error_response(
                Error::InvalidRequest("Missing x-amz-copy-source header".into()),
                &request_id,
            )
        }
    };

    info!(
        "CopyObject source={} dest={}/{} request_id={}",
        copy_source, dest_bucket, dest_key, request_id
    );

    // Parse source: /bucket/key or bucket/key
    let source = copy_source.trim_start_matches('/');
    let parts: Vec<&str> = source.splitn(2, '/').collect();
    if parts.len() != 2 {
        return error_response(
            Error::InvalidRequest("Invalid copy source format".into()),
            &request_id,
        );
    }
    let (src_bucket, src_key) = (parts[0], parts[1]);

    // URL decode the key
    let src_key = urlencoding::decode(src_key)
        .unwrap_or_else(|_| src_key.into())
        .to_string();

    // Check destination bucket exists
    match state.metadata.get_bucket(&dest_bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    // Get source object metadata
    let src_object = match state.metadata.get_object(src_bucket, &src_key).await {
        Ok(Some(obj)) => obj,
        Ok(None) => return error_response(Error::NoSuchKey, &request_id),
        Err(e) => return error_response(e, &request_id),
    };

    // Read source data
    let data = match state.storage.get(src_bucket, &src_key).await {
        Ok(data) => data,
        Err(e) => return error_response(e, &request_id),
    };

    // Check metadata directive
    let metadata_directive = headers
        .get("x-amz-metadata-directive")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("COPY");

    let (content_type, metadata) = if metadata_directive == "REPLACE" {
        // Use new metadata from headers
        let ct = headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| src_object.content_type.clone());
        (ct, extract_user_metadata(&headers))
    } else {
        // Copy metadata from source
        (src_object.content_type.clone(), src_object.metadata.clone())
    };

    // Store to destination
    let etag = match state
        .storage
        .put(&dest_bucket, &dest_key, data.clone())
        .await
    {
        Ok(etag) => etag,
        Err(e) => return error_response(e, &request_id),
    };

    // Create destination object metadata
    let mut dest_object = Object::new(
        dest_bucket.clone(),
        dest_key.clone(),
        data.len() as i64,
        etag.clone(),
        content_type,
    );
    dest_object.metadata = metadata;

    if let Err(e) = state.metadata.put_object(&dest_object).await {
        let _ = state.storage.delete(&dest_bucket, &dest_key).await;
        return error_response(e, &request_id);
    }

    let xml = xml::copy_object_response(&etag, &dest_object.last_modified);
    success_response(StatusCode::OK, xml, &request_id)
}

/// Extract user metadata from headers (x-amz-meta-*)
fn extract_user_metadata(headers: &HeaderMap) -> std::collections::HashMap<String, String> {
    let mut metadata = std::collections::HashMap::new();
    for (name, value) in headers.iter() {
        let name_str = name.as_str().to_lowercase();
        if name_str.starts_with("x-amz-meta-") {
            if let Ok(v) = value.to_str() {
                let key = name_str.strip_prefix("x-amz-meta-").unwrap().to_string();
                metadata.insert(key, v.to_string());
            }
        }
    }
    metadata
}

/// DELETE multiple objects (POST /?delete)
#[derive(Debug, Deserialize, Debug, Deserialize, Default)]
pub struct DeleteObjectsQuery {
    delete: Option<String>,
}

pub async fn delete_objects(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(_params): Query<DeleteObjectsQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!("DeleteObjects bucket={} request_id={}", bucket, request_id);

    // Parse XML body
    let delete_request = match xml::parse_delete_objects(&body) {
        Ok(req) => req,
        Err(e) => return error_response(Error::MalformedXML(e.to_string()), &request_id),
    };

    let quiet = delete_request.quiet.unwrap_or(false);
    let mut deleted = Vec::new();
    let mut errors = Vec::new();

    for obj in delete_request.objects {
        let key = obj.key;
        let version_id = obj.version_id;

        match state.storage.delete(&bucket, &key).await {
            Ok(_) => {
                if let Err(e) = state.metadata.delete_object(&bucket, &key).await {
                    errors.push(xml::DeleteError {
                        key: key.clone(),
                        version_id: version_id.clone(),
                        code: e.code().to_string(),
                        message: e.to_string(),
                    });
                } else if !quiet {
                    deleted.push(xml::DeletedObject {
                        key,
                        version_id,
                        delete_marker: false,
                        delete_marker_version_id: None,
                    });
                }
            }
            Err(e) => {
                errors.push(xml::DeleteError {
                    key,
                    version_id,
                    code: e.code().to_string(),
                    message: e.to_string(),
                });
            }
        }
    }

    let xml = xml::delete_objects_response(&deleted, &errors);
    success_response(StatusCode::OK, xml, &request_id)
}

// ============= Multipart Upload Operations =============

#[derive(Debug, Deserialize, Default)]
pub struct CreateMultipartQuery {
    uploads: Option<String>,
}

/// Initiate multipart upload (POST /bucket/key?uploads)
pub async fn create_multipart_upload(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
    Query(_params): Query<CreateMultipartQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "CreateMultipartUpload bucket={} key={} request_id={}",
        bucket, key, request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    // Validate key
    if let Err(e) = Object::validate_key(&key) {
        return error_response(e, &request_id);
    }

    // Get content type
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(String::from)
        .unwrap_or_else(|| {
            mime_guess::from_path(&key)
                .first_or_octet_stream()
                .to_string()
        });

    // Extract user metadata
    let metadata = extract_user_metadata(&headers);

    // Create multipart upload
    match state
        .metadata
        .create_multipart_upload(&bucket, &key, &content_type, &metadata)
        .await
    {
        Ok(upload_id) => {
            let xml = xml::initiate_multipart_upload_response(&bucket, &key, &upload_id);
            success_response(StatusCode::OK, xml, &request_id)
        }
        Err(e) => error_response(e, &request_id),
    }
}

#[derive(Debug, Deserialize, Debug, Deserialize, Default)]
pub struct UploadPartQuery {
    #[serde(rename = "uploadId", default)]
    upload_id: String,
    #[serde(rename = "partNumber", default)]
    part_number: i32,
}

/// Upload part (PUT /bucket/key?uploadId=xxx&partNumber=n)
pub async fn upload_part(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<UploadPartQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "UploadPart bucket={} key={} uploadId={} partNumber={} size={} request_id={}",
        bucket,
        key,
        params.upload_id,
        params.part_number,
        body.len(),
        request_id
    );

    // Validate part number (1-10000)
    if params.part_number < 1 || params.part_number > 10000 {
        return error_response(
            Error::InvalidArgument("Part number must be between 1 and 10000".into()),
            &request_id,
        );
    }

    // Verify upload exists
    match state
        .metadata
        .get_multipart_upload(&bucket, &key, &params.upload_id)
        .await
    {
        Ok(None) => return error_response(Error::NoSuchUpload, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    // Store part data
    let part_key = format!("{}/.parts/{}/{}", key, params.upload_id, params.part_number);
    let etag = match state.storage.put(&bucket, &part_key, body.clone()).await {
        Ok(etag) => etag,
        Err(e) => return error_response(e, &request_id),
    };

    // Record part in metadata
    if let Err(e) = state
        .metadata
        .put_upload_part(
            &params.upload_id,
            params.part_number,
            body.len() as i64,
            &etag,
        )
        .await
    {
        let _ = state.storage.delete(&bucket, &part_key).await;
        return error_response(e, &request_id);
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("ETag", format!("\"{}\"", etag))
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}

#[derive(Debug, Deserialize, Default)]
pub struct CompleteMultipartQuery {
    #[serde(rename = "uploadId", default)]
    upload_id: String,
}

/// Complete multipart upload (POST /bucket/key?uploadId=xxx)
pub async fn complete_multipart_upload(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<CompleteMultipartQuery>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "CompleteMultipartUpload bucket={} key={} uploadId={} request_id={}",
        bucket, key, params.upload_id, request_id
    );

    // Parse completion XML
    let completion = match xml::parse_complete_multipart(&body) {
        Ok(c) => c,
        Err(e) => return error_response(Error::MalformedXML(e.to_string()), &request_id),
    };

    // Get upload info
    let upload = match state
        .metadata
        .get_multipart_upload(&bucket, &key, &params.upload_id)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return error_response(Error::NoSuchUpload, &request_id),
        Err(e) => return error_response(e, &request_id),
    };

    // Get all parts
    let parts = match state.metadata.list_upload_parts(&params.upload_id).await {
        Ok(p) => p,
        Err(e) => return error_response(e, &request_id),
    };

    // Validate parts match
    if completion.parts.len() != parts.len() {
        return error_response(
            Error::InvalidPart("Part count mismatch".into()),
            &request_id,
        );
    }

    // Concatenate all parts
    let mut final_data = Vec::new();
    let mut part_etags = Vec::new();

    for (i, completed_part) in completion.parts.iter().enumerate() {
        let stored_part = parts.get(i);

        match stored_part {
            Some(sp) if sp.part_number == completed_part.part_number => {
                // Read part data
                let part_key = format!(
                    "{}/.parts/{}/{}",
                    key, params.upload_id, completed_part.part_number
                );
                match state.storage.get(&bucket, &part_key).await {
                    Ok(data) => {
                        final_data.extend_from_slice(&data);
                        part_etags.push(sp.etag.clone());
                    }
                    Err(e) => return error_response(e, &request_id),
                }
            }
            _ => {
                return error_response(
                    Error::InvalidPart(format!(
                        "Invalid part number: {}",
                        completed_part.part_number
                    )),
                    &request_id,
                );
            }
        }
    }

    // Calculate final ETag (MD5 of concatenated part MD5s + "-" + part count)
    let final_etag = hafiz_crypto::multipart_etag(&part_etags, parts.len());

    // Store final object
    if let Err(e) = state
        .storage
        .put(&bucket, &key, Bytes::from(final_data.clone()))
        .await
    {
        return error_response(e, &request_id);
    }

    // Create object metadata
    let mut object = Object::new(
        bucket.clone(),
        key.clone(),
        final_data.len() as i64,
        final_etag.clone(),
        upload.content_type.clone(),
    );
    object.metadata = upload.metadata.clone();

    if let Err(e) = state.metadata.put_object(&object).await {
        let _ = state.storage.delete(&bucket, &key).await;
        return error_response(e, &request_id);
    }

    // Clean up parts
    for part in &parts {
        let part_key = format!("{}/.parts/{}/{}", key, params.upload_id, part.part_number);
        let _ = state.storage.delete(&bucket, &part_key).await;
    }

    // Delete upload record
    let _ = state
        .metadata
        .delete_multipart_upload(&params.upload_id)
        .await;

    let xml = xml::complete_multipart_upload_response(&bucket, &key, &final_etag);
    success_response(StatusCode::OK, xml, &request_id)
}

#[derive(Debug, Deserialize, Default)]
pub struct AbortMultipartQuery {
    #[serde(rename = "uploadId", default)]
    upload_id: String,
}

/// Abort multipart upload (DELETE /bucket/key?uploadId=xxx)
pub async fn abort_multipart_upload(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<AbortMultipartQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "AbortMultipartUpload bucket={} key={} uploadId={} request_id={}",
        bucket, key, params.upload_id, request_id
    );

    // Get all parts to clean up
    if let Ok(parts) = state.metadata.list_upload_parts(&params.upload_id).await {
        for part in parts {
            let part_key = format!("{}/.parts/{}/{}", key, params.upload_id, part.part_number);
            let _ = state.storage.delete(&bucket, &part_key).await;
        }
    }

    // Delete upload record
    if let Err(e) = state
        .metadata
        .delete_multipart_upload(&params.upload_id)
        .await
    {
        return error_response(e, &request_id);
    }

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}

#[derive(Debug, Deserialize, Default)]
pub struct ListPartsQuery {
    #[serde(rename = "uploadId", default)]
    upload_id: String,
    #[serde(rename = "max-parts")]
    max_parts: Option<i32>,
    #[serde(rename = "part-number-marker")]
    part_number_marker: Option<i32>,
}

/// List parts (GET /bucket/key?uploadId=xxx)
pub async fn list_parts(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(params): Query<ListPartsQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "ListParts bucket={} key={} uploadId={} request_id={}",
        bucket, key, params.upload_id, request_id
    );

    // Verify upload exists
    let upload = match state
        .metadata
        .get_multipart_upload(&bucket, &key, &params.upload_id)
        .await
    {
        Ok(Some(u)) => u,
        Ok(None) => return error_response(Error::NoSuchUpload, &request_id),
        Err(e) => return error_response(e, &request_id),
    };

    // Get parts
    let parts = match state.metadata.list_upload_parts(&params.upload_id).await {
        Ok(p) => p,
        Err(e) => return error_response(e, &request_id),
    };

    let max_parts = params.max_parts.unwrap_or(1000).min(1000);
    let marker = params.part_number_marker.unwrap_or(0);

    let filtered_parts: Vec<_> = parts
        .into_iter()
        .filter(|p| p.part_number > marker)
        .take(max_parts as usize)
        .collect();

    let is_truncated = filtered_parts.len() == max_parts as usize;
    let next_marker = filtered_parts.last().map(|p| p.part_number);

    // Convert to PartInfo for XML response
    let part_infos: Vec<xml::PartInfo> = filtered_parts
        .into_iter()
        .map(|p| xml::PartInfo {
            part_number: p.part_number,
            last_modified: p.last_modified,
            etag: p.etag,
            size: p.size,
        })
        .collect();

    let xml = xml::list_parts_response(
        &bucket,
        &key,
        &params.upload_id,
        &upload.initiator_id,
        &upload.storage_class,
        &part_infos,
        max_parts,
        is_truncated,
        next_marker,
    );

    success_response(StatusCode::OK, xml, &request_id)
}

#[derive(Debug, Deserialize, Default)]
pub struct ListMultipartUploadsQuery {
    uploads: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    #[serde(rename = "max-uploads")]
    max_uploads: Option<i32>,
    #[serde(rename = "key-marker")]
    key_marker: Option<String>,
    #[serde(rename = "upload-id-marker")]
    upload_id_marker: Option<String>,
}

/// List multipart uploads (GET /bucket?uploads)
pub async fn list_multipart_uploads(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(params): Query<ListMultipartUploadsQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "ListMultipartUploads bucket={} request_id={}",
        bucket, request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    let max_uploads = params.max_uploads.unwrap_or(1000).min(1000);

    match state
        .metadata
        .list_multipart_uploads(
            &bucket,
            params.prefix.as_deref(),
            params.key_marker.as_deref(),
            params.upload_id_marker.as_deref(),
            max_uploads,
        )
        .await
    {
        Ok((uploads, is_truncated)) => {
            // Convert to UploadInfo for XML response
            let upload_infos: Vec<xml::UploadInfo> = uploads
                .into_iter()
                .map(|u| xml::UploadInfo {
                    key: u.key,
                    upload_id: u.upload_id,
                    initiator_id: u.initiator_id,
                    storage_class: u.storage_class,
                    initiated: u.initiated,
                })
                .collect();

            let xml = xml::list_multipart_uploads_response(
                &bucket,
                params.prefix.as_deref(),
                params.delimiter.as_deref(),
                params.key_marker.as_deref(),
                params.upload_id_marker.as_deref(),
                max_uploads,
                is_truncated,
                &upload_infos,
            );
            success_response(StatusCode::OK, xml, &request_id)
        }
        Err(e) => error_response(e, &request_id),
    }
}

// ============= Bucket Versioning Operations =============

/// GET bucket versioning status
pub async fn get_bucket_versioning(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "GetBucketVersioning bucket={} request_id={}",
        bucket, request_id
    );

    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(b)) => {
            let xml = xml::get_bucket_versioning_response(&b.versioning);
            success_response(StatusCode::OK, xml, &request_id)
        }
        Ok(None) => error_response(Error::NoSuchBucket, &request_id),
        Err(e) => error_response(e, &request_id),
    }
}

/// PUT bucket versioning status
pub async fn put_bucket_versioning(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "PutBucketVersioning bucket={} request_id={}",
        bucket, request_id
    );

    // Check bucket exists
    let bucket_info = match state.metadata.get_bucket(&bucket).await {
        Ok(Some(b)) => b,
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
    };

    // Parse versioning configuration
    let status = match xml::parse_versioning_configuration(&body) {
        Ok(s) => s,
        Err(e) => return error_response(Error::MalformedXML(e.to_string()), &request_id),
    };

    // Object Lock requires versioning to stay enabled
    if bucket_info.object_lock_enabled && !status.is_versioning_enabled() {
        return error_response(
            Error::InvalidRequest("Cannot disable versioning on Object Lock enabled bucket".into()),
            &request_id,
        );
    }

    if let Err(e) = state.metadata.set_bucket_versioning(&bucket, status).await {
        return error_response(e, &request_id);
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}

// ============= List Object Versions =============

#[derive(Debug, Deserialize, Default)]
pub struct ListObjectVersionsQuery {
    versions: Option<String>,
    prefix: Option<String>,
    delimiter: Option<String>,
    #[serde(rename = "max-keys")]
    max_keys: Option<i32>,
    #[serde(rename = "key-marker")]
    key_marker: Option<String>,
    #[serde(rename = "version-id-marker")]
    version_id_marker: Option<String>,
}

/// GET list object versions
pub async fn list_object_versions(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(params): Query<ListObjectVersionsQuery>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "ListObjectVersions bucket={} request_id={}",
        bucket, request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    let max_keys = params.max_keys.unwrap_or(1000).min(1000);

    match state
        .metadata
        .list_object_versions(
            &bucket,
            params.prefix.as_deref(),
            params.delimiter.as_deref(),
            max_keys,
            params.key_marker.as_deref(),
            params.version_id_marker.as_deref(),
        )
        .await
    {
        Ok((
            versions,
            delete_markers,
            common_prefixes,
            is_truncated,
            next_key_marker,
            next_version_id_marker,
        )) => {
            let xml = xml::list_object_versions_response(
                &bucket,
                params.prefix.as_deref(),
                params.delimiter.as_deref(),
                params.key_marker.as_deref(),
                params.version_id_marker.as_deref(),
                max_keys,
                is_truncated,
                &versions,
                &delete_markers,
                &common_prefixes,
                next_key_marker.as_deref(),
                next_version_id_marker.as_deref(),
            );
            success_response(StatusCode::OK, xml, &request_id)
        }
        Err(e) => error_response(e, &request_id),
    }
}

// ============= Versioned Object Operations =============

/// GET object with optional version
pub async fn get_object_versioned(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    headers: HeaderMap,
    version_id: Option<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "GetObject bucket={} key={} version={:?} request_id={}",
        bucket, key, version_id, request_id
    );

    // Get object metadata (with optional version)
    let object = match state
        .metadata
        .get_object_version(&bucket, &key, version_id.as_deref())
        .await
    {
        Ok(Some(obj)) => obj,
        Ok(None) => return error_response(Error::NoSuchKey, &request_id),
        Err(e) => return error_response(e, &request_id),
    };

    // Check if it's a delete marker
    if object.is_delete_marker {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("x-amz-request-id", &request_id)
            .header("x-amz-version-id", &object.version_id)
            .header("x-amz-delete-marker", "true")
            .body(Body::empty())
            .unwrap();
    }

    // Check for Range header
    let range = headers
        .get("range")
        .and_then(|v| v.to_str().ok())
        .map(|r| hafiz_core::types::ByteRange::parse(r));

    // Determine storage key based on version
    let storage_key = if object.version_id == "null" {
        key.clone()
    } else {
        format!("{}?versionId={}", key, object.version_id)
    };

    // Get object data
    let data = if let Some(Ok(byte_range)) = range {
        match byte_range.resolve(object.size) {
            Ok((start, end)) => {
                match state
                    .storage
                    .get_range(&bucket, &storage_key, start as u64, end as u64)
                    .await
                {
                    Ok(data) => {
                        return Response::builder()
                            .status(StatusCode::PARTIAL_CONTENT)
                            .header("Content-Type", &object.content_type)
                            .header("Content-Length", data.len())
                            .header(
                                "Content-Range",
                                format!("bytes {}-{}/{}", start, end, object.size),
                            )
                            .header("ETag", format!("\"{}\"", object.etag))
                            .header("Last-Modified", format_http_datetime(&object.last_modified))
                            .header("x-amz-request-id", &request_id)
                            .header("x-amz-version-id", &object.version_id)
                            .body(Body::from(data))
                            .unwrap();
                    }
                    Err(e) => return error_response(e, &request_id),
                }
            }
            Err(e) => return error_response(e, &request_id),
        }
    } else {
        match state.storage.get(&bucket, &storage_key).await {
            Ok(data) => data,
            Err(e) => return error_response(e, &request_id),
        }
    };

    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", &object.content_type)
        .header("Content-Length", data.len())
        .header("ETag", format!("\"{}\"", object.etag))
        .header("Last-Modified", format_http_datetime(&object.last_modified))
        .header("x-amz-request-id", &request_id)
        .header("x-amz-version-id", &object.version_id);

    // Add SSE headers if encrypted
    if object.encryption.is_encrypted() {
        response = response.header(
            "x-amz-server-side-encryption",
            object.encryption.encryption_type.as_str(),
        );
        if let Some(ref md5) = object.encryption.sse_customer_key_md5 {
            response = response.header("x-amz-server-side-encryption-customer-key-MD5", md5);
        }
    }

    // Add user metadata
    for (k, v) in &object.metadata {
        response = response.header(format!("x-amz-meta-{}", k), v);
    }

    response.body(Body::from(data)).unwrap()
}

/// DELETE object with versioning support
pub async fn delete_object_versioned(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    version_id: Option<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "DeleteObject bucket={} key={} version={:?} request_id={}",
        bucket, key, version_id, request_id
    );

    // Get bucket to check versioning status
    let bucket_info = match state.metadata.get_bucket(&bucket).await {
        Ok(Some(b)) => b,
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
    };

    if let Some(vid) = version_id {
        // Delete specific version
        if let Err(e) = state
            .storage
            .delete(&bucket, &format!("{}?versionId={}", key, vid))
            .await
        {
            error!("Failed to delete object storage: {}", e);
        }

        match state
            .metadata
            .delete_object_version(&bucket, &key, &vid)
            .await
        {
            Ok(deleted) => {
                let mut builder = Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .header("x-amz-request-id", &request_id)
                    .header("x-amz-version-id", &vid);

                if deleted {
                    builder = builder.header("x-amz-delete-marker", "true");
                }

                builder.body(Body::empty()).unwrap()
            }
            Err(e) => error_response(e, &request_id),
        }
    } else if bucket_info.versioning.is_versioning_enabled() {
        // Versioned bucket without version ID: create delete marker
        match state.metadata.create_delete_marker(&bucket, &key).await {
            Ok(marker_version_id) => Response::builder()
                .status(StatusCode::NO_CONTENT)
                .header("x-amz-request-id", &request_id)
                .header("x-amz-version-id", &marker_version_id)
                .header("x-amz-delete-marker", "true")
                .body(Body::empty())
                .unwrap(),
            Err(e) => error_response(e, &request_id),
        }
    } else {
        // Non-versioned bucket: actually delete the object
        if let Err(e) = state.storage.delete(&bucket, &key).await {
            error!("Failed to delete object storage: {}", e);
        }

        if let Err(e) = state.metadata.delete_object(&bucket, &key).await {
            return error_response(e, &request_id);
        }

        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header("x-amz-request-id", &request_id)
            .body(Body::empty())
            .unwrap()
    }
}

// ============= Object Tagging Operations =============

/// GET object tagging
pub async fn get_object_tagging(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    version_id: Option<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "GetObjectTagging bucket={} key={} version={:?} request_id={}",
        bucket, key, version_id, request_id
    );

    // Check object exists
    match state
        .metadata
        .get_object_version(&bucket, &key, version_id.as_deref())
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => return error_response(Error::NoSuchKey, &request_id),
        Err(e) => return error_response(e, &request_id),
    }

    match state
        .metadata
        .get_object_tags(&bucket, &key, version_id.as_deref())
        .await
    {
        Ok(tags) => {
            let xml = xml::get_object_tagging_response(&tags);
            let mut builder = Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/xml")
                .header("x-amz-request-id", &request_id);

            if let Some(vid) = version_id {
                builder = builder.header("x-amz-version-id", vid);
            }

            builder.body(Body::from(xml)).unwrap()
        }
        Err(e) => error_response(e, &request_id),
    }
}

/// PUT object tagging
pub async fn put_object_tagging(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    version_id: Option<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "PutObjectTagging bucket={} key={} version={:?} request_id={}",
        bucket, key, version_id, request_id
    );

    // Check object exists
    match state
        .metadata
        .get_object_version(&bucket, &key, version_id.as_deref())
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => return error_response(Error::NoSuchKey, &request_id),
        Err(e) => return error_response(e, &request_id),
    }

    // Parse tagging XML
    let tags = match xml::parse_tagging(&body) {
        Ok(t) => t,
        Err(e) => return error_response(Error::MalformedXML(e.to_string()), &request_id),
    };

    // Validate tags
    for tag in &tags.tags {
        if let Err(e) = tag.validate() {
            return error_response(e, &request_id);
        }
    }

    if let Err(e) = state
        .metadata
        .put_object_tags(&bucket, &key, version_id.as_deref(), &tags)
        .await
    {
        return error_response(e, &request_id);
    }

    let mut builder = Response::builder()
        .status(StatusCode::OK)
        .header("x-amz-request-id", &request_id);

    if let Some(vid) = version_id {
        builder = builder.header("x-amz-version-id", vid);
    }

    builder.body(Body::empty()).unwrap()
}

/// DELETE object tagging
pub async fn delete_object_tagging(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    version_id: Option<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "DeleteObjectTagging bucket={} key={} version={:?} request_id={}",
        bucket, key, version_id, request_id
    );

    // Check object exists
    match state
        .metadata
        .get_object_version(&bucket, &key, version_id.as_deref())
        .await
    {
        Ok(Some(_)) => {}
        Ok(None) => return error_response(Error::NoSuchKey, &request_id),
        Err(e) => return error_response(e, &request_id),
    }

    if let Err(e) = state
        .metadata
        .delete_object_tags(&bucket, &key, version_id.as_deref())
        .await
    {
        return error_response(e, &request_id);
    }

    let mut builder = Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("x-amz-request-id", &request_id);

    if let Some(vid) = version_id {
        builder = builder.header("x-amz-version-id", vid);
    }

    builder.body(Body::empty()).unwrap()
}

// ============= Bucket Lifecycle Operations =============

/// GET bucket lifecycle configuration
pub async fn get_bucket_lifecycle(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!(
        "GetBucketLifecycle bucket={} request_id={}",
        bucket, request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    match state.metadata.get_bucket_lifecycle(&bucket).await {
        Ok(Some(config)) => {
            let xml = xml::get_bucket_lifecycle_response(&config);
            success_response(StatusCode::OK, xml, &request_id)
        }
        Ok(None) => {
            // No lifecycle configuration
            error_response(Error::NoSuchLifecycleConfiguration, &request_id)
        }
        Err(e) => error_response(e, &request_id),
    }
}

/// PUT bucket lifecycle configuration
pub async fn put_bucket_lifecycle(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "PutBucketLifecycle bucket={} request_id={}",
        bucket, request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    // Parse lifecycle configuration XML
    let config = match xml::parse_lifecycle_configuration(&body) {
        Ok(c) => c,
        Err(e) => return error_response(Error::MalformedXML(e.to_string()), &request_id),
    };

    // Validate rules
    for rule in &config.rules {
        if let Err(e) = rule.validate() {
            return error_response(e, &request_id);
        }
    }

    if let Err(e) = state.metadata.put_bucket_lifecycle(&bucket, &config).await {
        return error_response(e, &request_id);
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}

/// DELETE bucket lifecycle configuration
pub async fn delete_bucket_lifecycle(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    info!(
        "DeleteBucketLifecycle bucket={} request_id={}",
        bucket, request_id
    );

    // Check bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(None) => return error_response(Error::NoSuchBucket, &request_id),
        Err(e) => return error_response(e, &request_id),
        _ => {}
    }

    if let Err(e) = state.metadata.delete_bucket_lifecycle(&bucket).await {
        return error_response(e, &request_id);
    }

    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header("x-amz-request-id", &request_id)
        .body(Body::empty())
        .unwrap()
}
