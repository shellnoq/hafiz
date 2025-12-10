//! Prometheus metrics for Hafiz
//!
//! Exposes metrics at `/metrics` endpoint in Prometheus format.

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use std::sync::Arc;
use std::time::Instant;
use tracing::debug;

/// Metric names
pub mod names {
    // HTTP metrics
    pub const HTTP_REQUESTS_TOTAL: &str = "hafiz_http_requests_total";
    pub const HTTP_REQUEST_DURATION_SECONDS: &str = "hafiz_http_request_duration_seconds";
    pub const HTTP_REQUEST_SIZE_BYTES: &str = "hafiz_http_request_size_bytes";
    pub const HTTP_RESPONSE_SIZE_BYTES: &str = "hafiz_http_response_size_bytes";
    pub const HTTP_ACTIVE_CONNECTIONS: &str = "hafiz_http_active_connections";

    // S3 operation metrics
    pub const S3_OPERATIONS_TOTAL: &str = "hafiz_s3_operations_total";
    pub const S3_OPERATION_DURATION_SECONDS: &str = "hafiz_s3_operation_duration_seconds";
    pub const S3_OPERATION_ERRORS_TOTAL: &str = "hafiz_s3_operation_errors_total";

    // Storage metrics
    pub const STORAGE_BYTES_READ_TOTAL: &str = "hafiz_storage_bytes_read_total";
    pub const STORAGE_BYTES_WRITTEN_TOTAL: &str = "hafiz_storage_bytes_written_total";
    pub const STORAGE_OBJECTS_TOTAL: &str = "hafiz_storage_objects_total";
    pub const STORAGE_BUCKETS_TOTAL: &str = "hafiz_storage_buckets_total";
    pub const STORAGE_USED_BYTES: &str = "hafiz_storage_used_bytes";

    // Multipart metrics
    pub const MULTIPART_UPLOADS_ACTIVE: &str = "hafiz_multipart_uploads_active";
    pub const MULTIPART_PARTS_UPLOADED_TOTAL: &str = "hafiz_multipart_parts_uploaded_total";

    // Cache metrics (if applicable)
    pub const CACHE_HITS_TOTAL: &str = "hafiz_cache_hits_total";
    pub const CACHE_MISSES_TOTAL: &str = "hafiz_cache_misses_total";

    // System metrics
    pub const UPTIME_SECONDS: &str = "hafiz_uptime_seconds";
    pub const INFO: &str = "hafiz_info";
}

/// S3 operation types for metrics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S3Operation {
    // Service
    ListBuckets,

    // Bucket
    CreateBucket,
    DeleteBucket,
    HeadBucket,
    GetBucketVersioning,
    PutBucketVersioning,
    GetBucketLifecycle,
    PutBucketLifecycle,
    DeleteBucketLifecycle,
    ListObjects,
    ListObjectVersions,
    ListMultipartUploads,

    // Object
    GetObject,
    PutObject,
    DeleteObject,
    HeadObject,
    CopyObject,
    GetObjectTagging,
    PutObjectTagging,
    DeleteObjectTagging,

    // Multipart
    CreateMultipartUpload,
    UploadPart,
    CompleteMultipartUpload,
    AbortMultipartUpload,
    ListParts,
}

