//! Bucket types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Bucket versioning status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum VersioningStatus {
    /// Versioning never enabled (default for new buckets)
    #[default]
    Unversioned,
    /// Versioning is enabled
    Enabled,
    /// Versioning was enabled but now suspended
    Suspended,
}

impl VersioningStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unversioned => "",
            Self::Enabled => "Enabled",
            Self::Suspended => "Suspended",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "enabled" => Self::Enabled,
            "suspended" => Self::Suspended,
            _ => Self::Unversioned,
        }
    }

    pub fn is_versioning_enabled(&self) -> bool {
        matches!(self, Self::Enabled)
    }

    /// Returns true if versioning was ever enabled (Enabled or Suspended)
    pub fn was_ever_enabled(&self) -> bool {
        matches!(self, Self::Enabled | Self::Suspended)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub owner_id: String,
    pub region: String,
    pub created_at: DateTime<Utc>,
    pub versioning: VersioningStatus,
    pub object_lock_enabled: bool,
}

impl Bucket {
    pub fn new(name: String, owner_id: String) -> Self {
        Self {
            name,
            owner_id,
            region: crate::DEFAULT_REGION.to_string(),
            created_at: Utc::now(),
            versioning: VersioningStatus::Unversioned,
            object_lock_enabled: false,
        }
    }

    pub fn with_versioning(mut self, status: VersioningStatus) -> Self {
        self.versioning = status;
        self
    }

    pub fn with_object_lock(mut self) -> Self {
        self.object_lock_enabled = true;
        self.versioning = VersioningStatus::Enabled; // Object Lock requires versioning
        self
    }

    pub fn validate_name(name: &str) -> Result<(), crate::Error> {
        if name.len() < crate::MIN_BUCKET_NAME_LENGTH {
            return Err(crate::Error::InvalidBucketName(
                "Bucket name too short (min 3 characters)".into(),
            ));
        }
        if name.len() > crate::MAX_BUCKET_NAME_LENGTH {
            return Err(crate::Error::InvalidBucketName(
                "Bucket name too long (max 63 characters)".into(),
            ));
        }

        let chars: Vec<char> = name.chars().collect();

        if !chars[0].is_ascii_lowercase() && !chars[0].is_ascii_digit() {
            return Err(crate::Error::InvalidBucketName(
                "Must start with lowercase letter or number".into(),
            ));
        }

        if !chars.last().unwrap().is_ascii_lowercase() && !chars.last().unwrap().is_ascii_digit() {
            return Err(crate::Error::InvalidBucketName(
                "Must end with lowercase letter or number".into(),
            ));
        }

        for c in &chars {
            if !c.is_ascii_lowercase() && !c.is_ascii_digit() && *c != '-' && *c != '.' {
                return Err(crate::Error::InvalidBucketName(format!(
                    "Invalid character: {}",
                    c
                )));
            }
        }

        if name.contains("..") {
            return Err(crate::Error::InvalidBucketName(
                "Cannot have consecutive periods".into(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: DateTime<Utc>,
}

impl From<Bucket> for BucketInfo {
    fn from(b: Bucket) -> Self {
        Self {
            name: b.name,
            creation_date: b.created_at,
        }
    }
}
