//! Object Lock / WORM (Write Once Read Many) types
//!
//! Implements S3-compatible Object Lock for regulatory compliance:
//! - SEC 17a-4 (Securities and Exchange Commission)
//! - FINRA (Financial Industry Regulatory Authority)
//! - HIPAA (Health Insurance Portability and Accountability Act)
//! - GDPR (General Data Protection Regulation)
//!
//! Reference: https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

// ============================================================================
// Object Lock Configuration (Bucket Level)
// ============================================================================

/// Object Lock configuration for a bucket
///
/// Once enabled on a bucket, Object Lock cannot be disabled.
/// All objects in the bucket can have retention settings.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "ObjectLockConfiguration")]
pub struct ObjectLockConfiguration {
    /// Whether Object Lock is enabled
    /// Valid values: "Enabled"
    #[serde(rename = "ObjectLockEnabled", skip_serializing_if = "Option::is_none")]
    pub object_lock_enabled: Option<String>,

    /// Default retention settings for the bucket
    #[serde(rename = "Rule", skip_serializing_if = "Option::is_none")]
    pub rule: Option<ObjectLockRule>,
}

/// Object Lock rule with default retention
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Rule")]
pub struct ObjectLockRule {
    /// Default retention settings
    #[serde(rename = "DefaultRetention")]
    pub default_retention: DefaultRetention,
}

/// Default retention settings for new objects
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "DefaultRetention")]
pub struct DefaultRetention {
    /// Retention mode: GOVERNANCE or COMPLIANCE
    #[serde(rename = "Mode", skip_serializing_if = "Option::is_none")]
    pub mode: Option<RetentionMode>,

    /// Retention period in days (1-36500)
    #[serde(rename = "Days", skip_serializing_if = "Option::is_none")]
    pub days: Option<u32>,

    /// Retention period in years (1-100)
    #[serde(rename = "Years", skip_serializing_if = "Option::is_none")]
    pub years: Option<u32>,
}

// ============================================================================
// Retention Mode
// ============================================================================

/// Object Lock retention mode
///
/// - GOVERNANCE: Users with specific IAM permissions can delete/modify
/// - COMPLIANCE: No one (including root) can delete until retention expires
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RetentionMode {
    /// Governance mode - can be overridden with special permissions
    Governance,
    /// Compliance mode - cannot be overridden by anyone
    Compliance,
}

impl std::fmt::Display for RetentionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetentionMode::Governance => write!(f, "GOVERNANCE"),
            RetentionMode::Compliance => write!(f, "COMPLIANCE"),
        }
    }
}

impl std::str::FromStr for RetentionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GOVERNANCE" => Ok(RetentionMode::Governance),
            "COMPLIANCE" => Ok(RetentionMode::Compliance),
            _ => Err(format!("Invalid retention mode: {}", s)),
        }
    }
}

// ============================================================================
// Object Retention (Object Level)
// ============================================================================

/// Object retention settings
///
/// Applied to individual objects to prevent deletion/modification
/// until the retain-until date has passed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "Retention")]
pub struct ObjectRetention {
    /// Retention mode
    #[serde(rename = "Mode")]
    pub mode: RetentionMode,

    /// Date until which the object is locked
    #[serde(rename = "RetainUntilDate")]
    pub retain_until_date: String, // ISO 8601 format
}

impl ObjectRetention {
    /// Create a new object retention
    pub fn new(mode: RetentionMode, retain_until: DateTime<Utc>) -> Self {
        Self {
            mode,
            retain_until_date: retain_until.to_rfc3339(),
        }
    }

    /// Get retain until date as DateTime
    pub fn retain_until(&self) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(&self.retain_until_date)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    }

    /// Check if retention has expired
    pub fn is_expired(&self) -> bool {
        self.retain_until()
            .map(|dt| Utc::now() > dt)
            .unwrap_or(true)
    }

    /// Check if object can be deleted
    pub fn can_delete(&self, has_governance_bypass: bool) -> bool {
        if self.is_expired() {
            return true;
        }

        match self.mode {
            RetentionMode::Governance => has_governance_bypass,
            RetentionMode::Compliance => false,
        }
    }

    /// Check if object can be modified
    pub fn can_modify(&self, has_governance_bypass: bool) -> bool {
        self.can_delete(has_governance_bypass)
    }
}

