//! LDAP Client implementation
//!
//! Handles LDAP connections, authentication, and user/group queries.
//! Supports LDAP, LDAPS (SSL), and STARTTLS connections.

use crate::ldap::types::*;
use ldap3::{Ldap, LdapConnAsync, LdapConnSettings, Scope, SearchEntry};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// LDAP Client for authentication and user management
pub struct LdapClient {
    config: LdapConfig,
    cache: Arc<RwLock<UserCache>>,
}

/// User cache for reducing LDAP queries
struct UserCache {
    users: HashMap<String, CachedUser>,
    ttl: Duration,
}

struct CachedUser {
    user: LdapUser,
    cached_at: Instant,
}

impl LdapClient {
    /// Create a new LDAP client
    pub fn new(config: LdapConfig) -> Self {
        let cache = Arc::new(RwLock::new(UserCache {
            users: HashMap::new(),
            ttl: Duration::from_secs(config.cache_ttl_seconds),
        }));

        Self { config, cache }
    }

    /// Authenticate a user with username and password
    pub async fn authenticate(&self, username: &str, password: &str) -> LdapAuthResult {
        if !self.config.enabled {
            return LdapAuthResult::ConfigError("LDAP is not enabled".to_string());
        }

        if let Err(e) = self.config.validate() {
            return LdapAuthResult::ConfigError(e);
        }

        // Check cache first for user info (not auth)
        if let Some(cached_user) = self.get_cached_user(username).await {
            debug!("Found cached user info for: {}", username);
            // Still need to verify password against LDAP
        }

        // Perform LDAP authentication
        match self.ldap_authenticate(username, password).await {
            Ok(user) => {
                // Cache the user
                self.cache_user(user.clone()).await;
                LdapAuthResult::Success(user)
            }
            Err(e) => e,
        }
    }

    /// Search for a user by username
    pub async fn search_user(&self, username: &str) -> Result<Option<LdapUser>, String> {
        if !self.config.enabled {
            return Err("LDAP is not enabled".to_string());
        }

        // Check cache first
        if let Some(user) = self.get_cached_user(username).await {
            return Ok(Some(user));
        }

        // Search LDAP
        self.ldap_search_user(username).await
    }

    /// Get user's groups
    pub async fn get_user_groups(
        &self,
        user_dn: &str,
        username: &str,
    ) -> Result<Vec<String>, String> {
        if self.config.group_base_dn.is_none() {
            return Ok(Vec::new());
        }

        self.ldap_get_groups(user_dn, username).await
    }

    /// Test LDAP connection
    pub async fn test_connection(&self) -> TestLdapConnectionResponse {
        match self.ldap_test_connection().await {
            Ok(info) => TestLdapConnectionResponse {
                success: true,
                message: "Connection successful".to_string(),
                server_info: Some(info),
            },
            Err(e) => TestLdapConnectionResponse {
                success: false,
                message: e,
                server_info: None,
            },
        }
    }

