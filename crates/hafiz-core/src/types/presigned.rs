//! Pre-signed URL types for temporary access
//!
//! Pre-signed URLs allow temporary access to private objects
//! without requiring authentication.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Pre-signed URL request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedRequest {
    /// HTTP method (GET, PUT, DELETE)
    pub method: PresignedMethod,
    /// Bucket name
    pub bucket: String,
    /// Object key
    pub key: String,
    /// Expiration duration in seconds (default: 3600)
    pub expires_in: u64,
    /// Content-Type for PUT requests
    pub content_type: Option<String>,
    /// Content-MD5 for PUT requests  
    pub content_md5: Option<String>,
    /// Custom headers to sign
    pub signed_headers: Option<Vec<(String, String)>>,
    /// Version ID for versioned objects
    pub version_id: Option<String>,
}

impl Default for PresignedRequest {
    fn default() -> Self {
        Self {
            method: PresignedMethod::Get,
            bucket: String::new(),
            key: String::new(),
            expires_in: 3600,
            content_type: None,
            content_md5: None,
            signed_headers: None,
            version_id: None,
        }
    }
}

/// HTTP methods supported for pre-signed URLs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PresignedMethod {
    /// GET - Download object
    Get,
    /// PUT - Upload object
    Put,
    /// DELETE - Delete object
    Delete,
    /// HEAD - Get object metadata
    Head,
}

impl std::fmt::Display for PresignedMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Head => write!(f, "HEAD"),
        }
    }
}

impl std::str::FromStr for PresignedMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "HEAD" => Ok(Self::Head),
            _ => Err(format!("Invalid method: {}", s)),
        }
    }
}

/// Pre-signed URL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    /// The complete pre-signed URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Expiration time
    pub expires_at: DateTime<Utc>,
    /// Headers to include with request (for PUT)
    pub headers: Option<Vec<(String, String)>>,
}

/// Limits for pre-signed URLs
pub struct PresignedLimits;

impl PresignedLimits {
    /// Minimum expiration time (1 second)
    pub const MIN_EXPIRES: u64 = 1;
    
    /// Maximum expiration time (7 days)
    pub const MAX_EXPIRES: u64 = 7 * 24 * 60 * 60;
    
    /// Default expiration time (1 hour)
    pub const DEFAULT_EXPIRES: u64 = 3600;
    
    /// Validate expiration time
    pub fn validate_expires(seconds: u64) -> Result<u64, String> {
        if seconds < Self::MIN_EXPIRES {
            Err(format!(
                "Expiration must be at least {} second",
                Self::MIN_EXPIRES
            ))
        } else if seconds > Self::MAX_EXPIRES {
            Err(format!(
                "Expiration cannot exceed {} seconds (7 days)",
                Self::MAX_EXPIRES
            ))
        } else {
            Ok(seconds)
        }
    }
}

/// Builder for pre-signed URL requests
pub struct PresignedRequestBuilder {
    request: PresignedRequest,
}

impl PresignedRequestBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            request: PresignedRequest::default(),
        }
    }

    /// Set the HTTP method
    pub fn method(mut self, method: PresignedMethod) -> Self {
        self.request.method = method;
        self
    }

    /// Set the bucket
    pub fn bucket(mut self, bucket: impl Into<String>) -> Self {
        self.request.bucket = bucket.into();
        self
    }

    /// Set the object key
    pub fn key(mut self, key: impl Into<String>) -> Self {
        self.request.key = key.into();
        self
    }

    /// Set expiration in seconds
    pub fn expires_in(mut self, seconds: u64) -> Self {
        self.request.expires_in = seconds;
        self
    }

    /// Set content type (for PUT)
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.request.content_type = Some(content_type.into());
        self
    }

    /// Set content MD5 (for PUT)
    pub fn content_md5(mut self, md5: impl Into<String>) -> Self {
        self.request.content_md5 = Some(md5.into());
        self
    }

    /// Add a signed header
    pub fn signed_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let headers = self.request.signed_headers.get_or_insert_with(Vec::new);
        headers.push((key.into(), value.into()));
        self
    }

    /// Set version ID
    pub fn version_id(mut self, version_id: impl Into<String>) -> Self {
        self.request.version_id = Some(version_id.into());
        self
    }

    /// Build the request
    pub fn build(self) -> Result<PresignedRequest, String> {
        if self.request.bucket.is_empty() {
            return Err("Bucket name is required".to_string());
        }
        if self.request.key.is_empty() {
            return Err("Object key is required".to_string());
        }
        
        PresignedLimits::validate_expires(self.request.expires_in)?;
        
        Ok(self.request)
    }
}

impl Default for PresignedRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presigned_request_builder() {
        let request = PresignedRequestBuilder::new()
            .method(PresignedMethod::Get)
            .bucket("my-bucket")
            .key("my-object.txt")
            .expires_in(3600)
            .build()
            .unwrap();

        assert_eq!(request.bucket, "my-bucket");
        assert_eq!(request.key, "my-object.txt");
        assert_eq!(request.expires_in, 3600);
    }

    #[test]
    fn test_presigned_limits() {
        assert!(PresignedLimits::validate_expires(0).is_err());
        assert!(PresignedLimits::validate_expires(1).is_ok());
        assert!(PresignedLimits::validate_expires(3600).is_ok());
        assert!(PresignedLimits::validate_expires(604800).is_ok()); // 7 days
        assert!(PresignedLimits::validate_expires(604801).is_err()); // > 7 days
    }

    #[test]
    fn test_method_parsing() {
        assert_eq!("GET".parse::<PresignedMethod>().unwrap(), PresignedMethod::Get);
        assert_eq!("put".parse::<PresignedMethod>().unwrap(), PresignedMethod::Put);
        assert!("INVALID".parse::<PresignedMethod>().is_err());
    }
}
