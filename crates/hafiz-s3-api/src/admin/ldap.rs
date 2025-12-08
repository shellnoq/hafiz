//! LDAP Admin API handlers
//!
//! Provides endpoints for:
//! - LDAP configuration management
//! - Connection testing
//! - User search testing
//! - Status monitoring

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use hafiz_auth::{
    LdapConfig, LdapStatus, LdapAuthProvider, LdapClient,
    ldap::{
        TestLdapConnectionRequest, TestLdapConnectionResponse,
        TestLdapSearchRequest, TestLdapSearchResponse,
        UpdateLdapConfigRequest,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// LDAP state for admin API
pub struct LdapAdminState {
    pub config: Arc<RwLock<LdapConfig>>,
    pub provider: Arc<RwLock<Option<LdapAuthProvider>>>,
}

impl LdapAdminState {
    pub fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(LdapConfig::default())),
            provider: Arc::new(RwLock::new(None)),
        }
    }

    pub fn with_config(config: LdapConfig) -> Self {
        let provider = if config.enabled {
            Some(LdapAuthProvider::new(config.clone()))
        } else {
            None
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            provider: Arc::new(RwLock::new(provider)),
        }
    }

    pub async fn update_config(&self, config: LdapConfig) {
        let provider = if config.enabled {
            Some(LdapAuthProvider::new(config.clone()))
        } else {
            None
        };

        *self.config.write().await = config;
        *self.provider.write().await = provider;
    }
}

/// Response wrapper
#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.into()),
        }
    }
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/v1/ldap/status - Get LDAP status
pub async fn get_ldap_status(
    State(state): State<Arc<LdapAdminState>>,
) -> impl IntoResponse {
    debug!("GET /api/v1/ldap/status");

    let config = state.config.read().await;
    let provider = state.provider.read().await;

    let status = if let Some(ref provider) = *provider {
        provider.client().get_status().await
    } else {
        LdapStatus {
            enabled: config.enabled,
            connected: false,
            server_url: config.server_url.clone(),
            server_type: config.server_type,
            last_connection: None,
            error: Some("LDAP provider not initialized".to_string()),
            cached_users: 0,
        }
    };

    Json(ApiResponse::success(status))
}

/// GET /api/v1/ldap/config - Get LDAP configuration (sanitized)
pub async fn get_ldap_config(
    State(state): State<Arc<LdapAdminState>>,
) -> impl IntoResponse {
    debug!("GET /api/v1/ldap/config");

    let config = state.config.read().await;
    
    // Return sanitized config (no passwords)
    let sanitized = SanitizedLdapConfig {
        enabled: config.enabled,
        server_url: config.server_url.clone(),
        start_tls: config.start_tls,
        bind_dn: config.bind_dn.clone(),
        user_base_dn: config.user_base_dn.clone(),
        user_filter: config.user_filter.clone(),
        group_base_dn: config.group_base_dn.clone(),
        group_filter: config.group_filter.clone(),
        attribute_mappings: config.attribute_mappings.clone(),
        group_policies: config.group_policies.clone(),
        default_policies: config.default_policies.clone(),
        timeout_seconds: config.timeout_seconds,
        cache_ttl_seconds: config.cache_ttl_seconds,
        server_type: config.server_type,
    };

    Json(ApiResponse::success(sanitized))
}

/// PUT /api/v1/ldap/config - Update LDAP configuration
pub async fn update_ldap_config(
    State(state): State<Arc<LdapAdminState>>,
    Json(request): Json<UpdateLdapConfigRequest>,
) -> impl IntoResponse {
    debug!("PUT /api/v1/ldap/config");

    // Validate configuration
    if let Err(e) = request.config.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e)),
        );
    }

    // Update configuration
    state.update_config(request.config.clone()).await;
    
    info!("LDAP configuration updated");

    (
        StatusCode::OK,
        Json(ApiResponse::success(UpdateConfigResponse {
            message: "LDAP configuration updated successfully".to_string(),
        })),
    )
}

/// POST /api/v1/ldap/test-connection - Test LDAP connection
pub async fn test_ldap_connection(
    State(state): State<Arc<LdapAdminState>>,
    Json(request): Json<TestLdapConnectionRequest>,
) -> impl IntoResponse {
    debug!("POST /api/v1/ldap/test-connection");

    // Create temporary config for testing
    let test_config = LdapConfig {
        enabled: true,
        server_url: request.server_url,
        start_tls: request.start_tls,
        skip_tls_verify: request.skip_tls_verify,
        bind_dn: request.bind_dn,
        bind_password: request.bind_password,
        user_base_dn: "dc=test".to_string(),
        ..Default::default()
    };

    let client = LdapClient::new(test_config);
    let response = client.test_connection().await;

    if response.success {
        info!("LDAP connection test successful");
    } else {
        error!("LDAP connection test failed: {}", response.message);
    }

    Json(ApiResponse::success(response))
}