    /// Get LDAP status
    pub async fn get_status(&self) -> LdapStatus {
        let cached_users = self.cache.read().await.users.len();

        let (connected, error) = match self.ldap_test_connection().await {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e)),
        };

        LdapStatus {
            enabled: self.config.enabled,
            connected,
            server_url: self.config.server_url.clone(),
            server_type: self.config.server_type,
            last_connection: if connected {
                Some(chrono::Utc::now().to_rfc3339())
            } else {
                None
            },
            error,
            cached_users,
        }
    }

    /// Clear user cache
    pub async fn clear_cache(&self) {
        self.cache.write().await.users.clear();
        info!("LDAP user cache cleared");
    }

    // =========================================================================
    // Private methods - Real LDAP implementation
    // =========================================================================

    async fn get_cached_user(&self, username: &str) -> Option<LdapUser> {
        let cache = self.cache.read().await;

        if let Some(cached) = cache.users.get(username) {
            if cached.cached_at.elapsed() < cache.ttl {
                return Some(cached.user.clone());
            }
        }

        None
    }

    async fn cache_user(&self, user: LdapUser) {
        let mut cache = self.cache.write().await;
        cache.users.insert(
            user.username.clone(),
            CachedUser {
                user,
                cached_at: Instant::now(),
            },
        );
    }

    /// Create LDAP connection with proper TLS settings
    async fn create_connection(&self) -> Result<(LdapConnAsync, Ldap), String> {
        let settings = LdapConnSettings::new()
            .set_conn_timeout(Duration::from_secs(self.config.timeout_seconds))
            .set_starttls(self.config.start_tls);

        debug!("Connecting to LDAP server: {}", self.config.server_url);

        LdapConnAsync::with_settings(settings, &self.config.server_url)
            .await
            .map_err(|e| format!("Failed to connect to LDAP server: {}", e))
    }

    /// Perform LDAP authentication
    async fn ldap_authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<LdapUser, LdapAuthResult> {
        // Step 1: Connect and bind with service account
        let (conn, mut ldap) = self
            .create_connection()
            .await
            .map_err(|e| LdapAuthResult::ConnectionError(e))?;

        ldap3::drive!(conn);

        // Bind with service account
        let result = ldap
            .simple_bind(&self.config.bind_dn, &self.config.bind_password)
            .await
            .map_err(|e| LdapAuthResult::ConnectionError(format!("Service bind failed: {}", e)))?;

        if result.rc != 0 {
            return Err(LdapAuthResult::ConfigError(format!(
                "Service account bind failed with code: {}",
                result.rc
            )));
        }

        // Step 2: Search for user
        let filter = self.config.build_user_filter(username);
        let attrs = vec![
            &self.config.attribute_mappings.username as &str,
            &self.config.attribute_mappings.email,
            &self.config.attribute_mappings.display_name,
            "dn",
        ];

        debug!("Searching for user with filter: {}", filter);

        let (rs, _res) = ldap
            .search(&self.config.user_base_dn, Scope::Subtree, &filter, attrs)
            .await
            .map_err(|e| LdapAuthResult::ConnectionError(format!("User search failed: {}", e)))?
            .success()
            .map_err(|e| LdapAuthResult::ConnectionError(format!("User search error: {}", e)))?;

        if rs.is_empty() {
            let _ = ldap.unbind().await;
            return Err(LdapAuthResult::UserNotFound);
        }

        let entry = SearchEntry::construct(rs.into_iter().next().unwrap());
        let user_dn = entry.dn.clone();

        debug!("Found user DN: {}", user_dn);

        // Step 3: Verify user password by binding as the user
        let (conn2, mut ldap2) = self
            .create_connection()
            .await
            .map_err(|e| LdapAuthResult::ConnectionError(e))?;

        ldap3::drive!(conn2);

        let user_bind = ldap2
            .simple_bind(&user_dn, password)
            .await
            .map_err(|e| LdapAuthResult::ConnectionError(format!("User bind failed: {}", e)))?;

        if user_bind.rc != 0 {
            let _ = ldap2.unbind().await;
            let _ = ldap.unbind().await;

            // RC 49 = Invalid credentials
            if user_bind.rc == 49 {
                return Err(LdapAuthResult::InvalidCredentials);
            }
            // RC 53 = Account disabled/locked
            if user_bind.rc == 53 {
                return Err(LdapAuthResult::AccountDisabled);
            }

            return Err(LdapAuthResult::InvalidCredentials);
        }

        let _ = ldap2.unbind().await;

        // Step 4: Get user groups
        let groups = self
            .ldap_get_groups_with_connection(&mut ldap, &user_dn, username)
            .await
            .unwrap_or_default();

        let _ = ldap.unbind().await;

        // Step 5: Map groups to policies
        let policies = self.config.map_groups_to_policies(&groups);

        // Build user object from LDAP attributes
        let user = LdapUser {
            dn: user_dn,
            username: get_first_attr(&entry, &self.config.attribute_mappings.username)
                .unwrap_or_else(|| username.to_string()),
            email: get_first_attr(&entry, &self.config.attribute_mappings.email),
            display_name: get_first_attr(&entry, &self.config.attribute_mappings.display_name),
            groups,
            policies,
            attributes: entry.attrs.into_iter().collect(),
        };

        Ok(user)
    }

    async fn ldap_search_user(&self, username: &str) -> Result<Option<LdapUser>, String> {
        let (conn, mut ldap) = self.create_connection().await?;
        ldap3::drive!(conn);

        // Bind with service account
        let result = ldap
            .simple_bind(&self.config.bind_dn, &self.config.bind_password)
            .await
            .map_err(|e| format!("Bind failed: {}", e))?;

        if result.rc != 0 {
            return Err(format!("Bind failed with code: {}", result.rc));
        }

        let filter = self.config.build_user_filter(username);
        let attrs = vec![
            &self.config.attribute_mappings.username as &str,
            &self.config.attribute_mappings.email,
            &self.config.attribute_mappings.display_name,
        ];

        let (rs, _res) = ldap
            .search(&self.config.user_base_dn, Scope::Subtree, &filter, attrs)
            .await
            .map_err(|e| format!("Search failed: {}", e))?
            .success()
            .map_err(|e| format!("Search error: {}", e))?;

        let _ = ldap.unbind().await;

        if rs.is_empty() {
            return Ok(None);
        }

        let entry = SearchEntry::construct(rs.into_iter().next().unwrap());
        let user_dn = entry.dn.clone();

        let groups = self
            .ldap_get_groups(&user_dn, username)
            .await
            .unwrap_or_default();
        let policies = self.config.map_groups_to_policies(&groups);

        Ok(Some(LdapUser {
            dn: user_dn,
            username: get_first_attr(&entry, &self.config.attribute_mappings.username)
                .unwrap_or_else(|| username.to_string()),
            email: get_first_attr(&entry, &self.config.attribute_mappings.email),
            display_name: get_first_attr(&entry, &self.config.attribute_mappings.display_name),
            groups,
            policies,
            attributes: entry.attrs.into_iter().collect(),
        }))
    }

    async fn ldap_get_groups(&self, user_dn: &str, username: &str) -> Result<Vec<String>, String> {
        let (conn, mut ldap) = self.create_connection().await?;
        ldap3::drive!(conn);

        let result = ldap
            .simple_bind(&self.config.bind_dn, &self.config.bind_password)
            .await
            .map_err(|e| format!("Bind failed: {}", e))?;

        if result.rc != 0 {
            return Err(format!("Bind failed with code: {}", result.rc));
        }

        let groups = self
            .ldap_get_groups_with_connection(&mut ldap, user_dn, username)
            .await?;

        let _ = ldap.unbind().await;
        Ok(groups)
    }

    async fn ldap_get_groups_with_connection(
        &self,
        ldap: &mut Ldap,
        user_dn: &str,
        username: &str,
    ) -> Result<Vec<String>, String> {
        let group_base = match &self.config.group_base_dn {
            Some(base) => base,
            None => return Ok(Vec::new()),
        };

        let filter = match self.config.build_group_filter(user_dn, username) {
            Some(f) => f,
            None => return Ok(Vec::new()),
        };

        debug!("Searching groups with filter: {}", filter);

        let (rs, _res) = ldap
            .search(
                group_base,
                Scope::Subtree,
                &filter,
                vec![&self.config.attribute_mappings.group_name as &str],
            )
            .await
            .map_err(|e| format!("Group search failed: {}", e))?
            .success()
            .map_err(|e| format!("Group search error: {}", e))?;

        let mut groups = Vec::new();
        for result in rs {
            let entry = SearchEntry::construct(result);
            if let Some(name) = get_first_attr(&entry, &self.config.attribute_mappings.group_name) {
                groups.push(name);
            }
        }

        debug!("Found {} groups for user", groups.len());
        Ok(groups)
    }

    async fn ldap_test_connection(&self) -> Result<LdapServerInfo, String> {
        if !self.config.enabled {
            return Err("LDAP is not enabled".to_string());
        }

        let (conn, mut ldap) = self.create_connection().await?;
        ldap3::drive!(conn);

        // Test bind with service account
        let result = ldap
            .simple_bind(&self.config.bind_dn, &self.config.bind_password)
            .await
            .map_err(|e| format!("Bind failed: {}", e))?;

        if result.rc != 0 {
            return Err(format!("Bind failed with code: {}", result.rc));
        }

        // Query root DSE for server info
        let (rs, _res) = ldap
            .search(
                "",
                Scope::Base,
                "(objectClass=*)",
                vec![
                    "vendorName",
                    "vendorVersion",
                    "namingContexts",
                    "supportedLDAPVersion",
                ],
            )
            .await
            .map_err(|e| format!("Root DSE query failed: {}", e))?
            .success()
            .map_err(|e| format!("Root DSE error: {}", e))?;

        let _ = ldap.unbind().await;

        let info = if let Some(result) = rs.into_iter().next() {
            let entry = SearchEntry::construct(result);
            LdapServerInfo {
                vendor: get_first_attr(&entry, "vendorName"),
                version: get_first_attr(&entry, "vendorVersion"),
                naming_contexts: entry
                    .attrs
                    .get("namingContexts")
                    .cloned()
                    .unwrap_or_default(),
                supported_ldap_version: entry
                    .attrs
                    .get("supportedLDAPVersion")
                    .cloned()
                    .unwrap_or_default(),
            }
        } else {
            LdapServerInfo {
                vendor: None,
                version: None,
                naming_contexts: vec![],
                supported_ldap_version: vec!["3".to_string()],
            }
        };

        Ok(info)
    }
}