impl S3Operation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ListBuckets => "ListBuckets",
            Self::CreateBucket => "CreateBucket",
            Self::DeleteBucket => "DeleteBucket",
            Self::HeadBucket => "HeadBucket",
            Self::GetBucketVersioning => "GetBucketVersioning",
            Self::PutBucketVersioning => "PutBucketVersioning",
            Self::GetBucketLifecycle => "GetBucketLifecycle",
            Self::PutBucketLifecycle => "PutBucketLifecycle",
            Self::DeleteBucketLifecycle => "DeleteBucketLifecycle",
            Self::ListObjects => "ListObjects",
            Self::ListObjectVersions => "ListObjectVersions",
            Self::ListMultipartUploads => "ListMultipartUploads",
            Self::GetObject => "GetObject",
            Self::PutObject => "PutObject",
            Self::DeleteObject => "DeleteObject",
            Self::HeadObject => "HeadObject",
            Self::CopyObject => "CopyObject",
            Self::GetObjectTagging => "GetObjectTagging",
            Self::PutObjectTagging => "PutObjectTagging",
            Self::DeleteObjectTagging => "DeleteObjectTagging",
            Self::CreateMultipartUpload => "CreateMultipartUpload",
            Self::UploadPart => "UploadPart",
            Self::CompleteMultipartUpload => "CompleteMultipartUpload",
            Self::AbortMultipartUpload => "AbortMultipartUpload",
            Self::ListParts => "ListParts",
        }
    }

    /// Detect operation from HTTP method and path
    pub fn from_request(method: &str, path: &str, query: Option<&str>) -> Option<Self> {
        let query = query.unwrap_or("");
        let has_key = path.split('/').filter(|s| !s.is_empty()).count() > 1;

        match (method, has_key) {
            // Service level
            ("GET", false) if path == "/" => Some(Self::ListBuckets),

            // Bucket level (no key)
            ("PUT", false) if !has_key && query.is_empty() => Some(Self::CreateBucket),
            ("DELETE", false) if !has_key && query.is_empty() => Some(Self::DeleteBucket),
            ("HEAD", false) if !has_key => Some(Self::HeadBucket),
            ("GET", false) if query.contains("versioning") => Some(Self::GetBucketVersioning),
            ("PUT", false) if query.contains("versioning") => Some(Self::PutBucketVersioning),
            ("GET", false) if query.contains("lifecycle") => Some(Self::GetBucketLifecycle),
            ("PUT", false) if query.contains("lifecycle") => Some(Self::PutBucketLifecycle),
            ("DELETE", false) if query.contains("lifecycle") => Some(Self::DeleteBucketLifecycle),
            ("GET", false) if query.contains("versions") => Some(Self::ListObjectVersions),
            ("GET", false) if query.contains("uploads") => Some(Self::ListMultipartUploads),
            ("GET", false) => Some(Self::ListObjects),

            // Object level (has key)
            ("GET", true) if query.contains("tagging") => Some(Self::GetObjectTagging),
            ("PUT", true) if query.contains("tagging") => Some(Self::PutObjectTagging),
            ("DELETE", true) if query.contains("tagging") => Some(Self::DeleteObjectTagging),
            ("POST", true) if query.contains("uploads") && !query.contains("uploadId") => {
                Some(Self::CreateMultipartUpload)
            }
            ("PUT", true) if query.contains("uploadId") && query.contains("partNumber") => {
                Some(Self::UploadPart)
            }
            ("POST", true) if query.contains("uploadId") => Some(Self::CompleteMultipartUpload),
            ("DELETE", true) if query.contains("uploadId") => Some(Self::AbortMultipartUpload),
            ("GET", true) if query.contains("uploadId") => Some(Self::ListParts),
            ("GET", true) => Some(Self::GetObject),
            ("PUT", true) => Some(Self::PutObject),
            ("DELETE", true) => Some(Self::DeleteObject),
            ("HEAD", true) => Some(Self::HeadObject),

            _ => None,
        }
    }
}

/// Metrics recorder
#[derive(Clone)]
pub struct MetricsRecorder {
    handle: PrometheusHandle,
    start_time: Instant,
}

impl MetricsRecorder {
    /// Initialize the metrics system
    pub fn new() -> Self {
        let builder = PrometheusBuilder::new();
        let handle = builder
            .install_recorder()
            .expect("Failed to install Prometheus recorder");

        // Set initial info metric
        gauge!(names::INFO, "version" => env!("CARGO_PKG_VERSION")).set(1.0);

        Self {
            handle,
            start_time: Instant::now(),
        }
    }

    /// Get metrics output in Prometheus format
    pub fn render(&self) -> String {
        // Update uptime
        gauge!(names::UPTIME_SECONDS).set(self.start_time.elapsed().as_secs_f64());

        self.handle.render()
    }

    /// Record an HTTP request
    pub fn record_http_request(
        &self,
        method: &str,
        path: &str,
        status: u16,
        duration_secs: f64,
        request_size: u64,
        response_size: u64,
    ) {
        let status_str = status.to_string();
        let status_class = format!("{}xx", status / 100);

        counter!(
            names::HTTP_REQUESTS_TOTAL,
            "method" => method.to_string(),
            "status" => status_str.clone(),
            "status_class" => status_class.clone()
        )
        .increment(1);

        histogram!(
            names::HTTP_REQUEST_DURATION_SECONDS,
            "method" => method.to_string()
        )
        .record(duration_secs);

        histogram!(names::HTTP_REQUEST_SIZE_BYTES).record(request_size as f64);
        histogram!(names::HTTP_RESPONSE_SIZE_BYTES).record(response_size as f64);
    }