// ============================================================================
// Legal Hold (Object Level)
// ============================================================================

/// Legal Hold status for an object
///
/// Legal holds are independent of retention settings and can be applied
/// or removed at any time. Objects with legal hold cannot be deleted
/// regardless of retention settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "LegalHold")]
pub struct ObjectLegalHold {
    /// Legal hold status: ON or OFF
    #[serde(rename = "Status")]
    pub status: LegalHoldStatus,
}

/// Legal hold status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LegalHoldStatus {
    /// Legal hold is active
    On,
    /// Legal hold is not active
    Off,
}

impl std::fmt::Display for LegalHoldStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LegalHoldStatus::On => write!(f, "ON"),
            LegalHoldStatus::Off => write!(f, "OFF"),
        }
    }
}

impl std::str::FromStr for LegalHoldStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "ON" => Ok(LegalHoldStatus::On),
            "OFF" => Ok(LegalHoldStatus::Off),
            _ => Err(format!("Invalid legal hold status: {}", s)),
        }
    }
}

impl ObjectLegalHold {
    /// Create a new legal hold with ON status
    pub fn on() -> Self {
        Self {
            status: LegalHoldStatus::On,
        }
    }

    /// Create a new legal hold with OFF status
    pub fn off() -> Self {
        Self {
            status: LegalHoldStatus::Off,
        }
    }

    /// Check if legal hold is active
    pub fn is_active(&self) -> bool {
        self.status == LegalHoldStatus::On
    }
}

// ============================================================================
// Object Lock State (Combined)
// ============================================================================

/// Combined Object Lock state for an object
///
/// Tracks both retention settings and legal hold status.
#[derive(Debug, Clone, Default)]
pub struct ObjectLockState {
    /// Retention settings (if any)
    pub retention: Option<ObjectRetention>,

    /// Legal hold status (if any)
    pub legal_hold: Option<ObjectLegalHold>,
}

impl ObjectLockState {
    /// Check if object can be deleted
    pub fn can_delete(&self, has_governance_bypass: bool) -> bool {
        // Legal hold always blocks deletion
        if let Some(ref hold) = self.legal_hold {
            if hold.is_active() {
                return false;
            }
        }

        // Check retention
        if let Some(ref retention) = self.retention {
            if !retention.can_delete(has_governance_bypass) {
                return false;
            }
        }

        true
    }

    /// Check if object can be modified
    pub fn can_modify(&self, has_governance_bypass: bool) -> bool {
        self.can_delete(has_governance_bypass)
    }

    /// Check if object is locked
    pub fn is_locked(&self) -> bool {
        !self.can_delete(false)
    }

    /// Get lock reason for error messages
    pub fn lock_reason(&self) -> Option<String> {
        if let Some(ref hold) = self.legal_hold {
            if hold.is_active() {
                return Some("Object is under legal hold".to_string());
            }
        }

        if let Some(ref retention) = self.retention {
            if !retention.is_expired() {
                return Some(format!(
                    "Object is locked in {} mode until {}",
                    retention.mode, retention.retain_until_date
                ));
            }
        }

        None
    }
}

// ============================================================================
// Validation
// ============================================================================

impl ObjectLockConfiguration {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ObjectLockError> {
        // Check if enabled
        if let Some(ref enabled) = self.object_lock_enabled {
            if enabled != "Enabled" {
                return Err(ObjectLockError::InvalidConfiguration(
                    "ObjectLockEnabled must be 'Enabled'".to_string(),
                ));
            }
        }

        // Validate rule if present
        if let Some(ref rule) = self.rule {
            rule.default_retention.validate()?;
        }

        Ok(())
    }

    /// Check if Object Lock is enabled
    pub fn is_enabled(&self) -> bool {
        self.object_lock_enabled.as_deref() == Some("Enabled")
    }

    /// Create enabled configuration with default retention
    pub fn enabled_with_retention(
        mode: RetentionMode,
        days: Option<u32>,
        years: Option<u32>,
    ) -> Self {
        Self {
            object_lock_enabled: Some("Enabled".to_string()),
            rule: Some(ObjectLockRule {
                default_retention: DefaultRetention {
                    mode: Some(mode),
                    days,
                    years,
                },
            }),
        }
    }

