//! Dashboard and statistics endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::server::AppState;

/// Dashboard statistics response
#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_buckets: i64,
    pub total_objects: i64,
    pub total_size: i64,
    pub total_users: i64,
    pub recent_buckets: Vec<BucketSummary>,
    pub storage_by_bucket: Vec<BucketStorageInfo>,
}

/// Bucket summary for dashboard
#[derive(Debug, Serialize)]
pub struct BucketSummary {
    pub name: String,
    pub object_count: i64,
    pub size: i64,
    pub created_at: String,
    pub versioning_enabled: bool,
    pub encryption_enabled: bool,
}

/// Bucket storage information
#[derive(Debug, Serialize)]
pub struct BucketStorageInfo {
    pub name: String,
    pub size: i64,
    pub percentage: f64,
}

/// Detailed bucket information
#[derive(Debug, Serialize)]
pub struct BucketDetailed {
    pub name: String,
    pub object_count: i64,
    pub size: i64,
    pub created_at: String,
    pub versioning_enabled: bool,
    pub versioning_status: String,
    pub encryption_enabled: bool,
    pub lifecycle_rules: i64,
    pub tags: Vec<BucketTag>,
}

#[derive(Debug, Serialize)]
pub struct BucketTag {
    pub key: String,
    pub value: String,
}

/// Storage statistics
#[derive(Debug, Serialize)]
pub struct StorageStats {
    pub total_size: i64,
    pub total_objects: i64,
    pub average_object_size: i64,
    pub largest_bucket: Option<String>,
    pub largest_bucket_size: i64,
    pub storage_by_type: Vec<StorageByType>,
}

#[derive(Debug, Serialize)]
pub struct StorageByType {
    pub content_type: String,
    pub count: i64,
    pub size: i64,
}

/// Bucket-specific statistics
#[derive(Debug, Serialize)]
pub struct BucketStats {
    pub name: String,
    pub object_count: i64,
    pub total_size: i64,
    pub version_count: i64,
    pub delete_marker_count: i64,
    pub multipart_uploads: i64,
    pub created_at: String,
    pub last_modified: Option<String>,
}

/// Get dashboard statistics
pub async fn get_dashboard_stats(
    State(state): State<AppState>,
) -> Result<Json<DashboardStats>, (StatusCode, String)> {
    let metadata = &state.metadata;
    
    // Get all buckets
    let buckets = metadata
        .list_buckets()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let total_buckets = buckets.len() as i64;
    let mut total_objects: i64 = 0;
    let mut total_size: i64 = 0;
    let mut recent_buckets = Vec::new();
    let mut storage_by_bucket = Vec::new();
    
    // Calculate stats for each bucket
    for bucket in &buckets {
        let objects = metadata
            .list_objects(&bucket.name, "", "", 10000)
            .await
            .unwrap_or_default();
        
        let bucket_objects = objects.len() as i64;
        let bucket_size: i64 = objects.iter().map(|o| o.size).sum();
        
        total_objects += bucket_objects;
        total_size += bucket_size;
        
        // Get versioning status
        let versioning = metadata
            .get_bucket_versioning(&bucket.name)
            .await
            .unwrap_or(None);
        let versioning_enabled = versioning.as_deref() == Some("Enabled");
        
        recent_buckets.push(BucketSummary {
            name: bucket.name.clone(),
            object_count: bucket_objects,
            size: bucket_size,
            created_at: bucket.created_at.to_rfc3339(),
            versioning_enabled,
            encryption_enabled: false, // TODO: Check encryption config
        });
        
        storage_by_bucket.push(BucketStorageInfo {
            name: bucket.name.clone(),
            size: bucket_size,
            percentage: 0.0, // Will calculate after total is known
        });
    }
    
    // Calculate percentages
    for info in &mut storage_by_bucket {
        if total_size > 0 {
            info.percentage = (info.size as f64 / total_size as f64) * 100.0;
        }
    }
    
    // Sort by creation date (newest first) and take top 5
    recent_buckets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    recent_buckets.truncate(5);
    
    // Get user count
    let total_users = metadata
        .list_credentials()
        .await
        .map(|c| c.len() as i64)
        .unwrap_or(1); // At least admin user
    
    Ok(Json(DashboardStats {
        total_buckets,
        total_objects,
        total_size,
        total_users,
        recent_buckets,
        storage_by_bucket,
    }))
}

