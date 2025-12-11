//! Common types used across the system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Request ID for tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(pub String);

impl RequestId {
    /// Generate a new request ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase())
    }

    /// Create from string
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for RequestId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Region specification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Region(pub String);

impl Region {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn us_east_1() -> Self {
        Self("us-east-1".to_string())
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::us_east_1()
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Region {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Content type with common MIME types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentType(pub String);

impl ContentType {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn octet_stream() -> Self {
        Self("application/octet-stream".to_string())
    }

    pub fn json() -> Self {
        Self("application/json".to_string())
    }

    pub fn xml() -> Self {
        Self("application/xml".to_string())
    }

    pub fn text_plain() -> Self {
        Self("text/plain".to_string())
    }

    pub fn text_html() -> Self {
        Self("text/html".to_string())
    }

    /// Guess content type from file extension
    pub fn from_extension(ext: &str) -> Self {
        let mime = match ext.to_lowercase().as_str() {
            "txt" => "text/plain",
            "html" | "htm" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "json" => "application/json",
            "xml" => "application/xml",
            "pdf" => "application/pdf",
            "zip" => "application/zip",
            "gz" | "gzip" => "application/gzip",
            "tar" => "application/x-tar",
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "webp" => "image/webp",
            "ico" => "image/x-icon",
            "mp3" => "audio/mpeg",
            "mp4" => "video/mp4",
            "webm" => "video/webm",
            "woff" => "font/woff",
            "woff2" => "font/woff2",
            "csv" => "text/csv",
            "md" => "text/markdown",
            "yaml" | "yml" => "application/yaml",
            "toml" => "application/toml",
            _ => "application/octet-stream",
        };
        Self(mime.to_string())
    }

    /// Guess content type from filename
    pub fn from_filename(filename: &str) -> Self {
        if let Some(ext) = filename.rsplit('.').next() {
            Self::from_extension(ext)
        } else {
            Self::octet_stream()
        }
    }
}

impl Default for ContentType {
    fn default() -> Self {
        Self::octet_stream()
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ContentType {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// ETag (entity tag)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ETag(pub String);

impl ETag {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Create ETag from MD5 hash
    pub fn from_md5(hash: &[u8]) -> Self {
        Self(format!("\"{}\"", hex::encode(hash)))
    }

    /// Create ETag for multipart upload
    pub fn from_multipart(part_etags: &[String], part_count: usize) -> Self {
        use digest::Digest;
        use md5::Md5;

        let mut hasher = Md5::new();
        for etag in part_etags {
            // Remove quotes and decode hex
            let clean = etag.trim_matches('"');
            if let Ok(bytes) = hex::decode(clean) {
                hasher.update(&bytes);
            }
        }
        let hash = hasher.finalize();
        Self(format!("\"{}-{}\"", hex::encode(hash), part_count))
    }

    /// Get the hash value without quotes
    pub fn hash(&self) -> &str {
        self.0.trim_matches('"')
    }

    /// Check if this is a multipart ETag
    pub fn is_multipart(&self) -> bool {
        self.0.contains('-')
    }
}

impl fmt::Display for ETag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for ETag {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Size with human-readable formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ByteSize(pub i64);

impl ByteSize {
    pub const BYTE: i64 = 1;
    pub const KB: i64 = 1024;
    pub const MB: i64 = 1024 * 1024;
    pub const GB: i64 = 1024 * 1024 * 1024;
    pub const TB: i64 = 1024 * 1024 * 1024 * 1024;
    pub const PB: i64 = 1024 * 1024 * 1024 * 1024 * 1024;

    pub fn bytes(n: i64) -> Self {
        Self(n)
    }

    pub fn kb(n: i64) -> Self {
        Self(n * Self::KB)
    }

    pub fn mb(n: i64) -> Self {
        Self(n * Self::MB)
    }

    pub fn gb(n: i64) -> Self {
        Self(n * Self::GB)
    }

    pub fn tb(n: i64) -> Self {
        Self(n * Self::TB)
    }

    pub fn as_bytes(&self) -> i64 {
        self.0
    }

    /// Parse from string like "10MB", "1.5GB"
    pub fn parse(s: &str) -> Result<Self, crate::error::Error> {
        let s = s.trim().to_uppercase();

        let (num_str, unit) = if s.ends_with("PB") || s.ends_with("PIB") {
            (&s[..s.len()-2], Self::PB)
        } else if s.ends_with("TB") || s.ends_with("TIB") {
            (&s[..s.len()-2], Self::TB)
        } else if s.ends_with("GB") || s.ends_with("GIB") {
            (&s[..s.len()-2], Self::GB)
        } else if s.ends_with("MB") || s.ends_with("MIB") {
            (&s[..s.len()-2], Self::MB)
        } else if s.ends_with("KB") || s.ends_with("KIB") {
            (&s[..s.len()-2], Self::KB)
        } else if s.ends_with('B') {
            (&s[..s.len()-1], Self::BYTE)
        } else {
            (s.as_str(), Self::BYTE)
        };

        let num: f64 = num_str.trim().parse().map_err(|_| {
            crate::error::Error::InvalidArgument(format!("Invalid size: {}", s))
        })?;

        Ok(Self((num * unit as f64) as i64))
    }

    /// Format as human-readable string
    pub fn to_human_readable(&self) -> String {
        let bytes = self.0 as f64;
        if bytes >= Self::PB as f64 {
            format!("{:.2} PB", bytes / Self::PB as f64)
        } else if bytes >= Self::TB as f64 {
            format!("{:.2} TB", bytes / Self::TB as f64)
        } else if bytes >= Self::GB as f64 {
            format!("{:.2} GB", bytes / Self::GB as f64)
        } else if bytes >= Self::MB as f64 {
            format!("{:.2} MB", bytes / Self::MB as f64)
        } else if bytes >= Self::KB as f64 {
            format!("{:.2} KB", bytes / Self::KB as f64)
        } else {
            format!("{} B", self.0)
        }
    }
}

impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_human_readable())
    }
}

/// Timestamp wrapper with S3-compatible formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn from_datetime(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }

    /// Format as ISO 8601 (S3 format)
    pub fn to_iso8601(&self) -> String {
        self.0.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
    }

    /// Format as RFC 2822 (HTTP header format)
    pub fn to_rfc2822(&self) -> String {
        self.0.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }

    /// Format for AWS Signature V4
    pub fn to_amz_date(&self) -> String {
        self.0.format("%Y%m%dT%H%M%SZ").to_string()
    }

    /// Format date only (for signature)
    pub fn to_date_stamp(&self) -> String {
        self.0.format("%Y%m%d").to_string()
    }
}

impl Default for Timestamp {
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso8601())
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(dt: DateTime<Utc>) -> Self {
        Self(dt)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(ts: Timestamp) -> Self {
        ts.0
    }
}

/// Pagination parameters
#[derive(Debug, Clone, Default)]
pub struct Pagination {
    /// Maximum number of items
    pub max_keys: i32,
    /// Continuation token
    pub continuation_token: Option<String>,
    /// Start after key
    pub start_after: Option<String>,
    /// Prefix filter
    pub prefix: Option<String>,
    /// Delimiter for common prefixes
    pub delimiter: Option<String>,
}

impl Pagination {
    pub fn new() -> Self {
        Self {
            max_keys: 1000,
            ..Default::default()
        }
    }

    pub fn with_max_keys(mut self, max_keys: i32) -> Self {
        self.max_keys = max_keys;
        self
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn with_delimiter(mut self, delimiter: impl Into<String>) -> Self {
        self.delimiter = Some(delimiter.into());
        self
    }
}

/// Common S3 headers
pub mod headers {
    pub const X_AMZ_REQUEST_ID: &str = "x-amz-request-id";
    pub const X_AMZ_ID_2: &str = "x-amz-id-2";
    pub const X_AMZ_DATE: &str = "x-amz-date";
    pub const X_AMZ_CONTENT_SHA256: &str = "x-amz-content-sha256";
    pub const X_AMZ_SECURITY_TOKEN: &str = "x-amz-security-token";
    pub const X_AMZ_META_PREFIX: &str = "x-amz-meta-";
    pub const X_AMZ_COPY_SOURCE: &str = "x-amz-copy-source";
    pub const X_AMZ_SERVER_SIDE_ENCRYPTION: &str = "x-amz-server-side-encryption";
    pub const X_AMZ_VERSION_ID: &str = "x-amz-version-id";
    pub const X_AMZ_DELETE_MARKER: &str = "x-amz-delete-marker";
    pub const X_AMZ_STORAGE_CLASS: &str = "x-amz-storage-class";
    pub const X_AMZ_TAGGING: &str = "x-amz-tagging";
    pub const X_AMZ_OBJECT_LOCK_MODE: &str = "x-amz-object-lock-mode";
    pub const X_AMZ_OBJECT_LOCK_RETAIN_UNTIL_DATE: &str = "x-amz-object-lock-retain-until-date";
    pub const X_AMZ_BUCKET_REGION: &str = "x-amz-bucket-region";
    pub const AUTHORIZATION: &str = "authorization";
    pub const HOST: &str = "host";
    pub const CONTENT_TYPE: &str = "content-type";
    pub const CONTENT_LENGTH: &str = "content-length";
    pub const CONTENT_MD5: &str = "content-md5";
    pub const ETAG: &str = "etag";
    pub const LAST_MODIFIED: &str = "last-modified";
    pub const RANGE: &str = "range";
    pub const IF_MATCH: &str = "if-match";
    pub const IF_NONE_MATCH: &str = "if-none-match";
    pub const IF_MODIFIED_SINCE: &str = "if-modified-since";
    pub const IF_UNMODIFIED_SINCE: &str = "if-unmodified-since";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id() {
        let id = RequestId::new();
        assert_eq!(id.0.len(), 32);
    }

    #[test]
    fn test_content_type_from_extension() {
        assert_eq!(ContentType::from_extension("json").0, "application/json");
        assert_eq!(ContentType::from_extension("png").0, "image/png");
        assert_eq!(ContentType::from_extension("unknown").0, "application/octet-stream");
    }

    #[test]
    fn test_content_type_from_filename() {
        assert_eq!(ContentType::from_filename("file.json").0, "application/json");
        assert_eq!(ContentType::from_filename("image.PNG").0, "image/png");
    }

    #[test]
    fn test_byte_size() {
        assert_eq!(ByteSize::mb(1).as_bytes(), 1024 * 1024);
        assert_eq!(ByteSize::gb(1).as_bytes(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_byte_size_parse() {
        assert_eq!(ByteSize::parse("10MB").unwrap().as_bytes(), 10 * 1024 * 1024);
        assert_eq!(ByteSize::parse("1GB").unwrap().as_bytes(), 1024 * 1024 * 1024);
        assert_eq!(ByteSize::parse("1024").unwrap().as_bytes(), 1024);
    }

    #[test]
    fn test_byte_size_display() {
        assert_eq!(ByteSize::mb(1).to_human_readable(), "1.00 MB");
        assert_eq!(ByteSize::gb(2).to_human_readable(), "2.00 GB");
    }

    #[test]
    fn test_timestamp_formats() {
        let ts = Timestamp::now();
        assert!(ts.to_iso8601().contains('T'));
        assert!(ts.to_amz_date().contains('T'));
        assert_eq!(ts.to_date_stamp().len(), 8);
    }

    #[test]
    fn test_etag() {
        use digest::Digest;
        use md5::Md5;

        let mut hasher = Md5::new();
        hasher.update(b"hello");
        let hash = hasher.finalize();
        let etag = ETag::from_md5(&hash);
        assert!(etag.0.starts_with('"'));
        assert!(etag.0.ends_with('"'));
        assert!(!etag.is_multipart());
    }

    #[test]
    fn test_multipart_etag() {
        let parts = vec!["abc123".to_string(), "def456".to_string()];
        let etag = ETag::from_multipart(&parts, 2);
        assert!(etag.is_multipart());
        assert!(etag.0.contains("-2"));
    }
}
