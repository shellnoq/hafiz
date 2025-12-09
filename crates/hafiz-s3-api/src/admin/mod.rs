//! Admin API routes for Hafiz management
//!
//! These endpoints provide administrative access to manage buckets,
//! users, cluster, LDAP, and view system statistics.

#[cfg(feature = "cluster")]
mod cluster;
mod ldap;
mod presigned;
mod stats;
mod users;
mod server;

use axum::{
    Router,
    routing::{get, post, delete, put},
    middleware,
};

use crate::middleware::auth::admin_auth;
use crate::server::AppState;

#[cfg(feature = "cluster")]
pub use cluster::*;
pub use ldap::*;
pub use presigned::*;
pub use stats::*;
pub use users::*;
pub use server::*;

/// Create the admin API router
pub fn admin_routes() -> Router<AppState> {
    let router = Router::new()
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

        // Pre-signed URLs
        .route("/presigned", post(generate_presigned))
        .route("/presigned/download/:bucket/*key", post(generate_presigned_download))
        .route("/presigned/upload/:bucket/*key", post(generate_presigned_upload));

    // Add cluster routes if feature is enabled
    #[cfg(feature = "cluster")]
    let router = router
        .route("/cluster/status", get(get_cluster_status))
        .route("/cluster/health", get(cluster_health_check))
        .route("/cluster/nodes", get(list_cluster_nodes))
        .route("/cluster/nodes/:node_id", get(get_cluster_node))
        .route("/cluster/nodes/:node_id/drain", post(drain_cluster_node))
        .route("/cluster/nodes/:node_id", delete(remove_cluster_node))
        .route("/cluster/replication/rules", get(list_replication_rules))
        .route("/cluster/replication/rules", post(create_replication_rule))
        .route("/cluster/replication/rules/:rule_id", get(get_replication_rule))
        .route("/cluster/replication/rules/:rule_id", delete(delete_replication_rule))
        .route("/cluster/replication/stats", get(get_replication_stats));

    router.layer(middleware::from_fn(admin_auth))
}

/// Admin API without authentication (for development/testing)
pub fn admin_routes_no_auth() -> Router<AppState> {
    let router = Router::new()
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
        // Pre-signed URLs
        .route("/presigned", post(generate_presigned))
        .route("/presigned/download/:bucket/*key", post(generate_presigned_download))
        .route("/presigned/upload/:bucket/*key", post(generate_presigned_upload));

    // Add cluster routes if feature is enabled
    #[cfg(feature = "cluster")]
    let router = router
        .route("/cluster/status", get(get_cluster_status))
        .route("/cluster/health", get(cluster_health_check))
        .route("/cluster/nodes", get(list_cluster_nodes))
        .route("/cluster/nodes/:node_id", get(get_cluster_node))
        .route("/cluster/nodes/:node_id/drain", post(drain_cluster_node))
        .route("/cluster/nodes/:node_id", delete(remove_cluster_node))
        .route("/cluster/replication/rules", get(list_replication_rules))
        .route("/cluster/replication/rules", post(create_replication_rule))
        .route("/cluster/replication/rules/:rule_id", get(get_replication_rule))
        .route("/cluster/replication/rules/:rule_id", delete(delete_replication_rule))
        .route("/cluster/replication/stats", get(get_replication_stats));

    router
}