    /// Create enabled configuration without default retention
    pub fn enabled() -> Self {
        Self {
            object_lock_enabled: Some("Enabled".to_string()),
            rule: None,
        }
    }
}

impl DefaultRetention {
    /// Maximum retention period in days (100 years)
    pub const MAX_DAYS: u32 = 36500;

    /// Maximum retention period in years
    pub const MAX_YEARS: u32 = 100;

    /// Validate the default retention
    pub fn validate(&self) -> Result<(), ObjectLockError> {
        // Must have either days or years, but not both
        match (&self.days, &self.years) {
            (Some(_), Some(_)) => {
                return Err(ObjectLockError::InvalidConfiguration(
                    "Cannot specify both Days and Years".to_string(),
                ));
            }
            (None, None) => {
                return Err(ObjectLockError::InvalidConfiguration(
                    "Must specify either Days or Years".to_string(),
                ));
            }
            _ => {}
        }

        // Validate days range
        if let Some(days) = self.days {
            if days == 0 || days > Self::MAX_DAYS {
                return Err(ObjectLockError::InvalidConfiguration(format!(
                    "Days must be between 1 and {}",
                    Self::MAX_DAYS
                )));
            }
        }

        // Validate years range
        if let Some(years) = self.years {
            if years == 0 || years > Self::MAX_YEARS {
                return Err(ObjectLockError::InvalidConfiguration(format!(
                    "Years must be between 1 and {}",
                    Self::MAX_YEARS
                )));
            }
        }

        // Mode is required
        if self.mode.is_none() {
            return Err(ObjectLockError::InvalidConfiguration(
                "Mode is required".to_string(),
            ));
        }

        Ok(())
    }

    /// Calculate retain until date from now
    pub fn calculate_retain_until(&self) -> DateTime<Utc> {
        let now = Utc::now();

        if let Some(days) = self.days {
            now + Duration::days(days as i64)
        } else if let Some(years) = self.years {
            now + Duration::days(years as i64 * 365)
        } else {
            now
        }
    }

    /// Create retention from these default settings
    pub fn to_retention(&self) -> Option<ObjectRetention> {
        self.mode
            .map(|mode| ObjectRetention::new(mode, self.calculate_retain_until()))
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Object Lock errors
#[derive(Debug, Clone)]
pub enum ObjectLockError {
    /// Invalid configuration
    InvalidConfiguration(String),
    /// Object is locked
    ObjectLocked(String),
    /// Object Lock not enabled on bucket
    NotEnabled,
    /// Cannot disable Object Lock
    CannotDisable,
    /// Access denied
    AccessDenied(String),
}

impl std::fmt::Display for ObjectLockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfiguration(msg) => {
                write!(f, "Invalid Object Lock configuration: {}", msg)
            }
            Self::ObjectLocked(msg) => write!(f, "Object is locked: {}", msg),
            Self::NotEnabled => write!(f, "Object Lock is not enabled on this bucket"),
            Self::CannotDisable => write!(f, "Object Lock cannot be disabled once enabled"),
            Self::AccessDenied(msg) => write!(f, "Access denied: {}", msg),
        }
    }
}

impl std::error::Error for ObjectLockError {}

// ============================================================================
// XML Serialization
// ============================================================================

impl ObjectLockConfiguration {
    /// Parse from XML
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        quick_xml::de::from_str(xml).map_err(|e| format!("Invalid Object Lock XML: {}", e))
    }

    /// Serialize to XML
    pub fn to_xml(&self) -> Result<String, String> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        let body = quick_xml::se::to_string(self)
            .map_err(|e| format!("Failed to serialize Object Lock: {}", e))?;
        xml.push_str(&body);

        Ok(xml)
    }
}

impl ObjectRetention {
    /// Parse from XML
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        quick_xml::de::from_str(xml).map_err(|e| format!("Invalid Retention XML: {}", e))
    }

    /// Serialize to XML
    pub fn to_xml(&self) -> Result<String, String> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        let body = quick_xml::se::to_string(self)
            .map_err(|e| format!("Failed to serialize Retention: {}", e))?;
        xml.push_str(&body);

        Ok(xml)
    }
}