    /// Record an S3 operation
    pub fn record_s3_operation(&self, op: S3Operation, success: bool, duration_secs: f64) {
        let op_name = op.as_str();

        counter!(
            names::S3_OPERATIONS_TOTAL,
            "operation" => op_name,
            "status" => if success { "success" } else { "error" }
        )
        .increment(1);

        histogram!(
            names::S3_OPERATION_DURATION_SECONDS,
            "operation" => op_name
        )
        .record(duration_secs);

        if !success {
            counter!(
                names::S3_OPERATION_ERRORS_TOTAL,
                "operation" => op_name
            )
            .increment(1);
        }
    }

    /// Record bytes read from storage
    pub fn record_bytes_read(&self, bytes: u64) {
        counter!(names::STORAGE_BYTES_READ_TOTAL).increment(bytes);
    }

    /// Record bytes written to storage
    pub fn record_bytes_written(&self, bytes: u64) {
        counter!(names::STORAGE_BYTES_WRITTEN_TOTAL).increment(bytes);
    }

    /// Update storage gauges
    pub fn update_storage_stats(&self, buckets: u64, objects: u64, used_bytes: u64) {
        gauge!(names::STORAGE_BUCKETS_TOTAL).set(buckets as f64);
        gauge!(names::STORAGE_OBJECTS_TOTAL).set(objects as f64);
        gauge!(names::STORAGE_USED_BYTES).set(used_bytes as f64);
    }

    /// Update active multipart uploads
    pub fn set_active_multipart_uploads(&self, count: u64) {
        gauge!(names::MULTIPART_UPLOADS_ACTIVE).set(count as f64);
    }

    /// Record multipart part upload
    pub fn record_part_uploaded(&self) {
        counter!(names::MULTIPART_PARTS_UPLOADED_TOTAL).increment(1);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        counter!(names::CACHE_HITS_TOTAL).increment(1);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        counter!(names::CACHE_MISSES_TOTAL).increment(1);
    }

    /// Update active connections
    pub fn set_active_connections(&self, count: u64) {
        gauge!(names::HTTP_ACTIVE_CONNECTIONS).set(count as f64);
    }
}

impl Default for MetricsRecorder {
    fn default() -> Self {
        Self::new()
    }
}

/// Axum middleware for recording HTTP metrics
pub async fn metrics_middleware(
    State(metrics): State<Arc<MetricsRecorder>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let query = request.uri().query().map(|s| s.to_string());

    // Get request size from Content-Length header
    let request_size = request
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    // Detect S3 operation
    let s3_op = S3Operation::from_request(&method, &path, query.as_deref());

    let response = next.run(request).await;

    let duration = start.elapsed().as_secs_f64();
    let status = response.status().as_u16();

    // Get response size from Content-Length header
    let response_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    // Record HTTP metrics
    metrics.record_http_request(
        &method,
        &path,
        status,
        duration,
        request_size,
        response_size,
    );

    // Record S3 operation metrics
    if let Some(op) = s3_op {
        let success = status < 400;
        metrics.record_s3_operation(op, success, duration);
    }

    debug!(
        method = %method,
        path = %path,
        status = %status,
        duration_ms = %(duration * 1000.0),
        "Request completed"
    );

    response
}

/// Handler for /metrics endpoint
pub async fn metrics_handler(State(metrics): State<Arc<MetricsRecorder>>) -> impl IntoResponse {
    let output = metrics.render();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4; charset=utf-8")],
        output,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_detection() {
        assert_eq!(
            S3Operation::from_request("GET", "/", None),
            Some(S3Operation::ListBuckets)
        );

        assert_eq!(
            S3Operation::from_request("PUT", "/mybucket", None),
            Some(S3Operation::CreateBucket)
        );

        assert_eq!(
            S3Operation::from_request("GET", "/mybucket/mykey", None),
            Some(S3Operation::GetObject)
        );

        assert_eq!(
            S3Operation::from_request("PUT", "/mybucket/mykey", Some("uploadId=123&partNumber=1")),
            Some(S3Operation::UploadPart)
        );

        assert_eq!(
            S3Operation::from_request("GET", "/mybucket", Some("versioning")),
            Some(S3Operation::GetBucketVersioning)
        );
    }
}
