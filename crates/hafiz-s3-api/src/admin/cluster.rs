//! Cluster management API endpoints
//!
//! Provides REST API for cluster administration:
//! - View cluster status and nodes
//! - Manage replication rules
//! - Monitor replication progress

#![cfg(feature = "cluster")]

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use hafiz_core::types::{
    ClusterNode, ClusterStats, ConflictResolution, NodeId, NodeRole, NodeStatus, ReplicationMode,
    ReplicationRule, ReplicationStatus,
};

use crate::server::AppState;

// ============================================================================
// Response Types
// ============================================================================

/// Cluster status response
#[derive(Debug, Serialize)]
pub struct ClusterStatusResponse {
    pub enabled: bool,
    pub cluster_name: String,
    pub local_node: NodeInfoResponse,
    pub stats: ClusterStats,
}

/// Node information response
#[derive(Debug, Serialize)]
pub struct NodeInfoResponse {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub role: String,
    pub status: String,
    pub region: Option<String>,
    pub zone: Option<String>,
    pub joined_at: String,
    pub last_heartbeat: String,
    pub version: String,
}

impl From<ClusterNode> for NodeInfoResponse {
    fn from(node: ClusterNode) -> Self {
        Self {
            id: node.id,
            name: node.name,
            endpoint: node.endpoint,
            role: format!("{:?}", node.role).to_lowercase(),
            status: format!("{:?}", node.status).to_lowercase(),
            region: node.region,
            zone: node.zone,
            joined_at: node.joined_at.to_rfc3339(),
            last_heartbeat: node.last_heartbeat.to_rfc3339(),
            version: node.version,
        }
    }
}

/// List of nodes response
#[derive(Debug, Serialize)]
pub struct NodesListResponse {
    pub nodes: Vec<NodeInfoResponse>,
    pub total: usize,
    pub healthy: usize,
}

/// Replication rule response
#[derive(Debug, Serialize)]
pub struct ReplicationRuleResponse {
    pub id: String,
    pub enabled: bool,
    pub source_bucket: String,
    pub destination_bucket: String,
    pub target_nodes: Vec<String>,
    pub prefix_filter: Option<String>,
    pub mode: String,
    pub priority: i32,
    pub replicate_deletes: bool,
    pub replicate_existing: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ReplicationRule> for ReplicationRuleResponse {
    fn from(rule: ReplicationRule) -> Self {
        Self {
            id: rule.id,
            enabled: rule.enabled,
            source_bucket: rule.source_bucket,
            destination_bucket: rule.destination_bucket,
            target_nodes: rule.target_nodes,
            prefix_filter: rule.prefix_filter,
            mode: format!("{:?}", rule.mode).to_lowercase(),
            priority: rule.priority,
            replicate_deletes: rule.replicate_deletes,
            replicate_existing: rule.replicate_existing,
            created_at: rule.created_at.to_rfc3339(),
            updated_at: rule.updated_at.to_rfc3339(),
        }
    }
}

/// Replication rules list response
#[derive(Debug, Serialize)]
pub struct ReplicationRulesResponse {
    pub rules: Vec<ReplicationRuleResponse>,
    pub total: usize,
}

/// Replicator statistics response
#[derive(Debug, Serialize)]
pub struct ReplicatorStatsResponse {
    pub events_processed: u64,
    pub successful: u64,
    pub failed: u64,
    pub pending: u64,
    pub in_progress: u64,
    pub bytes_replicated: u64,
    pub avg_latency_ms: f64,
}

// ============================================================================
// Request Types
// ============================================================================

/// Create replication rule request
#[derive(Debug, Deserialize)]
pub struct CreateReplicationRuleRequest {
    pub source_bucket: String,
    pub destination_bucket: Option<String>,
    pub target_nodes: Option<Vec<String>>,
    pub prefix_filter: Option<String>,
    pub mode: Option<String>,
    pub priority: Option<i32>,
    pub replicate_deletes: Option<bool>,
    pub replicate_existing: Option<bool>,
}

/// Update replication rule request
#[derive(Debug, Deserialize)]
pub struct UpdateReplicationRuleRequest {
    pub enabled: Option<bool>,
    pub target_nodes: Option<Vec<String>>,
    pub prefix_filter: Option<String>,
    pub mode: Option<String>,
    pub priority: Option<i32>,
    pub replicate_deletes: Option<bool>,
}

/// Drain node request
#[derive(Debug, Deserialize)]
pub struct DrainNodeRequest {
    pub graceful: Option<bool>,
    pub timeout_secs: Option<u64>,
}

// ============================================================================
// Handlers
// ============================================================================

/// GET /api/v1/cluster/status
/// Get cluster status and statistics
pub async fn get_cluster_status(
    State(state): State<AppState>,
) -> Result<Json<ClusterStatusResponse>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    let local_node = cluster.local_node();
    let stats = cluster.stats();

    Ok(Json(ClusterStatusResponse {
        enabled: cluster.is_enabled(),
        cluster_name: local_node.name.clone(),
        local_node: local_node.into(),
        stats,
    }))
}

/// GET /api/v1/cluster/nodes
/// List all nodes in the cluster
pub async fn list_cluster_nodes(
    State(state): State<AppState>,
) -> Result<Json<NodesListResponse>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    let mut nodes: Vec<NodeInfoResponse> = cluster.nodes().into_iter().map(|n| n.into()).collect();

    // Add local node
    nodes.insert(0, cluster.local_node().into());

    let healthy = nodes.iter().filter(|n| n.status == "healthy").count();
    let total = nodes.len();

    Ok(Json(NodesListResponse {
        nodes,
        total,
        healthy,
    }))
}

