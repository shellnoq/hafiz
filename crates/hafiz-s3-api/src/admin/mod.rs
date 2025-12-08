//! Admin API routes for Hafiz management
//!
//! These endpoints provide administrative access to manage buckets,
//! users, and view system statistics.

mod stats;
mod users;
mod server;

use axum::{
    Router,
    routing::{get, post, delete},
    middleware,
};

use crate::middleware::auth::admin_auth;
use crate::server::AppState;

pub use stats::*;
pub use users::*;
pub use server::*;

/// Create the admin API router
pub fn admin_routes() -> Router<AppState> {
    Router::new()
        // Dashboard & Stats
        .route("/stats", get(get_dashboard_stats))
        .route("/stats/storage", get(get_storage_stats))
        
        // Server info
        .route("/server/info", get(get_server_info))
        .route("/server/health", get(health_check))
        
        // Bucket management (enhanced versions)
        .route("/buckets", get(list_buckets_detailed))
        .route("/buckets/:name/stats", get(get_bucket_stats))
        
        // User management
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/:access_key", get(get_user))
        .route("/users/:access_key", delete(delete_user))
        .route("/users/:access_key/enable", post(enable_user))
        .route("/users/:access_key/disable", post(disable_user))
        .route("/users/:access_key/keys", post(rotate_keys))
        
        // Apply admin authentication to all routes
        .layer(middleware::from_fn(admin_auth))
}

/// Admin API without authentication (for development/testing)
pub fn admin_routes_no_auth() -> Router<AppState> {
    Router::new()
        .route("/stats", get(get_dashboard_stats))
        .route("/stats/storage", get(get_storage_stats))
        .route("/server/info", get(get_server_info))
        .route("/server/health", get(health_check))
        .route("/buckets", get(list_buckets_detailed))
        .route("/buckets/:name/stats", get(get_bucket_stats))
        .route("/users", get(list_users))
        .route("/users", post(create_user))
        .route("/users/:access_key", get(get_user))
        .route("/users/:access_key", delete(delete_user))
        .route("/users/:access_key/enable", post(enable_user))
        .route("/users/:access_key/disable", post(disable_user))
        .route("/users/:access_key/keys", post(rotate_keys))
}
