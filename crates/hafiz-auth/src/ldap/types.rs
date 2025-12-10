//! LDAP/Active Directory types and configuration
//!
//! Supports:
//! - LDAP (OpenLDAP, 389 Directory Server)
//! - Active Directory
//! - User/Group mapping
//! - TLS/STARTTLS connections

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// LDAP Configuration
// ============================================================================

/// LDAP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LdapConfig {
    /// Enable LDAP authentication
    #[serde(default)]
    pub enabled: bool,

    /// LDAP server URL (ldap:// or ldaps://)
    /// Example: "ldap://ldap.example.com:389" or "ldaps://ldap.example.com:636"
    pub server_url: String,

    /// Use STARTTLS for connection upgrade
    #[serde(default)]
    pub start_tls: bool,

    /// Skip TLS certificate verification (not recommended for production)
    #[serde(default)]
    pub skip_tls_verify: bool,

    /// Bind DN for LDAP queries (service account)
    /// Example: "cn=admin,dc=example,dc=com"
    pub bind_dn: String,

    /// Bind password
    pub bind_password: String,

    /// Base DN for user searches
    /// Example: "ou=users,dc=example,dc=com"
    pub user_base_dn: String,

    /// User search filter
    /// Use {username} as placeholder
    /// Example: "(uid={username})" or "(sAMAccountName={username})"
    #[serde(default = "default_user_filter")]
    pub user_filter: String,

    /// Base DN for group searches
    /// Example: "ou=groups,dc=example,dc=com"
    #[serde(default)]
    pub group_base_dn: Option<String>,

    /// Group search filter
    /// Use {dn} as placeholder for user DN
    /// Example: "(member={dn})" or "(memberUid={username})"
    #[serde(default)]
    pub group_filter: Option<String>,

    /// LDAP attribute mappings
    #[serde(default)]
    pub attribute_mappings: AttributeMappings,

    /// Group to policy mappings
    #[serde(default)]
    pub group_policies: HashMap<String, Vec<String>>,

    /// Default policies for authenticated users without group mapping
    #[serde(default)]
    pub default_policies: Vec<String>,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Cache TTL for authenticated users (seconds)
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_seconds: u64,

    /// LDAP server type hint
    #[serde(default)]
    pub server_type: LdapServerType,
}

fn default_user_filter() -> String {
    "(uid={username})".to_string()
}

fn default_timeout() -> u64 {
    10
}

fn default_cache_ttl() -> u64 {
    300 // 5 minutes
}

impl Default for LdapConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            server_url: "ldap://localhost:389".to_string(),
            start_tls: false,
            skip_tls_verify: false,
            bind_dn: String::new(),
            bind_password: String::new(),
            user_base_dn: String::new(),
            user_filter: default_user_filter(),
            group_base_dn: None,
            group_filter: None,
            attribute_mappings: AttributeMappings::default(),
            group_policies: HashMap::new(),
            default_policies: vec!["readonly".to_string()],
            timeout_seconds: default_timeout(),
            cache_ttl_seconds: default_cache_ttl(),
            server_type: LdapServerType::default(),
        }
    }
}

/// LDAP server type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LdapServerType {
    /// Generic LDAP server
    #[default]
    Ldap,
    /// Microsoft Active Directory
    ActiveDirectory,
    /// OpenLDAP
    OpenLdap,
    /// 389 Directory Server
    Directory389,
}

impl LdapServerType {
    /// Get default user filter for this server type
    pub fn default_user_filter(&self) -> &'static str {
        match self {
            LdapServerType::ActiveDirectory => "(sAMAccountName={username})",
            _ => "(uid={username})",
        }
    }

    /// Get default group filter for this server type
    pub fn default_group_filter(&self) -> &'static str {
        match self {
            LdapServerType::ActiveDirectory => "(member={dn})",
            _ => "(memberUid={username})",
        }
    }
}

/// LDAP attribute mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AttributeMappings {
    /// Username attribute
    #[serde(default = "default_username_attr")]
    pub username: String,

    /// Email attribute
    #[serde(default = "default_email_attr")]
    pub email: String,

    /// Display name attribute
    #[serde(default = "default_display_name_attr")]
    pub display_name: String,

    /// Group name attribute
    #[serde(default = "default_group_name_attr")]
    pub group_name: String,

    /// Member attribute (for group membership)
    #[serde(default = "default_member_attr")]
    pub member: String,
}

