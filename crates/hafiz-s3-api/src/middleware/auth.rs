//! Authentication middleware for Admin API

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};

use crate::server::AppState;

/// Admin authentication middleware
///
/// Supports two authentication methods:
/// 1. Bearer token: Authorization: Bearer <access_key>:<secret_key_base64>
/// 2. Basic auth: Authorization: Basic <base64(access_key:secret_key)>
pub async fn admin_auth(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            validate_bearer_auth(header, &state).await?;
        }
        Some(header) if header.starts_with("Basic ") => {
            validate_basic_auth(header, &state).await?;
        }
        _ => {
            // For development, also check query params
            let uri = request.uri();
            if let Some(query) = uri.query() {
                if query.contains("access_key=") && query.contains("secret_key=") {
                    // Extract from query params (development only)
                    // In production this should be disabled
                    let params: std::collections::HashMap<_, _> =
                        url::form_urlencoded::parse(query.as_bytes())
                            .into_owned()
                            .collect();

                    if let (Some(ak), Some(sk)) =
                        (params.get("access_key"), params.get("secret_key"))
                    {
                        validate_credentials(ak, sk, &state).await?;
                    } else {
                        return Err(StatusCode::UNAUTHORIZED);
                    }
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            } else {
                return Err(StatusCode::UNAUTHORIZED);
            }
        }
    }

    Ok(next.run(request).await)
}

/// Validate Bearer token authentication
async fn validate_bearer_auth(header: &str, state: &AppState) -> Result<(), StatusCode> {
    let token = header.trim_start_matches("Bearer ");

    // Token format: access_key:secret_key_base64
    let parts: Vec<&str> = token.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let access_key = parts[0];
    let secret_key = BASE64
        .decode(parts[1])
        .map_err(|_| StatusCode::UNAUTHORIZED)
        .and_then(|bytes| String::from_utf8(bytes).map_err(|_| StatusCode::UNAUTHORIZED))?;

    validate_credentials(access_key, &secret_key, state).await
}

/// Validate Basic authentication
async fn validate_basic_auth(header: &str, state: &AppState) -> Result<(), StatusCode> {
    let encoded = header.trim_start_matches("Basic ");

    let decoded = BASE64
        .decode(encoded)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let credentials = String::from_utf8(decoded).map_err(|_| StatusCode::UNAUTHORIZED)?;

    let parts: Vec<&str> = credentials.splitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    validate_credentials(parts[0], parts[1], state).await
}

/// Validate credentials against the metadata store
async fn validate_credentials(
    access_key: &str,
    secret_key: &str,
    state: &AppState,
) -> Result<(), StatusCode> {
    let metadata = &state.metadata;

    let cred = metadata
        .get_credentials(access_key)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !cred.enabled {
        return Err(StatusCode::FORBIDDEN);
    }

    if cred.secret_key != secret_key {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(())
}
