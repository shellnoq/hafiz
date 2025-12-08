//! Object types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::user::Owner;

/// Version ID for versioned objects
pub const NULL_VERSION_ID: &str = "null";

/// Maximum number of tags per object
pub const MAX_TAGS_PER_OBJECT: usize = 10;
/// Maximum tag key length
pub const MAX_TAG_KEY_LENGTH: usize = 128;
/// Maximum tag value length
pub const MAX_TAG_VALUE_LENGTH: usize = 256;

/// Object tag
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

impl Tag {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn validate(&self) -> Result<(), crate::Error> {
        if self.key.is_empty() || self.key.len() > MAX_TAG_KEY_LENGTH {
            return Err(crate::Error::InvalidArgument(format!(
                "Tag key must be 1-{} characters",
                MAX_TAG_KEY_LENGTH
            )));
        }
        if self.value.len() > MAX_TAG_VALUE_LENGTH {
            return Err(crate::Error::InvalidArgument(format!(
                "Tag value must be 0-{} characters",
                MAX_TAG_VALUE_LENGTH
            )));
        }
        Ok(())
    }
}

/// Tag set for an object
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TagSet {
    pub tags: Vec<Tag>,
}

impl TagSet {
    pub fn new() -> Self {
        Self { tags: Vec::new() }
    }

    pub fn add(&mut self, tag: Tag) -> Result<(), crate::Error> {
        tag.validate()?;
        if self.tags.len() >= MAX_TAGS_PER_OBJECT {
            return Err(crate::Error::InvalidArgument(format!(
                "Maximum {} tags per object",
                MAX_TAGS_PER_OBJECT
            )));
        }
        // Remove existing tag with same key
        self.tags.retain(|t| t.key != tag.key);
        self.tags.push(tag);
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.tags.iter().find(|t| t.key == key).map(|t| t.value.as_str())
    }

    pub fn remove(&mut self, key: &str) {
        self.tags.retain(|t| t.key != key);
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tags.len()
    }
}

/// Object version status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VersionStatus {
    /// Current/latest version
    Current,
    /// Previous version (not latest)
    Previous,
    /// Delete marker (object deleted but versioned)
    DeleteMarker,
}

impl Default for VersionStatus {
    fn default() -> Self {
        Self::Current
    }
}

/// Encryption type for objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EncryptionType {
    /// No encryption
    #[default]
    None,
    /// SSE-S3: Server-managed keys
    SseS3,
    /// SSE-C: Customer-provided keys
    SseC,
}

impl EncryptionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::SseS3 => "AES256",
            Self::SseC => "AES256",
        }
    }
    
    pub fn from_header(header: Option<&str>) -> Self {
        match header {
            Some("AES256") => Self::SseS3,
            Some("aws:kms") => Self::SseS3,
            _ => Self::None,
        }
    }
}

/// Encryption information for an object
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EncryptionInfo {
    /// Encryption type
    pub encryption_type: EncryptionType,
    /// Encrypted Data Encryption Key (base64, for SSE-S3)
    pub encrypted_dek: Option<String>,
    /// Nonce for DEK encryption (base64)
    pub dek_nonce: Option<String>,
    /// Nonce for data encryption (base64)
    pub data_nonce: Option<String>,
    /// MD5 of customer key (for SSE-C)
    pub sse_customer_key_md5: Option<String>,
}

impl EncryptionInfo {
    pub fn none() -> Self {
        Self::default()
    }
    
    pub fn is_encrypted(&self) -> bool {
        self.encryption_type != EncryptionType::None
    }
}

/// Simple Object representation for API layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub bucket: String,
    pub key: String,
    pub size: i64,
    pub etag: String,
    pub content_type: Option<String>,
    pub metadata: HashMap<String, String>,
    pub last_modified: DateTime<Utc>,
    pub owner: Option<Owner>,
    pub encryption: Option<EncryptionInfo>,
}

impl Object {
    pub fn new(bucket: String, key: String, size: i64, etag: String) -> Self {
        Self {
            bucket,
            key,
            size,
            etag,
            content_type: None,
            metadata: HashMap::new(),
            last_modified: Utc::now(),
            owner: None,
            encryption: None,
        }
    }
}

/// Internal object with versioning support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInternal {
    pub bucket: String,
    pub key: String,
    pub version_id: String,
    pub size: i64,
    pub etag: String,
    pub content_type: String,
    pub metadata: HashMap<String, String>,
    pub last_modified: DateTime<Utc>,
    pub is_latest: bool,
    pub is_delete_marker: bool,
    /// Encryption information (None if not encrypted)
    #[serde(default)]
    pub encryption: EncryptionInfo,
}

impl ObjectInternal {
    pub fn new(bucket: String, key: String, size: i64, etag: String, content_type: String) -> Self {
        Self {
            bucket,
            key,
            version_id: NULL_VERSION_ID.to_string(),
            size,
            etag,
            content_type,
            metadata: HashMap::new(),
            last_modified: Utc::now(),
            is_latest: true,
            is_delete_marker: false,
            encryption: EncryptionInfo::none(),
        }
    }