/// GET /api/v1/cluster/nodes/:node_id
/// Get details of a specific node
pub async fn get_cluster_node(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<Json<NodeInfoResponse>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    // Check if it's the local node
    let local = cluster.local_node();
    if local.id == node_id {
        return Ok(Json(local.into()));
    }

    // Find in cluster nodes
    let node = cluster.get_node(&node_id).ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            format!("Node not found: {}", node_id),
        )
    })?;

    Ok(Json(node.into()))
}

/// POST /api/v1/cluster/nodes/:node_id/drain
/// Drain a node (prepare for maintenance)
pub async fn drain_cluster_node(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
    Json(request): Json<DrainNodeRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    // TODO: Implement drain logic
    // 1. Stop accepting new requests on the node
    // 2. Wait for in-flight requests to complete
    // 3. Trigger replication of any pending data
    // 4. Mark node as draining

    Ok(Json(serde_json::json!({
        "status": "draining",
        "node_id": node_id,
        "message": "Node drain initiated"
    })))
}

/// DELETE /api/v1/cluster/nodes/:node_id
/// Remove a node from the cluster
pub async fn remove_cluster_node(
    State(state): State<AppState>,
    Path(node_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    // Don't allow removing the local node via API
    if cluster.local_node().id == node_id {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot remove local node via API".to_string(),
        ));
    }

    // TODO: Implement node removal
    // 1. Notify the node to leave
    // 2. Remove from cluster membership
    // 3. Trigger data rebalancing if needed

    Ok(Json(serde_json::json!({
        "status": "removed",
        "node_id": node_id,
        "message": "Node removed from cluster"
    })))
}

/// GET /api/v1/cluster/replication/rules
/// List all replication rules
pub async fn list_replication_rules(
    State(state): State<AppState>,
) -> Result<Json<ReplicationRulesResponse>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    let rules: Vec<ReplicationRuleResponse> = cluster
        .replication_rules()
        .into_iter()
        .map(|r| r.into())
        .collect();

    let total = rules.len();

    Ok(Json(ReplicationRulesResponse { rules, total }))
}

/// POST /api/v1/cluster/replication/rules
/// Create a new replication rule
pub async fn create_replication_rule(
    State(state): State<AppState>,
    Json(request): Json<CreateReplicationRuleRequest>,
) -> Result<(StatusCode, Json<ReplicationRuleResponse>), (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    // Validate source bucket exists
    let bucket_exists = state.metadata.get_bucket(&request.source_bucket).await;
    if bucket_exists.is_err() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Source bucket not found: {}", request.source_bucket),
        ));
    }

    // Create rule
    let mut rule = ReplicationRule::new(
        request.source_bucket.clone(),
        request.destination_bucket.unwrap_or(request.source_bucket),
    );

    if let Some(nodes) = request.target_nodes {
        rule.target_nodes = nodes;
    }
    if let Some(prefix) = request.prefix_filter {
        rule.prefix_filter = Some(prefix);
    }
    if let Some(mode) = request.mode {
        rule.mode = match mode.as_str() {
            "sync" => ReplicationMode::Sync,
            "async" => ReplicationMode::Async,
            _ => ReplicationMode::None,
        };
    }
    if let Some(priority) = request.priority {
        rule.priority = priority;
    }
    if let Some(replicate_deletes) = request.replicate_deletes {
        rule.replicate_deletes = replicate_deletes;
    }
    if let Some(replicate_existing) = request.replicate_existing {
        rule.replicate_existing = replicate_existing;
    }

    cluster.add_replication_rule(rule.clone());

    Ok((StatusCode::CREATED, Json(rule.into())))
}

/// GET /api/v1/cluster/replication/rules/:rule_id
/// Get a specific replication rule
pub async fn get_replication_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<Json<ReplicationRuleResponse>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    let rule = cluster
        .replication_rules()
        .into_iter()
        .find(|r| r.id == rule_id)
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                format!("Rule not found: {}", rule_id),
            )
        })?;

    Ok(Json(rule.into()))
}

/// DELETE /api/v1/cluster/replication/rules/:rule_id
/// Delete a replication rule
pub async fn delete_replication_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    if cluster.remove_replication_rule(&rule_id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            format!("Rule not found: {}", rule_id),
        ))
    }
}

/// GET /api/v1/cluster/replication/stats
/// Get replication statistics
pub async fn get_replication_stats(
    State(state): State<AppState>,
) -> Result<Json<ReplicatorStatsResponse>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Cluster mode not enabled".to_string(),
        )
    })?;

    let stats = cluster.replicator_stats();

    Ok(Json(ReplicatorStatsResponse {
        events_processed: stats.events_processed,
        successful: stats.successful,
        failed: stats.failed,
        pending: stats.pending,
        in_progress: stats.in_progress,
        bytes_replicated: stats.bytes_replicated,
        avg_latency_ms: stats.avg_latency_ms,
    }))
}

/// GET /api/v1/cluster/health
/// Cluster health check endpoint
pub async fn cluster_health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let cluster = state.cluster.as_ref();

    let (cluster_enabled, cluster_healthy, node_count) = if let Some(c) = cluster {
        let healthy_nodes = c.healthy_nodes().len() + 1; // +1 for local
        let total_nodes = c.nodes().len() + 1;
        (true, healthy_nodes > total_nodes / 2, total_nodes)
    } else {
        (false, true, 1)
    };

    let status = if cluster_healthy {
        "healthy"
    } else {
        "degraded"
    };

    Ok(Json(serde_json::json!({
        "status": status,
        "cluster_enabled": cluster_enabled,
        "node_count": node_count,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
