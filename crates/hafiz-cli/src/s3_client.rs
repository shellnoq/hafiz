//! S3 client wrapper for Hafiz CLI

use crate::config::Config;
use anyhow::{Context, Result};
use aws_config::Region;
use aws_credential_types::Credentials;
use aws_sdk_s3::config::Builder as S3ConfigBuilder;
use aws_sdk_s3::Client;

/// Create an S3 client from configuration
pub async fn create_client(config: &Config) -> Result<Client> {
    config.validate()?;

    let endpoint = config.endpoint.as_ref().unwrap();
    let access_key = config.access_key.as_ref().unwrap();
    let secret_key = config.secret_key.as_ref().unwrap();

    let credentials = Credentials::new(access_key, secret_key, None, None, "hafiz-cli");

    let s3_config = S3ConfigBuilder::new()
        .region(Region::new(config.region.clone()))
        .credentials_provider(credentials)
        .endpoint_url(endpoint)
        .force_path_style(config.path_style)
        .build();

    Ok(Client::from_conf(s3_config))
}

/// Parse an S3 URI into bucket and key components
/// Format: s3://bucket/key or s3://bucket
#[derive(Debug, Clone)]
pub struct S3Uri {
    pub bucket: String,
    pub key: Option<String>,
}

impl S3Uri {
    /// Parse an S3 URI string
    pub fn parse(uri: &str) -> Result<Self> {
        // Handle s3:// prefix
        let path = uri
            .strip_prefix("s3://")
            .with_context(|| format!("Invalid S3 URI: {}. Must start with s3://", uri))?;

        if path.is_empty() {
            // s3:// - list buckets
            return Ok(Self {
                bucket: String::new(),
                key: None,
            });
        }

        // Split into bucket and key
        let (bucket, key) = match path.find('/') {
            Some(idx) => {
                let (b, k) = path.split_at(idx);
                let key = k.strip_prefix('/').unwrap_or(k);
                (
                    b.to_string(),
                    if key.is_empty() {
                        None
                    } else {
                        Some(key.to_string())
                    },
                )
            }
            None => (path.to_string(), None),
        };

        if bucket.is_empty() {
            anyhow::bail!("Invalid S3 URI: bucket name cannot be empty");
        }

        Ok(Self { bucket, key })
    }

    /// Check if this is a bucket-only URI (no key)
    pub fn is_bucket_only(&self) -> bool {
        self.key.is_none()
    }

    /// Check if this is a prefix (ends with /)
    pub fn is_prefix(&self) -> bool {
        self.key.as_ref().map_or(true, |k| k.ends_with('/'))
    }

    /// Get the key or empty string
    pub fn key_or_empty(&self) -> &str {
        self.key.as_deref().unwrap_or("")
    }

    /// Convert back to S3 URI string
    pub fn to_string(&self) -> String {
        match &self.key {
            Some(k) => format!("s3://{}/{}", self.bucket, k),
            None => format!("s3://{}", self.bucket),
        }
    }
}

/// Check if a path is an S3 URI
pub fn is_s3_uri(path: &str) -> bool {
    path.starts_with("s3://")
}

/// Determine the operation type based on source and destination
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferDirection {
    /// Local to S3 (upload)
    Upload,
    /// S3 to Local (download)
    Download,
    /// S3 to S3 (copy)
    S3ToS3,
    /// Local to Local (not supported)
    LocalToLocal,
}

impl TransferDirection {
    pub fn determine(source: &str, dest: &str) -> Self {
        match (is_s3_uri(source), is_s3_uri(dest)) {
            (false, true) => TransferDirection::Upload,
            (true, false) => TransferDirection::Download,
            (true, true) => TransferDirection::S3ToS3,
            (false, false) => TransferDirection::LocalToLocal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_s3_uri() {
        // Bucket only
        let uri = S3Uri::parse("s3://mybucket").unwrap();
        assert_eq!(uri.bucket, "mybucket");
        assert!(uri.key.is_none());

        // Bucket with trailing slash
        let uri = S3Uri::parse("s3://mybucket/").unwrap();
        assert_eq!(uri.bucket, "mybucket");
        assert!(uri.key.is_none());

        // Bucket with key
        let uri = S3Uri::parse("s3://mybucket/mykey").unwrap();
        assert_eq!(uri.bucket, "mybucket");
        assert_eq!(uri.key, Some("mykey".to_string()));

        // Bucket with prefix key
        let uri = S3Uri::parse("s3://mybucket/path/to/key").unwrap();
        assert_eq!(uri.bucket, "mybucket");
        assert_eq!(uri.key, Some("path/to/key".to_string()));

        // Bucket with prefix ending in /
        let uri = S3Uri::parse("s3://mybucket/path/to/").unwrap();
        assert_eq!(uri.bucket, "mybucket");
        assert_eq!(uri.key, Some("path/to/".to_string()));

        // Empty s3:// - list buckets
        let uri = S3Uri::parse("s3://").unwrap();
        assert!(uri.bucket.is_empty());
        assert!(uri.key.is_none());
    }

    #[test]
    fn test_invalid_s3_uri() {
        // Missing s3:// prefix
        assert!(S3Uri::parse("mybucket").is_err());
        assert!(S3Uri::parse("http://mybucket").is_err());
    }

    #[test]
    fn test_transfer_direction() {
        assert_eq!(
            TransferDirection::determine("./file.txt", "s3://bucket/key"),
            TransferDirection::Upload
        );
        assert_eq!(
            TransferDirection::determine("s3://bucket/key", "./file.txt"),
            TransferDirection::Download
        );
        assert_eq!(
            TransferDirection::determine("s3://bucket1/key", "s3://bucket2/key"),
            TransferDirection::S3ToS3
        );
        assert_eq!(
            TransferDirection::determine("./src", "./dst"),
            TransferDirection::LocalToLocal
        );
    }
}
