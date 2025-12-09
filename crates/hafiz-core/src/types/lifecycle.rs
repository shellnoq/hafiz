//! Bucket Lifecycle Configuration Types

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Lifecycle configuration for a bucket
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LifecycleConfiguration {
    pub rules: Vec<LifecycleRule>,
}

impl LifecycleConfiguration {
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub fn add_rule(&mut self, rule: LifecycleRule) -> Result<(), crate::Error> {
        rule.validate()?;

        // Check for duplicate rule IDs
        if self.rules.iter().any(|r| r.id == rule.id) {
            return Err(crate::Error::InvalidArgument(format!(
                "Duplicate rule ID: {}",
                rule.id
            )));
        }

        // Max 1000 rules
        if self.rules.len() >= 1000 {
            return Err(crate::Error::InvalidArgument(
                "Maximum 1000 lifecycle rules per bucket".into(),
            ));
        }

        self.rules.push(rule);
        Ok(())
    }

    pub fn get_rule(&self, id: &str) -> Option<&LifecycleRule> {
        self.rules.iter().find(|r| r.id == id)
    }

    pub fn remove_rule(&mut self, id: &str) {
        self.rules.retain(|r| r.id != id);
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

/// A single lifecycle rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleRule {
    /// Unique identifier for the rule (max 255 chars)
    pub id: String,
    /// Whether the rule is enabled
    pub status: RuleStatus,
    /// Filter to select objects this rule applies to
    pub filter: LifecycleFilter,
    /// Expiration action for current versions
    pub expiration: Option<Expiration>,
    /// Expiration for noncurrent versions
    pub noncurrent_version_expiration: Option<NoncurrentVersionExpiration>,
    /// Abort incomplete multipart uploads
    pub abort_incomplete_multipart_upload: Option<AbortIncompleteMultipartUpload>,
    /// Transitions to different storage classes (future use)
    pub transitions: Vec<Transition>,
    /// Transitions for noncurrent versions (future use)
    pub noncurrent_version_transitions: Vec<NoncurrentVersionTransition>,
}

impl LifecycleRule {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: RuleStatus::Enabled,
            filter: LifecycleFilter::default(),
            expiration: None,
            noncurrent_version_expiration: None,
            abort_incomplete_multipart_upload: None,
            transitions: Vec::new(),
            noncurrent_version_transitions: Vec::new(),
        }
    }

    pub fn validate(&self) -> Result<(), crate::Error> {
        if self.id.is_empty() || self.id.len() > 255 {
            return Err(crate::Error::InvalidArgument(
                "Rule ID must be 1-255 characters".into(),
            ));
        }

        // Must have at least one action
        if self.expiration.is_none()
            && self.noncurrent_version_expiration.is_none()
            && self.abort_incomplete_multipart_upload.is_none()
            && self.transitions.is_empty()
            && self.noncurrent_version_transitions.is_empty()
        {
            return Err(crate::Error::InvalidArgument(
                "Rule must have at least one action".into(),
            ));
        }

        // Validate expiration
        if let Some(ref exp) = self.expiration {
            exp.validate()?;
        }

        Ok(())
    }

    pub fn with_status(mut self, status: RuleStatus) -> Self {
        self.status = status;
        self
    }

    pub fn with_prefix_filter(mut self, prefix: impl Into<String>) -> Self {
        self.filter = LifecycleFilter::Prefix(prefix.into());
        self
    }

    pub fn with_expiration_days(mut self, days: u32) -> Self {
        self.expiration = Some(Expiration::Days(days));
        self
    }

    pub fn with_expiration_date(mut self, date: NaiveDate) -> Self {
        self.expiration = Some(Expiration::Date(date));
        self
    }

    pub fn with_noncurrent_expiration(mut self, days: u32) -> Self {
        self.noncurrent_version_expiration = Some(NoncurrentVersionExpiration {
            noncurrent_days: days,
            newer_noncurrent_versions: None,
        });
        self
    }

    pub fn with_abort_incomplete_multipart(mut self, days: u32) -> Self {
        self.abort_incomplete_multipart_upload = Some(AbortIncompleteMultipartUpload {
            days_after_initiation: days,
        });
        self
    }

    /// Check if this rule applies to the given object key
    pub fn applies_to(&self, key: &str, tags: &[super::Tag]) -> bool {
        if self.status != RuleStatus::Enabled {
            return false;
        }
        self.filter.matches(key, tags)
    }
}

/// Rule status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuleStatus {
    Enabled,
    Disabled,
}

impl Default for RuleStatus {
    fn default() -> Self {
        Self::Enabled
    }
}

/// Filter to select objects for lifecycle rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleFilter {
    /// No filter - applies to all objects
    All,
    /// Filter by key prefix
    Prefix(String),
    /// Filter by single tag
    Tag(super::Tag),
    /// Filter by prefix AND tags
    And {
        prefix: Option<String>,
        tags: Vec<super::Tag>,
    },
}