/// Helper to get first attribute value from LDAP entry
fn get_first_attr(entry: &SearchEntry, attr: &str) -> Option<String> {
    entry.attrs.get(attr).and_then(|v| v.first().cloned())
}

/// LDAP authentication provider for integration with auth system
pub struct LdapAuthProvider {
    client: Arc<LdapClient>,
}

impl LdapAuthProvider {
    /// Create a new LDAP auth provider
    pub fn new(config: LdapConfig) -> Self {
        Self {
            client: Arc::new(LdapClient::new(config)),
        }
    }

    /// Authenticate user
    pub async fn authenticate(&self, username: &str, password: &str) -> LdapAuthResult {
        self.client.authenticate(username, password).await
    }

    /// Get underlying client for admin operations
    pub fn client(&self) -> Arc<LdapClient> {
        self.client.clone()
    }

    /// Check if LDAP is enabled
    pub fn is_enabled(&self) -> bool {
        self.client.config.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ldap_client_creation() {
        let config = LdapConfig {
            enabled: true,
            server_url: "ldap://localhost:389".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "admin".to_string(),
            user_base_dn: "ou=users,dc=example,dc=com".to_string(),
            ..Default::default()
        };

        let client = LdapClient::new(config);
        // Note: actual connection test requires running LDAP server
        assert!(client.config.enabled);
    }

    #[tokio::test]
    async fn test_user_caching() {
        let config = LdapConfig {
            enabled: true,
            server_url: "ldap://localhost:389".to_string(),
            bind_dn: "cn=admin,dc=example,dc=com".to_string(),
            bind_password: "admin".to_string(),
            user_base_dn: "ou=users,dc=example,dc=com".to_string(),
            cache_ttl_seconds: 60,
            ..Default::default()
        };

        let client = LdapClient::new(config);

        let user = LdapUser {
            dn: "uid=test,ou=users,dc=example,dc=com".to_string(),
            username: "test".to_string(),
            email: Some("test@example.com".to_string()),
            display_name: Some("Test User".to_string()),
            groups: vec![],
            policies: vec!["readonly".to_string()],
            attributes: HashMap::new(),
        };

        client.cache_user(user.clone()).await;

        let cached = client.get_cached_user("test").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().username, "test");
    }
}