    pub fn with_version(mut self, version_id: String) -> Self {
        self.version_id = version_id;
        self
    }

    pub fn with_encryption(mut self, encryption: EncryptionInfo) -> Self {
        self.encryption = encryption;
        self
    }

    pub fn as_delete_marker(bucket: String, key: String, version_id: String) -> Self {
        Self {
            bucket,
            key,
            version_id,
            size: 0,
            etag: String::new(),
            content_type: String::new(),
            metadata: HashMap::new(),
            last_modified: Utc::now(),
            is_latest: true,
            is_delete_marker: true,
            encryption: EncryptionInfo::none(),
        }
    }

    pub fn validate_key(key: &str) -> Result<(), crate::Error> {
        if key.is_empty() {
            return Err(crate::Error::InvalidArgument("Key cannot be empty".into()));
        }
        if key.len() > crate::MAX_KEY_LENGTH {
            return Err(crate::Error::InvalidArgument(format!(
                "Key too long (max {} bytes)",
                crate::MAX_KEY_LENGTH
            )));
        }
        Ok(())
    }

    /// Generate a new version ID
    pub fn generate_version_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        // Format: base62 encoded timestamp + random suffix
        format!("{:016x}{:08x}", timestamp, rand::random::<u32>())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub key: String,
    pub last_modified: DateTime<Utc>,
    pub etag: String,
    pub size: i64,
    pub storage_class: String,
    pub version_id: Option<String>,
    pub is_latest: Option<bool>,
}

impl From<Object> for ObjectInfo {
    fn from(o: Object) -> Self {
        Self {
            key: o.key,
            last_modified: o.last_modified,
            etag: o.etag,
            size: o.size,
            storage_class: "STANDARD".to_string(),
            version_id: Some(o.version_id),
            is_latest: Some(o.is_latest),
        }
    }
}

/// Object version for ListObjectVersions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectVersion {
    pub key: String,
    pub version_id: String,
    pub is_latest: bool,
    pub last_modified: DateTime<Utc>,
    pub etag: String,
    pub size: i64,
    #[serde(default)]
    pub storage_class: Option<String>,
    pub owner: Option<super::user::Owner>,
}

/// Delete marker for ListObjectVersions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMarker {
    pub key: String,
    pub version_id: String,
    pub is_latest: bool,
    pub last_modified: DateTime<Utc>,
    pub owner: Option<super::user::Owner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectsResult {
    pub name: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: i32,
    pub is_truncated: bool,
    pub contents: Vec<ObjectInfo>,
    pub common_prefixes: Vec<String>,
    pub continuation_token: Option<String>,
    pub next_continuation_token: Option<String>,
}

/// Result for ListObjectVersions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListVersionsResult {
    pub name: String,
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub max_keys: i32,
    pub is_truncated: bool,
    pub versions: Vec<ObjectVersion>,
    pub delete_markers: Vec<DeleteMarker>,
    pub common_prefixes: Vec<String>,
    pub key_marker: Option<String>,
    pub version_id_marker: Option<String>,
    pub next_key_marker: Option<String>,
    pub next_version_id_marker: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ByteRange {
    pub start: Option<i64>,
    pub end: Option<i64>,
}

impl ByteRange {
    pub fn parse(header: &str) -> Result<Self, crate::Error> {
        if !header.starts_with("bytes=") {
            return Err(crate::Error::InvalidRange("Invalid range format".into()));
        }

        let range_str = &header[6..];
        let parts: Vec<&str> = range_str.split('-').collect();
        
        if parts.len() != 2 {
            return Err(crate::Error::InvalidRange("Invalid range format".into()));
        }

        let start = if parts[0].is_empty() {
            None
        } else {
            Some(parts[0].parse::<i64>().map_err(|_| {
                crate::Error::InvalidRange("Invalid range start".into())
            })?)
        };

        let end = if parts[1].is_empty() {
            None
        } else {
            Some(parts[1].parse::<i64>().map_err(|_| {
                crate::Error::InvalidRange("Invalid range end".into())
            })?)
        };

        Ok(ByteRange { start, end })
    }

    pub fn resolve(&self, size: i64) -> Result<(i64, i64), crate::Error> {
        match (self.start, self.end) {
            (Some(start), Some(end)) => {
                if start > end || start >= size {
                    return Err(crate::Error::InvalidRange("Range not satisfiable".into()));
                }
                Ok((start, std::cmp::min(end, size - 1)))
            }
            (Some(start), None) => {
                if start >= size {
                    return Err(crate::Error::InvalidRange("Range not satisfiable".into()));
                }
                Ok((start, size - 1))
            }
            (None, Some(suffix)) => {
                let start = std::cmp::max(0, size - suffix);
                Ok((start, size - 1))
            }
            (None, None) => Err(crate::Error::InvalidRange("Invalid range".into())),
        }
    }
}
