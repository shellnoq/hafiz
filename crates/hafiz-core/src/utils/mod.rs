//! Utility functions

use uuid::Uuid;

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string().replace("-", "").to_uppercase()
}

/// Generate an ETag from content hash
pub fn generate_etag(md5_hash: &str) -> String {
    format!("\"{}\"", md5_hash)
}

/// Parse ETag (remove quotes)
pub fn parse_etag(etag: &str) -> String {
    etag.trim_matches('"').to_string()
}

/// XML escape string
pub fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Format datetime for S3 responses
pub fn format_s3_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Format datetime for HTTP headers
pub fn format_http_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}