fn default_username_attr() -> String {
    "uid".to_string()
}

fn default_email_attr() -> String {
    "mail".to_string()
}

fn default_display_name_attr() -> String {
    "cn".to_string()
}

fn default_group_name_attr() -> String {
    "cn".to_string()
}

fn default_member_attr() -> String {
    "member".to_string()
}

impl Default for AttributeMappings {
    fn default() -> Self {
        Self {
            username: default_username_attr(),
            email: default_email_attr(),
            display_name: default_display_name_attr(),
            group_name: default_group_name_attr(),
            member: default_member_attr(),
        }
    }
}

impl AttributeMappings {
    /// Get Active Directory default mappings
    pub fn active_directory() -> Self {
        Self {
            username: "sAMAccountName".to_string(),
            email: "mail".to_string(),
            display_name: "displayName".to_string(),
            group_name: "cn".to_string(),
            member: "member".to_string(),
        }
    }

    /// Get OpenLDAP default mappings
    pub fn openldap() -> Self {
        Self {
            username: "uid".to_string(),
            email: "mail".to_string(),
            display_name: "cn".to_string(),
            group_name: "cn".to_string(),
            member: "memberUid".to_string(),
        }
    }
}

// ============================================================================
// LDAP User
// ============================================================================

/// LDAP user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapUser {
    /// User DN (Distinguished Name)
    pub dn: String,

    /// Username
    pub username: String,

    /// Email address
    pub email: Option<String>,

    /// Display name
    pub display_name: Option<String>,

    /// Group memberships
    pub groups: Vec<String>,

    /// Mapped policies based on group membership
    pub policies: Vec<String>,

    /// Raw LDAP attributes
    #[serde(default)]
    pub attributes: HashMap<String, Vec<String>>,
}

impl LdapUser {
    /// Check if user is admin based on policies
    pub fn is_admin(&self) -> bool {
        self.policies.contains(&"admin".to_string())
    }

    /// Get first value of an attribute
    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .get(name)
            .and_then(|v| v.first())
            .map(|s| s.as_str())
    }
}

// ============================================================================
// LDAP Group
// ============================================================================

/// LDAP group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapGroup {
    /// Group DN
    pub dn: String,

    /// Group name
    pub name: String,

    /// Group members (DNs or usernames)
    pub members: Vec<String>,

    /// Description
    pub description: Option<String>,
}

// ============================================================================
// Authentication Result
// ============================================================================

/// LDAP authentication result
#[derive(Debug, Clone)]
pub enum LdapAuthResult {
    /// Authentication successful
    Success(LdapUser),
    /// Invalid credentials
    InvalidCredentials,
    /// User not found
    UserNotFound,
    /// Account disabled/locked
    AccountDisabled,
    /// Connection error
    ConnectionError(String),
    /// Configuration error
    ConfigError(String),
}

impl LdapAuthResult {
    pub fn is_success(&self) -> bool {
        matches!(self, LdapAuthResult::Success(_))
    }

    pub fn user(&self) -> Option<&LdapUser> {
        match self {
            LdapAuthResult::Success(user) => Some(user),
            _ => None,
        }
    }
}

// ============================================================================
// LDAP Status
// ============================================================================

/// LDAP connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapStatus {
    /// Whether LDAP is enabled
    pub enabled: bool,

    /// Whether connection is healthy
    pub connected: bool,

    /// Server URL
    pub server_url: String,

    /// Server type
    pub server_type: LdapServerType,

    /// Last successful connection time
    pub last_connection: Option<String>,

    /// Error message if connection failed
    pub error: Option<String>,

    /// Number of cached users
    pub cached_users: usize,
}

// ============================================================================
// Admin API Types
// ============================================================================

/// Request to test LDAP connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestLdapConnectionRequest {
    pub server_url: String,
    pub bind_dn: String,
    pub bind_password: String,
    #[serde(default)]
    pub start_tls: bool,
    #[serde(default)]
    pub skip_tls_verify: bool,
}

/// Response from LDAP connection test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestLdapConnectionResponse {
    pub success: bool,
    pub message: String,
    pub server_info: Option<LdapServerInfo>,
}

/// LDAP server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdapServerInfo {
    pub vendor: Option<String>,
    pub version: Option<String>,
    pub naming_contexts: Vec<String>,
    pub supported_ldap_version: Vec<String>,
}

/// Request to test LDAP user search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestLdapSearchRequest {
    pub username: String,
}

