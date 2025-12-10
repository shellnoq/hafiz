//! User management endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::AppState;
use hafiz_auth::generate_credentials;

/// User information response
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub name: String,
    pub access_key: String,
    pub email: Option<String>,
    pub enabled: bool,
    pub created_at: String,
    pub last_used: Option<String>,
    pub policies: Vec<String>,
}

/// User list response
#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub users: Vec<UserInfo>,
    pub total: i64,
}

/// Create user request
#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: Option<String>,
    pub policies: Option<Vec<String>>,
}

/// Create user response
#[derive(Debug, Serialize)]
pub struct CreateUserResponse {
    pub name: String,
    pub access_key: String,
    pub secret_key: String,
    pub email: Option<String>,
    pub created_at: String,
}

/// User update request
#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub enabled: Option<bool>,
    pub policies: Option<Vec<String>>,
}

/// Key rotation response
#[derive(Debug, Serialize)]
pub struct RotateKeysResponse {
    pub access_key: String,
    pub secret_key: String,
    pub created_at: String,
}

/// List all users
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<UserListResponse>, (StatusCode, String)> {
    let metadata = &state.metadata;

    let credentials = metadata
        .list_credentials()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let users: Vec<UserInfo> = credentials
        .into_iter()
        .map(|cred| UserInfo {
            name: cred.name.unwrap_or_else(|| cred.access_key.clone()),
            access_key: cred.access_key,
            email: cred.email,
            enabled: cred.enabled,
            created_at: cred.created_at.to_rfc3339(),
            last_used: cred.last_used.map(|d| d.to_rfc3339()),
            policies: cred.policies,
        })
        .collect();

    let total = users.len() as i64;

    Ok(Json(UserListResponse { users, total }))
}

/// Get a specific user
pub async fn get_user(
    State(state): State<AppState>,
    Path(access_key): Path<String>,
) -> Result<Json<UserInfo>, (StatusCode, String)> {
    let metadata = &state.metadata;

    let cred = metadata
        .get_credentials(&access_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("User '{}' not found", access_key),
        ))?;

    Ok(Json(UserInfo {
        name: cred.name.unwrap_or_else(|| cred.access_key.clone()),
        access_key: cred.access_key,
        email: cred.email,
        enabled: cred.enabled,
        created_at: cred.created_at.to_rfc3339(),
        last_used: cred.last_used.map(|d| d.to_rfc3339()),
        policies: cred.policies,
    }))
}

/// Create a new user
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<CreateUserResponse>), (StatusCode, String)> {
    // Validate name
    if req.name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()));
    }

    if req.name.len() > 64 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Name too long (max 64 characters)".to_string(),
        ));
    }

    // Generate credentials
    let (access_key, secret_key) = generate_credentials();

    let metadata = &state.metadata;

    // Check if name already exists
    let existing = metadata.list_credentials().await.unwrap_or_default();
    if existing
        .iter()
        .any(|c| c.name.as_deref() == Some(&req.name))
    {
        return Err((
            StatusCode::CONFLICT,
            format!("User '{}' already exists", req.name),
        ));
    }

    // Create credentials
    let now = chrono::Utc::now();
    let cred = hafiz_core::types::Credentials {
        access_key: access_key.clone(),
        secret_key: secret_key.clone(),
        name: Some(req.name.clone()),
        email: req.email.clone(),
        enabled: true,
        created_at: now,
        last_used: None,
        policies: req.policies.unwrap_or_default(),
    };

    metadata
        .create_credentials(&cred)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(CreateUserResponse {
            name: req.name,
            access_key,
            secret_key,
            email: req.email,
            created_at: now.to_rfc3339(),
        }),
    ))
}

/// Delete a user
pub async fn delete_user(
    State(state): State<AppState>,
    Path(access_key): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let metadata = &state.metadata;

    // Check user exists
    let cred = metadata
        .get_credentials(&access_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("User '{}' not found", access_key),
        ))?;

    // Prevent deleting the last admin user
    let all_users = metadata.list_credentials().await.unwrap_or_default();
    if all_users.len() <= 1 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot delete the last user".to_string(),
        ));
    }

    // Delete
    metadata
        .delete_credentials(&access_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Enable a user
pub async fn enable_user(
    State(state): State<AppState>,
    Path(access_key): Path<String>,
) -> Result<Json<UserInfo>, (StatusCode, String)> {
    let metadata = &state.metadata;

    // Get current user
    let mut cred = metadata
        .get_credentials(&access_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("User '{}' not found", access_key),
        ))?;

    cred.enabled = true;

    metadata
        .update_credentials(&cred)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(UserInfo {
        name: cred.name.unwrap_or_else(|| cred.access_key.clone()),
        access_key: cred.access_key,
        email: cred.email,
        enabled: cred.enabled,
        created_at: cred.created_at.to_rfc3339(),
        last_used: cred.last_used.map(|d| d.to_rfc3339()),
        policies: cred.policies,
    }))
}

/// Disable a user
pub async fn disable_user(
    State(state): State<AppState>,
    Path(access_key): Path<String>,
) -> Result<Json<UserInfo>, (StatusCode, String)> {
    let metadata = &state.metadata;

    // Get current user
    let mut cred = metadata
        .get_credentials(&access_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("User '{}' not found", access_key),
        ))?;

    // Prevent disabling all users
    let all_users = metadata.list_credentials().await.unwrap_or_default();
    let enabled_count = all_users.iter().filter(|u| u.enabled).count();
    if enabled_count <= 1 && cred.enabled {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot disable the last enabled user".to_string(),
        ));
    }

    cred.enabled = false;

    metadata
        .update_credentials(&cred)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(UserInfo {
        name: cred.name.unwrap_or_else(|| cred.access_key.clone()),
        access_key: cred.access_key,
        email: cred.email,
        enabled: cred.enabled,
        created_at: cred.created_at.to_rfc3339(),
        last_used: cred.last_used.map(|d| d.to_rfc3339()),
        policies: cred.policies,
    }))
}

/// Rotate user's access keys
pub async fn rotate_keys(
    State(state): State<AppState>,
    Path(access_key): Path<String>,
) -> Result<Json<RotateKeysResponse>, (StatusCode, String)> {
    let metadata = &state.metadata;

    // Get current user
    let old_cred = metadata
        .get_credentials(&access_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((
            StatusCode::NOT_FOUND,
            format!("User '{}' not found", access_key),
        ))?;

    // Generate new credentials
    let (new_access_key, new_secret_key) = generate_credentials();
    let now = chrono::Utc::now();

    // Create new credentials with same settings
    let new_cred = hafiz_core::types::Credentials {
        access_key: new_access_key.clone(),
        secret_key: new_secret_key.clone(),
        name: old_cred.name,
        email: old_cred.email,
        enabled: old_cred.enabled,
        created_at: now,
        last_used: None,
        policies: old_cred.policies,
    };

    // Delete old and create new
    metadata
        .delete_credentials(&access_key)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    metadata
        .create_credentials(&new_cred)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RotateKeysResponse {
        access_key: new_access_key,
        secret_key: new_secret_key,
        created_at: now.to_rfc3339(),
    }))
}