impl Default for LifecycleFilter {
    fn default() -> Self {
        Self::All
    }
}

impl LifecycleFilter {
    pub fn matches(&self, key: &str, tags: &[super::Tag]) -> bool {
        match self {
            Self::All => true,
            Self::Prefix(prefix) => key.starts_with(prefix),
            Self::Tag(tag) => tags.iter().any(|t| t.key == tag.key && t.value == tag.value),
            Self::And { prefix, tags: filter_tags } => {
                let prefix_match = prefix.as_ref().map_or(true, |p| key.starts_with(p));
                let tags_match = filter_tags.iter().all(|ft| {
                    tags.iter().any(|t| t.key == ft.key && t.value == ft.value)
                });
                prefix_match && tags_match
            }
        }
    }
}

/// Expiration action for current object versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expiration {
    /// Delete after N days from creation
    Days(u32),
    /// Delete on specific date
    Date(NaiveDate),
    /// Delete expired delete markers (cleanup)
    ExpiredObjectDeleteMarker,
}

impl Expiration {
    pub fn validate(&self) -> Result<(), crate::Error> {
        match self {
            Self::Days(days) if *days == 0 => {
                Err(crate::Error::InvalidArgument("Days must be > 0".into()))
            }
            _ => Ok(()),
        }
    }

    /// Check if an object should be expired based on this expiration rule
    pub fn should_expire(&self, last_modified: &DateTime<Utc>) -> bool {
        let now = Utc::now();
        match self {
            Self::Days(days) => {
                let expiry = *last_modified + chrono::Duration::days(*days as i64);
                now >= expiry
            }
            Self::Date(date) => {
                let expiry = date.and_hms_opt(0, 0, 0).unwrap();
                now.date_naive() >= *date
            }
            Self::ExpiredObjectDeleteMarker => {
                // This is handled separately - check if delete marker is the only version
                false
            }
        }
    }
}

/// Expiration for noncurrent (old) versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoncurrentVersionExpiration {
    /// Delete noncurrent versions after N days
    pub noncurrent_days: u32,
    /// Keep at most N newer noncurrent versions (optional)
    pub newer_noncurrent_versions: Option<u32>,
}

impl NoncurrentVersionExpiration {
    /// Check if a noncurrent version should be expired
    pub fn should_expire(&self, became_noncurrent: &DateTime<Utc>) -> bool {
        let now = Utc::now();
        let expiry = *became_noncurrent + chrono::Duration::days(self.noncurrent_days as i64);
        now >= expiry
    }
}

/// Abort incomplete multipart uploads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortIncompleteMultipartUpload {
    /// Abort after N days from initiation
    pub days_after_initiation: u32,
}

impl AbortIncompleteMultipartUpload {
    /// Check if an incomplete multipart upload should be aborted
    pub fn should_abort(&self, initiated: &DateTime<Utc>) -> bool {
        let now = Utc::now();
        let expiry = *initiated + chrono::Duration::days(self.days_after_initiation as i64);
        now >= expiry
    }
}

/// Transition to different storage class (future use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    /// Days after creation to transition
    pub days: Option<u32>,
    /// Specific date to transition
    pub date: Option<NaiveDate>,
    /// Target storage class
    pub storage_class: StorageClass,
}

/// Transition for noncurrent versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoncurrentVersionTransition {
    /// Days after becoming noncurrent to transition
    pub noncurrent_days: u32,
    /// Target storage class
    pub storage_class: StorageClass,
    /// Keep at most N newer noncurrent versions (optional)
    pub newer_noncurrent_versions: Option<u32>,
}

/// Storage classes (for future tiered storage support)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageClass {
    Standard,
    InfrequentAccess,
    Archive,
    DeepArchive,
}

impl Default for StorageClass {
    fn default() -> Self {
        Self::Standard
    }
}

impl StorageClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Standard => "STANDARD",
            Self::InfrequentAccess => "STANDARD_IA",
            Self::Archive => "GLACIER",
            Self::DeepArchive => "DEEP_ARCHIVE",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expiration_days() {
        let exp = Expiration::Days(30);
        let old_date = Utc::now() - chrono::Duration::days(31);
        let recent_date = Utc::now() - chrono::Duration::days(1);

        assert!(exp.should_expire(&old_date));
        assert!(!exp.should_expire(&recent_date));
    }

    #[test]
    fn test_filter_prefix() {
        let filter = LifecycleFilter::Prefix("logs/".into());
        assert!(filter.matches("logs/2024/test.log", &[]));
        assert!(!filter.matches("data/file.txt", &[]));
    }

    #[test]
    fn test_filter_tag() {
        let filter = LifecycleFilter::Tag(super::super::Tag::new("env", "dev"));
        let tags = vec![super::super::Tag::new("env", "dev")];
        assert!(filter.matches("any-key", &tags));
        assert!(!filter.matches("any-key", &[]));
    }
}