/// Response from LDAP user search test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestLdapSearchResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<LdapUser>,
}

/// Request to update LDAP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLdapConfigRequest {
    #[serde(flatten)]
    pub config: LdapConfig,
}

// ============================================================================
// Helper Functions
// ============================================================================

impl LdapConfig {
    /// Build user search filter with username substitution
    pub fn build_user_filter(&self, username: &str) -> String {
        self.user_filter.replace("{username}", username)
    }

    /// Build group search filter with DN/username substitution
    pub fn build_group_filter(&self, user_dn: &str, username: &str) -> Option<String> {
        self.group_filter
            .as_ref()
            .map(|f| f.replace("{dn}", user_dn).replace("{username}", username))
    }

    /// Map groups to policies
    pub fn map_groups_to_policies(&self, groups: &[String]) -> Vec<String> {
        let mut policies = Vec::new();

        for group in groups {
            if let Some(group_policies) = self.group_policies.get(group) {
                for policy in group_policies {
                    if !policies.contains(policy) {
                        policies.push(policy.clone());
                    }
                }
            }
        }

        // Add default policies if no group mappings matched
        if policies.is_empty() {
            policies = self.default_policies.clone();
        }

        policies
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }

        if self.server_url.is_empty() {
            return Err("Server URL is required".to_string());
        }

        if !self.server_url.starts_with("ldap://") && !self.server_url.starts_with("ldaps://") {
            return Err("Server URL must start with ldap:// or ldaps://".to_string());
        }

        if self.bind_dn.is_empty() {
            return Err("Bind DN is required".to_string());
        }

        if self.user_base_dn.is_empty() {
            return Err("User base DN is required".to_string());
        }

        if self.user_filter.is_empty() {
            return Err("User filter is required".to_string());
        }

        if !self.user_filter.contains("{username}") {
            return Err("User filter must contain {username} placeholder".to_string());
        }

        Ok(())
    }

    /// Create configuration from server type with sensible defaults
    pub fn from_server_type(server_type: LdapServerType, server_url: &str) -> Self {
        let mut config = Self {
            enabled: true,
            server_url: server_url.to_string(),
            server_type,
            ..Default::default()
        };

        match server_type {
            LdapServerType::ActiveDirectory => {
                config.user_filter = "(sAMAccountName={username})".to_string();
                config.group_filter = Some("(member={dn})".to_string());
                config.attribute_mappings = AttributeMappings::active_directory();
            }
            LdapServerType::OpenLdap | LdapServerType::Directory389 => {
                config.user_filter = "(uid={username})".to_string();
                config.group_filter = Some("(memberUid={username})".to_string());
                config.attribute_mappings = AttributeMappings::openldap();
            }
            LdapServerType::Ldap => {
                // Use defaults
            }
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_filter_building() {
        let config = LdapConfig {
            user_filter: "(uid={username})".to_string(),
            ..Default::default()
        };

        assert_eq!(config.build_user_filter("john"), "(uid=john)");
    }

    #[test]
    fn test_group_policy_mapping() {
        let mut group_policies = HashMap::new();
        group_policies.insert("admins".to_string(), vec!["admin".to_string()]);
        group_policies.insert("developers".to_string(), vec!["readwrite".to_string()]);

        let config = LdapConfig {
            group_policies,
            default_policies: vec!["readonly".to_string()],
            ..Default::default()
        };

        // User in admins group
        let policies = config.map_groups_to_policies(&["admins".to_string()]);
        assert!(policies.contains(&"admin".to_string()));

        // User in unknown group gets default
        let policies = config.map_groups_to_policies(&["unknown".to_string()]);
        assert!(policies.contains(&"readonly".to_string()));
    }

    #[test]
    fn test_config_validation() {
        let mut config = LdapConfig::default();
        config.enabled = true;

        // Should fail - empty server URL
        assert!(config.validate().is_err());

        config.server_url = "ldap://localhost:389".to_string();
        config.bind_dn = "cn=admin,dc=example,dc=com".to_string();
        config.user_base_dn = "ou=users,dc=example,dc=com".to_string();

        // Should pass now
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_active_directory_defaults() {
        let config = LdapConfig::from_server_type(
            LdapServerType::ActiveDirectory,
            "ldaps://dc.example.com:636",
        );

        assert_eq!(config.user_filter, "(sAMAccountName={username})");
        assert_eq!(config.attribute_mappings.username, "sAMAccountName");
    }
}