/// Get storage statistics
pub async fn get_storage_stats(
    State(state): State<AppState>,
) -> Result<Json<StorageStats>, (StatusCode, String)> {
    let metadata = &state.metadata;
    
    let buckets = metadata
        .list_buckets()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut total_size: i64 = 0;
    let mut total_objects: i64 = 0;
    let mut largest_bucket: Option<String> = None;
    let mut largest_bucket_size: i64 = 0;
    let mut type_stats: std::collections::HashMap<String, (i64, i64)> = std::collections::HashMap::new();
    
    for bucket in &buckets {
        let objects = metadata
            .list_objects(&bucket.name, "", "", 10000)
            .await
            .unwrap_or_default();
        
        let bucket_size: i64 = objects.iter().map(|o| o.size).sum();
        let bucket_objects = objects.len() as i64;
        
        total_size += bucket_size;
        total_objects += bucket_objects;
        
        if bucket_size > largest_bucket_size {
            largest_bucket_size = bucket_size;
            largest_bucket = Some(bucket.name.clone());
        }
        
        // Aggregate by content type
        for obj in &objects {
            let content_type = obj.content_type.clone().unwrap_or_else(|| "application/octet-stream".to_string());
            let entry = type_stats.entry(content_type).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += obj.size;
        }
    }
    
    let average_object_size = if total_objects > 0 {
        total_size / total_objects
    } else {
        0
    };
    
    let mut storage_by_type: Vec<StorageByType> = type_stats
        .into_iter()
        .map(|(content_type, (count, size))| StorageByType {
            content_type,
            count,
            size,
        })
        .collect();
    
    // Sort by size descending
    storage_by_type.sort_by(|a, b| b.size.cmp(&a.size));
    storage_by_type.truncate(10);
    
    Ok(Json(StorageStats {
        total_size,
        total_objects,
        average_object_size,
        largest_bucket,
        largest_bucket_size,
        storage_by_type,
    }))
}

/// List buckets with detailed information
pub async fn list_buckets_detailed(
    State(state): State<AppState>,
) -> Result<Json<Vec<BucketDetailed>>, (StatusCode, String)> {
    let metadata = &state.metadata;
    
    let buckets = metadata
        .list_buckets()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let mut result = Vec::new();
    
    for bucket in buckets {
        let objects = metadata
            .list_objects(&bucket.name, "", "", 10000)
            .await
            .unwrap_or_default();
        
        let object_count = objects.len() as i64;
        let size: i64 = objects.iter().map(|o| o.size).sum();
        
        // Get versioning status
        let versioning = metadata
            .get_bucket_versioning(&bucket.name)
            .await
            .unwrap_or(None);
        let versioning_status = versioning.clone().unwrap_or_else(|| "Disabled".to_string());
        let versioning_enabled = versioning.as_deref() == Some("Enabled");
        
        // Get tags
        let tags_map = metadata
            .get_bucket_tags(&bucket.name)
            .await
            .unwrap_or_default();
        let tags: Vec<BucketTag> = tags_map
            .into_iter()
            .map(|(key, value)| BucketTag { key, value })
            .collect();
        
        // Get lifecycle rules count
        let lifecycle_rules = metadata
            .get_lifecycle_rules(&bucket.name)
            .await
            .map(|rules| rules.len() as i64)
            .unwrap_or(0);
        
        result.push(BucketDetailed {
            name: bucket.name,
            object_count,
            size,
            created_at: bucket.created_at.to_rfc3339(),
            versioning_enabled,
            versioning_status,
            encryption_enabled: false, // TODO
            lifecycle_rules,
            tags,
        });
    }
    
    // Sort by name
    result.sort_by(|a, b| a.name.cmp(&b.name));
    
    Ok(Json(result))
}

/// Get statistics for a specific bucket
pub async fn get_bucket_stats(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<BucketStats>, (StatusCode, String)> {
    let metadata = &state.metadata;
    
    // Check bucket exists
    let bucket = metadata
        .get_bucket(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, format!("Bucket '{}' not found", name)))?;
    
    // Get objects
    let objects = metadata
        .list_objects(&name, "", "", 10000)
        .await
        .unwrap_or_default();
    
    let object_count = objects.len() as i64;
    let total_size: i64 = objects.iter().map(|o| o.size).sum();
    
    // Get versions count
    let version_count = metadata
        .list_object_versions(&name, "", "", 10000)
        .await
        .map(|v| v.len() as i64)
        .unwrap_or(object_count);
    
    // Get delete markers
    let delete_marker_count = metadata
        .list_delete_markers(&name, "", 10000)
        .await
        .map(|d| d.len() as i64)
        .unwrap_or(0);
    
    // Get multipart uploads count
    let multipart_uploads = metadata
        .list_multipart_uploads(&name, "", "", 10000)
        .await
        .map(|u| u.len() as i64)
        .unwrap_or(0);
    
    // Last modified (most recent object)
    let last_modified = objects
        .iter()
        .map(|o| &o.last_modified)
        .max()
        .map(|d| d.to_rfc3339());
    
    Ok(Json(BucketStats {
        name,
        object_count,
        total_size,
        version_count,
        delete_marker_count,
        multipart_uploads,
        created_at: bucket.created_at.to_rfc3339(),
        last_modified,
    }))
}