/// POST /api/v1/ldap/test-search - Test LDAP user search
pub async fn test_ldap_search(
    State(state): State<Arc<LdapAdminState>>,
    Json(request): Json<TestLdapSearchRequest>,
) -> impl IntoResponse {
    debug!("POST /api/v1/ldap/test-search username={}", request.username);

    let provider = state.provider.read().await;

    let response = if let Some(ref provider) = *provider {
        match provider.client().search_user(&request.username).await {
            Ok(Some(user)) => TestLdapSearchResponse {
                success: true,
                message: "User found".to_string(),
                user: Some(user),
            },
            Ok(None) => TestLdapSearchResponse {
                success: false,
                message: "User not found".to_string(),
                user: None,
            },
            Err(e) => TestLdapSearchResponse {
                success: false,
                message: e,
                user: None,
            },
        }
    } else {
        TestLdapSearchResponse {
            success: false,
            message: "LDAP is not configured".to_string(),
            user: None,
        }
    };

    Json(ApiResponse::success(response))
}

/// POST /api/v1/ldap/clear-cache - Clear user cache
pub async fn clear_ldap_cache(
    State(state): State<Arc<LdapAdminState>>,
) -> impl IntoResponse {
    debug!("POST /api/v1/ldap/clear-cache");

    let provider = state.provider.read().await;

    if let Some(ref provider) = *provider {
        provider.client().clear_cache().await;
        info!("LDAP cache cleared");
        Json(ApiResponse::success(ClearCacheResponse {
            message: "Cache cleared successfully".to_string(),
        }))
    } else {
        Json(ApiResponse::<ClearCacheResponse>::error("LDAP is not configured"))
    }
}

/// POST /api/v1/ldap/authenticate - Test authentication
pub async fn test_ldap_authenticate(
    State(state): State<Arc<LdapAdminState>>,
    Json(request): Json<TestAuthenticateRequest>,
) -> impl IntoResponse {
    debug!("POST /api/v1/ldap/authenticate username={}", request.username);

    let provider = state.provider.read().await;

    let response = if let Some(ref provider) = *provider {
        let result = provider.authenticate(&request.username, &request.password).await;
        
        match result {
            hafiz_auth::LdapAuthResult::Success(user) => TestAuthenticateResponse {
                success: true,
                message: "Authentication successful".to_string(),
                user: Some(user),
            },
            hafiz_auth::LdapAuthResult::InvalidCredentials => TestAuthenticateResponse {
                success: false,
                message: "Invalid credentials".to_string(),
                user: None,
            },
            hafiz_auth::LdapAuthResult::UserNotFound => TestAuthenticateResponse {
                success: false,
                message: "User not found".to_string(),
                user: None,
            },
            hafiz_auth::LdapAuthResult::AccountDisabled => TestAuthenticateResponse {
                success: false,
                message: "Account is disabled".to_string(),
                user: None,
            },
            hafiz_auth::LdapAuthResult::ConnectionError(e) => TestAuthenticateResponse {
                success: false,
                message: format!("Connection error: {}", e),
                user: None,
            },
            hafiz_auth::LdapAuthResult::ConfigError(e) => TestAuthenticateResponse {
                success: false,
                message: format!("Configuration error: {}", e),
                user: None,
            },
        }
    } else {
        TestAuthenticateResponse {
            success: false,
            message: "LDAP is not configured".to_string(),
            user: None,
        }
    };

    Json(ApiResponse::success(response))
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizedLdapConfig {
    pub enabled: bool,
    pub server_url: String,
    pub start_tls: bool,
    pub bind_dn: String,
    pub user_base_dn: String,
    pub user_filter: String,
    pub group_base_dn: Option<String>,
    pub group_filter: Option<String>,
    pub attribute_mappings: hafiz_auth::AttributeMappings,
    pub group_policies: std::collections::HashMap<String, Vec<String>>,
    pub default_policies: Vec<String>,
    pub timeout_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub server_type: hafiz_auth::LdapServerType,
}

#[derive(Debug, Serialize)]
struct UpdateConfigResponse {
    message: String,
}

#[derive(Debug, Serialize)]
struct ClearCacheResponse {
    message: String,
}

#[derive(Debug, Deserialize)]
pub struct TestAuthenticateRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct TestAuthenticateResponse {
    pub success: bool,
    pub message: String,
    pub user: Option<hafiz_auth::LdapUser>,
}

// ============================================================================
// Router
// ============================================================================

use axum::{routing::{get, post, put}, Router};

/// Create LDAP admin routes
pub fn ldap_routes(state: Arc<LdapAdminState>) -> Router {
    Router::new()
        .route("/status", get(get_ldap_status))
        .route("/config", get(get_ldap_config))
        .route("/config", put(update_ldap_config))
        .route("/test-connection", post(test_ldap_connection))
        .route("/test-search", post(test_ldap_search))
        .route("/authenticate", post(test_ldap_authenticate))
        .route("/clear-cache", post(clear_ldap_cache))
        .with_state(state)
}