impl ObjectLegalHold {
    /// Parse from XML
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        quick_xml::de::from_str(xml).map_err(|e| format!("Invalid Legal Hold XML: {}", e))
    }

    /// Serialize to XML
    pub fn to_xml(&self) -> Result<String, String> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        let body = quick_xml::se::to_string(self)
            .map_err(|e| format!("Failed to serialize Legal Hold: {}", e))?;
        xml.push_str(&body);

        Ok(xml)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_mode_parsing() {
        assert_eq!(
            "GOVERNANCE".parse::<RetentionMode>().unwrap(),
            RetentionMode::Governance
        );
        assert_eq!(
            "COMPLIANCE".parse::<RetentionMode>().unwrap(),
            RetentionMode::Compliance
        );
        assert_eq!(
            "governance".parse::<RetentionMode>().unwrap(),
            RetentionMode::Governance
        );
        assert!("INVALID".parse::<RetentionMode>().is_err());
    }

    #[test]
    fn test_legal_hold_status() {
        let hold = ObjectLegalHold::on();
        assert!(hold.is_active());

        let hold = ObjectLegalHold::off();
        assert!(!hold.is_active());
    }

    #[test]
    fn test_retention_expiry() {
        let past = Utc::now() - Duration::days(1);
        let retention = ObjectRetention::new(RetentionMode::Governance, past);
        assert!(retention.is_expired());
        assert!(retention.can_delete(false));

        let future = Utc::now() + Duration::days(1);
        let retention = ObjectRetention::new(RetentionMode::Governance, future);
        assert!(!retention.is_expired());
        assert!(!retention.can_delete(false));
        assert!(retention.can_delete(true)); // With governance bypass

        let retention = ObjectRetention::new(RetentionMode::Compliance, future);
        assert!(!retention.can_delete(true)); // Compliance mode - no bypass
    }

    #[test]
    fn test_object_lock_state() {
        let mut state = ObjectLockState::default();
        assert!(!state.is_locked());
        assert!(state.can_delete(false));

        // Add legal hold
        state.legal_hold = Some(ObjectLegalHold::on());
        assert!(state.is_locked());
        assert!(!state.can_delete(false));
        assert!(!state.can_delete(true)); // Legal hold blocks even with bypass

        // Remove legal hold, add retention
        state.legal_hold = None;
        state.retention = Some(ObjectRetention::new(
            RetentionMode::Governance,
            Utc::now() + Duration::days(30),
        ));
        assert!(state.is_locked());
        assert!(!state.can_delete(false));
        assert!(state.can_delete(true)); // Governance allows bypass
    }

    #[test]
    fn test_default_retention_validation() {
        // Valid with days
        let dr = DefaultRetention {
            mode: Some(RetentionMode::Governance),
            days: Some(30),
            years: None,
        };
        assert!(dr.validate().is_ok());

        // Valid with years
        let dr = DefaultRetention {
            mode: Some(RetentionMode::Compliance),
            days: None,
            years: Some(7),
        };
        assert!(dr.validate().is_ok());

        // Invalid: both days and years
        let dr = DefaultRetention {
            mode: Some(RetentionMode::Governance),
            days: Some(30),
            years: Some(1),
        };
        assert!(dr.validate().is_err());

        // Invalid: neither days nor years
        let dr = DefaultRetention {
            mode: Some(RetentionMode::Governance),
            days: None,
            years: None,
        };
        assert!(dr.validate().is_err());

        // Invalid: days out of range
        let dr = DefaultRetention {
            mode: Some(RetentionMode::Governance),
            days: Some(40000),
            years: None,
        };
        assert!(dr.validate().is_err());
    }

    #[test]
    fn test_configuration_xml() {
        let config = ObjectLockConfiguration::enabled_with_retention(
            RetentionMode::Compliance,
            Some(365),
            None,
        );

        let xml = config.to_xml().unwrap();
        assert!(xml.contains("COMPLIANCE"));
        assert!(xml.contains("365"));

        let parsed = ObjectLockConfiguration::from_xml(&xml).unwrap();
        assert!(parsed.is_enabled());
    }
}
