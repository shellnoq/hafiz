//! API client implementation
//!
//! Makes HTTP requests to the Hafiz Admin API.

use super::types::*;
use gloo_net::http::Request;
use web_sys::window;

/// Base URL for Admin API
fn api_base() -> String {
    // Check localStorage for custom API URL, default to relative path
    if let Some(storage) = window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        if let Ok(Some(url)) = storage.get_item("hafiz_api_url") {
            return url;
        }
    }
    "/api/v1".to_string()
}

/// Get credentials from localStorage
fn get_auth_header() -> Option<String> {
    let storage = window()?.local_storage().ok()??;
    let access_key = storage.get_item("hafiz_access_key").ok()??;
    let secret_key = storage.get_item("hafiz_secret_key").ok()??;

    // Use Basic auth: base64(access_key:secret_key)
    let credentials = format!("{}:{}", access_key, secret_key);
    let encoded = base64_encode(&credentials);
    Some(format!("Basic {}", encoded))
}

/// Simple base64 encoding for browser
fn base64_encode(input: &str) -> String {
    let window = window().expect("no window");
    window.btoa(input).unwrap_or_default()
}

/// Make authenticated GET request
async fn get<T: serde::de::DeserializeOwned>(endpoint: &str) -> Result<T, ApiError> {
    let url = format!("{}{}", api_base(), endpoint);

    let mut request = Request::get(&url);

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Request failed with status {}", status)
            } else {
                text
            },
        });
    }

    response.json().await.map_err(|e| ApiError {
        code: "ParseError".to_string(),
        message: e.to_string(),
    })
}

/// Make authenticated POST request
async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
    endpoint: &str,
    body: &B,
) -> Result<T, ApiError> {
    let url = format!("{}{}", api_base(), endpoint);

    let mut request = Request::post(&url)
        .header("Content-Type", "application/json");

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .body(serde_json::to_string(body).unwrap_or_default())
        .map_err(|e| ApiError {
            code: "SerializeError".to_string(),
            message: e.to_string(),
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Request failed with status {}", status)
            } else {
                text
            },
        });
    }

    response.json().await.map_err(|e| ApiError {
        code: "ParseError".to_string(),
        message: e.to_string(),
    })
}

/// Make authenticated PUT request
async fn put<T: serde::de::DeserializeOwned, B: serde::Serialize>(
    endpoint: &str,
    body: &B,
) -> Result<T, ApiError> {
    let url = format!("{}{}", api_base(), endpoint);

    let mut request = Request::put(&url)
        .header("Content-Type", "application/json");

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .body(serde_json::to_string(body).unwrap_or_default())
        .map_err(|e| ApiError {
            code: "SerializeError".to_string(),
            message: e.to_string(),
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Request failed with status {}", status)
            } else {
                text
            },
        });
    }

    response.json().await.map_err(|e| ApiError {
        code: "ParseError".to_string(),
        message: e.to_string(),
    })
}

/// Make authenticated DELETE request
async fn delete(endpoint: &str) -> Result<(), ApiError> {
    let url = format!("{}{}", api_base(), endpoint);

    let mut request = Request::delete(&url);

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Request failed with status {}", status)
            } else {
                text
            },
        });
    }

    Ok(())
}

// ============= Dashboard & Stats =============

/// Fetch dashboard statistics
pub async fn get_dashboard_stats() -> Result<DashboardStats, ApiError> {
    #[derive(serde::Deserialize)]
    struct ApiDashboardStats {
        total_buckets: i64,
        total_objects: i64,
        total_size: i64,
        total_users: i64,
        recent_buckets: Vec<ApiBucketSummary>,
        #[serde(default)]
        storage_by_bucket: Vec<ApiBucketStorage>,
    }

    #[derive(serde::Deserialize)]
    struct ApiBucketSummary {
        name: String,
        object_count: i64,
        size: i64,
        created_at: String,
        versioning_enabled: bool,
        encryption_enabled: bool,
    }

    #[derive(serde::Deserialize)]
    struct ApiBucketStorage {
        name: String,
        size: i64,
        percentage: f64,
    }

    let stats: ApiDashboardStats = get("/stats").await?;

    Ok(DashboardStats {
        total_buckets: stats.total_buckets,
        total_objects: stats.total_objects,
        total_size: stats.total_size,
        total_users: stats.total_users,
        recent_buckets: stats.recent_buckets
            .into_iter()
            .map(|b| BucketInfo {
                name: b.name,
                object_count: b.object_count,
                size: b.size,
                created_at: b.created_at,
                versioning_enabled: b.versioning_enabled,
                encryption_enabled: b.encryption_enabled,
            })
            .collect(),
    })
}

// ============= Bucket Operations =============

/// List all buckets with details
pub async fn list_buckets() -> Result<Vec<BucketInfo>, ApiError> {
    #[derive(serde::Deserialize)]
    struct ApiBucketDetailed {
        name: String,
        object_count: i64,
        size: i64,
        created_at: String,
        versioning_enabled: bool,
        #[serde(default)]
        versioning_status: String,
        encryption_enabled: bool,
        #[serde(default)]
        lifecycle_rules: i64,
        #[serde(default)]
        tags: Vec<ApiTag>,
    }

    #[derive(serde::Deserialize)]
    struct ApiTag {
        key: String,
        value: String,
    }

    let buckets: Vec<ApiBucketDetailed> = get("/buckets").await?;

    Ok(buckets
        .into_iter()
        .map(|b| BucketInfo {
            name: b.name,
            object_count: b.object_count,
            size: b.size,
            created_at: b.created_at,
            versioning_enabled: b.versioning_enabled,
            encryption_enabled: b.encryption_enabled,
        })
        .collect())
}

/// Get bucket statistics
pub async fn get_bucket(name: &str) -> Result<BucketInfo, ApiError> {
    #[derive(serde::Deserialize)]
    struct ApiBucketStats {
        name: String,
        object_count: i64,
        total_size: i64,
        #[serde(default)]
        version_count: i64,
        #[serde(default)]
        delete_marker_count: i64,
        #[serde(default)]
        multipart_uploads: i64,
        created_at: String,
        #[serde(default)]
        last_modified: Option<String>,
    }

    let stats: ApiBucketStats = get(&format!("/buckets/{}/stats", name)).await?;

    Ok(BucketInfo {
        name: stats.name,
        object_count: stats.object_count,
        size: stats.total_size,
        created_at: stats.created_at,
        versioning_enabled: stats.version_count > stats.object_count,
        encryption_enabled: false,
    })
}

/// Create a new bucket (via S3 API)
pub async fn create_bucket(name: &str) -> Result<BucketInfo, ApiError> {
    // Validate name
    if name.is_empty() {
        return Err(ApiError {
            code: "InvalidBucketName".to_string(),
            message: "Bucket name cannot be empty".to_string(),
        });
    }

    if name.len() < 3 || name.len() > 63 {
        return Err(ApiError {
            code: "InvalidBucketName".to_string(),
            message: "Bucket name must be between 3 and 63 characters".to_string(),
        });
    }

    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '.') {
        return Err(ApiError {
            code: "InvalidBucketName".to_string(),
            message: "Bucket name must contain only lowercase letters, numbers, hyphens, and dots".to_string(),
        });
    }

    // Create bucket via S3 PUT /{bucket}
    let s3_url = format!("/{}", name);

    let mut request = Request::put(&s3_url);

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: format!("Failed to create bucket: status {}", status),
        });
    }

    Ok(BucketInfo {
        name: name.to_string(),
        object_count: 0,
        size: 0,
        created_at: chrono::Utc::now().to_rfc3339(),
        versioning_enabled: false,
        encryption_enabled: false,
    })
}

/// Delete a bucket (via S3 API)
pub async fn delete_bucket(name: &str) -> Result<(), ApiError> {
    let s3_url = format!("/{}", name);

    let mut request = Request::delete(&s3_url);

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if status == 409 {
                "Bucket is not empty".to_string()
            } else {
                format!("Failed to delete bucket: status {}", status)
            },
        });
    }

    Ok(())
}

// ============= Object Operations =============

/// List objects in a bucket (via S3 API)
pub async fn list_objects(bucket: &str, prefix: &str) -> Result<ObjectListing, ApiError> {
    let url = if prefix.is_empty() {
        format!("/{}?list-type=2&delimiter=/", bucket)
    } else {
        format!("/{}?list-type=2&delimiter=/&prefix={}", bucket, urlencoding::encode(prefix))
    };

    let mut request = Request::get(&url);

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: format!("Failed to list objects: status {}", status),
        });
    }

    let xml = response.text().await.map_err(|e| ApiError {
        code: "ParseError".to_string(),
        message: e.to_string(),
    })?;

    parse_list_objects_response(&xml)
}

/// Parse S3 ListObjectsV2 XML response
fn parse_list_objects_response(xml: &str) -> Result<ObjectListing, ApiError> {
    let mut objects = Vec::new();
    let mut common_prefixes = Vec::new();
    let mut is_truncated = false;
    let mut next_marker = None;

    // Extract Contents
    for content in xml.split("<Contents>").skip(1) {
        if let Some(end) = content.find("</Contents>") {
            let content_xml = &content[..end];

            let key = extract_xml_value(content_xml, "Key").unwrap_or_default();
            let size = extract_xml_value(content_xml, "Size")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let etag = extract_xml_value(content_xml, "ETag")
                .unwrap_or_default()
                .trim_matches('"')
                .to_string();
            let last_modified = extract_xml_value(content_xml, "LastModified")
                .unwrap_or_default();

            objects.push(ObjectInfo {
                key,
                size,
                etag,
                content_type: "application/octet-stream".to_string(),
                last_modified,
                version_id: None,
                encryption: None,
            });
        }
    }

    // Extract CommonPrefixes
    for prefix in xml.split("<CommonPrefixes>").skip(1) {
        if let Some(end) = prefix.find("</CommonPrefixes>") {
            let prefix_xml = &prefix[..end];
            if let Some(p) = extract_xml_value(prefix_xml, "Prefix") {
                common_prefixes.push(p);
            }
        }
    }

    // Check truncation
    if let Some(truncated) = extract_xml_value(xml, "IsTruncated") {
        is_truncated = truncated == "true";
    }

    if let Some(marker) = extract_xml_value(xml, "NextContinuationToken") {
        next_marker = Some(marker);
    }

    Ok(ObjectListing {
        objects,
        common_prefixes,
        is_truncated,
        next_marker,
    })
}

/// Extract value from XML tag
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);

    let start = xml.find(&start_tag)? + start_tag.len();
    let end = xml.find(&end_tag)?;

    if start < end {
        Some(xml[start..end].to_string())
    } else {
        None
    }
}

// ============= User Operations =============

/// List all users
pub async fn list_users() -> Result<Vec<UserInfo>, ApiError> {
    #[derive(serde::Deserialize)]
    struct ApiUserList {
        users: Vec<ApiUserInfo>,
        total: i64,
    }

    #[derive(serde::Deserialize)]
    struct ApiUserInfo {
        name: String,
        access_key: String,
        email: Option<String>,
        enabled: bool,
        created_at: String,
        #[serde(default)]
        last_used: Option<String>,
        #[serde(default)]
        policies: Vec<String>,
    }

    let response: ApiUserList = get("/users").await?;

    Ok(response.users
        .into_iter()
        .map(|u| UserInfo {
            name: u.name,
            access_key: u.access_key,
            email: u.email,
            enabled: u.enabled,
            created_at: u.created_at,
        })
        .collect())
}

/// Create a new user
pub async fn create_user(name: &str, email: Option<&str>) -> Result<(String, String), ApiError> {
    if name.is_empty() {
        return Err(ApiError {
            code: "InvalidUserName".to_string(),
            message: "Username cannot be empty".to_string(),
        });
    }

    #[derive(serde::Serialize)]
    struct CreateUserRequest {
        name: String,
        email: Option<String>,
    }

    #[derive(serde::Deserialize)]
    struct CreateUserResponse {
        name: String,
        access_key: String,
        secret_key: String,
        email: Option<String>,
        created_at: String,
    }

    let request = CreateUserRequest {
        name: name.to_string(),
        email: email.map(|e| e.to_string()),
    };

    let response: CreateUserResponse = post("/users", &request).await?;

    Ok((response.access_key, response.secret_key))
}

/// Delete a user
pub async fn delete_user(access_key: &str) -> Result<(), ApiError> {
    delete(&format!("/users/{}", access_key)).await
}

/// Delete credentials (alias for delete_user)
pub async fn delete_credentials(access_key: &str) -> Result<(), ApiError> {
    delete(&format!("/users/{}", access_key)).await
}

/// Update credentials (enable/disable)
pub async fn update_credentials(access_key: &str, enabled: bool) -> Result<(), ApiError> {
    if enabled {
        enable_user(access_key).await
    } else {
        disable_user(access_key).await
    }
}

/// Enable a user
pub async fn enable_user(access_key: &str) -> Result<(), ApiError> {
    let url = format!("{}/users/{}/enable", api_base(), access_key);

    let mut request = Request::post(&url);

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        return Err(ApiError {
            code: format!("HTTP{}", response.status()),
            message: "Failed to enable user".to_string(),
        });
    }

    Ok(())
}

/// Disable a user
pub async fn disable_user(access_key: &str) -> Result<(), ApiError> {
    let url = format!("{}/users/{}/disable", api_base(), access_key);

    let mut request = Request::post(&url);

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        return Err(ApiError {
            code: format!("HTTP{}", response.status()),
            message: "Failed to disable user".to_string(),
        });
    }

    Ok(())
}

// ============= Server Operations =============

/// Get server information
pub async fn get_server_info() -> Result<ServerInfo, ApiError> {
    #[derive(serde::Deserialize)]
    struct ApiServerInfo {
        version: String,
        s3_endpoint: String,
        admin_endpoint: String,
        storage_backend: String,
        database_type: String,
        uptime: String,
        #[serde(default)]
        features: Option<ApiFeatures>,
    }

    #[derive(serde::Deserialize)]
    struct ApiFeatures {
        versioning: bool,
        multipart_upload: bool,
        server_side_encryption: bool,
        customer_encryption: bool,
        lifecycle: bool,
        tagging: bool,
    }

    let info: ApiServerInfo = get("/server/info").await?;

    Ok(ServerInfo {
        version: info.version,
        s3_endpoint: info.s3_endpoint,
        admin_endpoint: info.admin_endpoint,
        storage_backend: info.storage_backend,
        database_type: info.database_type,
        uptime: info.uptime,
    })
}

/// Health check
pub async fn health_check() -> Result<HealthStatus, ApiError> {
    #[derive(serde::Deserialize)]
    struct ApiHealth {
        status: String,
        checks: ApiChecks,
        timestamp: String,
    }

    #[derive(serde::Deserialize)]
    struct ApiChecks {
        storage: ApiCheckStatus,
        database: ApiCheckStatus,
        memory: ApiCheckStatus,
    }

    #[derive(serde::Deserialize)]
    struct ApiCheckStatus {
        status: String,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        latency_ms: Option<u64>,
    }

    let health: ApiHealth = get("/server/health").await?;

    Ok(HealthStatus {
        status: health.status,
        storage_ok: health.checks.storage.status == "ok",
        database_ok: health.checks.database.status == "ok",
        timestamp: health.timestamp,
    })
}

// ============= Authentication =============

/// Validate credentials by calling server info
pub async fn validate_credentials(access_key: &str, secret_key: &str) -> Result<bool, ApiError> {
    // Temporarily store credentials
    if let Some(storage) = window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.set_item("hafiz_access_key", access_key);
        let _ = storage.set_item("hafiz_secret_key", secret_key);
    }

    // Try to get server info - if works, credentials are valid
    match get_server_info().await {
        Ok(_) => Ok(true),
        Err(e) if e.code.contains("401") || e.code.contains("403") => {
            // Clear invalid credentials
            if let Some(storage) = window()
                .and_then(|w| w.local_storage().ok())
                .flatten()
            {
                let _ = storage.remove_item("hafiz_access_key");
                let _ = storage.remove_item("hafiz_secret_key");
            }
            Ok(false)
        }
        Err(e) => Err(e),
    }
}

/// Logout - clear stored credentials
pub fn logout() {
    if let Some(storage) = window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
    {
        let _ = storage.remove_item("hafiz_access_key");
        let _ = storage.remove_item("hafiz_secret_key");
    }
}

/// Check if user is logged in
pub fn is_logged_in() -> bool {
    window()
        .and_then(|w| w.local_storage().ok())
        .flatten()
        .and_then(|s| s.get_item("hafiz_access_key").ok())
        .flatten()
        .is_some()
}

// ============= Object Upload =============

/// Upload an object to a bucket using S3 PUT API
pub async fn upload_object(bucket: &str, key: &str, file: web_sys::File) -> Result<(), ApiError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use js_sys::{ArrayBuffer, Uint8Array};

    // Read file contents
    let array_buffer_promise = file.array_buffer();
    let array_buffer = JsFuture::from(array_buffer_promise)
        .await
        .map_err(|e| ApiError {
            code: "FileReadError".to_string(),
            message: format!("Failed to read file: {:?}", e),
        })?;

    let array_buffer: ArrayBuffer = array_buffer.unchecked_into();
    let uint8_array = Uint8Array::new(&array_buffer);
    let bytes: Vec<u8> = uint8_array.to_vec();

    // Build S3 PUT URL (root path, not /api/v1)
    let encoded_key = urlencoding::encode(key);
    let url = format!("/{}/{}", bucket, encoded_key);

    // Get auth header
    let auth = get_auth_header().ok_or_else(|| ApiError {
        code: "AuthError".to_string(),
        message: "Not authenticated".to_string(),
    })?;

    // Get content type from file
    let content_type = file.type_();
    let content_type = if content_type.is_empty() {
        "application/octet-stream".to_string()
    } else {
        content_type
    };

    // Make PUT request
    let response = Request::put(&url)
        .header("Authorization", &auth)
        .header("Content-Type", &content_type)
        .body(bytes)
        .map_err(|e| ApiError {
            code: "RequestError".to_string(),
            message: e.to_string(),
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Upload failed with status {}", status)
            } else {
                text
            },
        });
    }

    Ok(())
}

/// Delete an object from a bucket using S3 DELETE API
pub async fn delete_object(bucket: &str, key: &str) -> Result<(), ApiError> {
    let encoded_key = urlencoding::encode(key);
    let url = format!("/{}/{}", bucket, encoded_key);

    let auth = get_auth_header().ok_or_else(|| ApiError {
        code: "AuthError".to_string(),
        message: "Not authenticated".to_string(),
    })?;

    let response = Request::delete(&url)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Delete failed with status {}", status)
            } else {
                text
            },
        });
    }

    Ok(())
}

/// Download an object from a bucket
pub async fn download_object(bucket: &str, key: &str) -> Result<Vec<u8>, ApiError> {
    let encoded_key = urlencoding::encode(key);
    let url = format!("/{}/{}", bucket, encoded_key);

    let auth = get_auth_header().ok_or_else(|| ApiError {
        code: "AuthError".to_string(),
        message: "Not authenticated".to_string(),
    })?;

    let response = Request::get(&url)
        .header("Authorization", &auth)
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Download failed with status {}", status)
            } else {
                text
            },
        });
    }

    response.binary().await.map_err(|e| ApiError {
        code: "ReadError".to_string(),
        message: e.to_string(),
    })
}

// ============================================================================
// Cluster API
// ============================================================================

/// Get cluster status
pub async fn get_cluster_status() -> Result<ClusterStatus, ApiError> {
    get("/cluster/status").await
}

/// Get cluster health
pub async fn get_cluster_health() -> Result<ClusterHealth, ApiError> {
    get("/cluster/health").await
}

/// List cluster nodes
pub async fn list_cluster_nodes() -> Result<NodesList, ApiError> {
    get("/cluster/nodes").await
}

/// Get a specific node
pub async fn get_cluster_node(node_id: &str) -> Result<NodeInfo, ApiError> {
    get(&format!("/cluster/nodes/{}", node_id)).await
}

/// Drain a node
pub async fn drain_cluster_node(node_id: &str) -> Result<serde_json::Value, ApiError> {
    post(&format!("/cluster/nodes/{}/drain", node_id), &serde_json::json!({})).await
}

/// Remove a node from cluster
pub async fn remove_cluster_node(node_id: &str) -> Result<(), ApiError> {
    delete(&format!("/cluster/nodes/{}", node_id)).await
}

/// List replication rules
pub async fn list_replication_rules() -> Result<ReplicationRulesList, ApiError> {
    get("/cluster/replication/rules").await
}

/// Create a replication rule
pub async fn create_replication_rule(request: &CreateReplicationRuleRequest) -> Result<ReplicationRule, ApiError> {
    post("/cluster/replication/rules", request).await
}

/// Get a specific replication rule
pub async fn get_replication_rule(rule_id: &str) -> Result<ReplicationRule, ApiError> {
    get(&format!("/cluster/replication/rules/{}", rule_id)).await
}

/// Delete a replication rule
pub async fn delete_replication_rule(rule_id: &str) -> Result<(), ApiError> {
    delete(&format!("/cluster/replication/rules/{}", rule_id)).await
}

/// Get replication statistics
pub async fn get_replication_stats() -> Result<ReplicationStats, ApiError> {
    get("/cluster/replication/stats").await
}

// ============================================================================
// Pre-signed URLs API
// ============================================================================

/// Generate a pre-signed URL
pub async fn generate_presigned_url(request: &PresignedUrlRequest) -> Result<PresignedUrlResponse, ApiError> {
    post("/presigned", request).await
}

/// Generate a pre-signed download URL (shortcut)
pub async fn generate_presigned_download(bucket: &str, key: &str) -> Result<PresignedUrlResponse, ApiError> {
    let encoded_key = urlencoding::encode(key);
    post_empty(&format!("/presigned/download/{}/{}", bucket, encoded_key)).await
}

/// Generate a pre-signed upload URL (shortcut)
pub async fn generate_presigned_upload(bucket: &str, key: &str) -> Result<PresignedUrlResponse, ApiError> {
    let encoded_key = urlencoding::encode(key);
    post_empty(&format!("/presigned/upload/{}/{}", bucket, encoded_key)).await
}

/// POST request without body
async fn post_empty<T: serde::de::DeserializeOwned>(endpoint: &str) -> Result<T, ApiError> {
    let url = format!("{}{}", api_base(), endpoint);

    let mut request = Request::post(&url)
        .header("Content-Type", "application/json");

    if let Some(auth) = get_auth_header() {
        request = request.header("Authorization", &auth);
    }

    let response = request
        .body("{}")
        .map_err(|e| ApiError {
            code: "RequestError".to_string(),
            message: e.to_string(),
        })?
        .send()
        .await
        .map_err(|e| ApiError {
            code: "NetworkError".to_string(),
            message: e.to_string(),
        })?;

    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(ApiError {
            code: format!("HTTP{}", status),
            message: if text.is_empty() {
                format!("Request failed with status {}", status)
            } else {
                text
            },
        });
    }

    response.json().await.map_err(|e| ApiError {
        code: "ParseError".to_string(),
        message: e.to_string(),
    })
}

// ============= LDAP API =============

/// Get LDAP status
pub async fn get_ldap_status() -> Result<LdapStatus, ApiError> {
    let response: ApiResponse<LdapStatus> = get("/ldap/status").await?;
    response.data.ok_or_else(|| ApiError {
        code: "NoData".to_string(),
        message: response.error.unwrap_or_else(|| "No data returned".to_string()),
    })
}

/// Get LDAP configuration (sanitized)
pub async fn get_ldap_config() -> Result<LdapConfig, ApiError> {
    let response: ApiResponse<LdapConfig> = get("/ldap/config").await?;
    response.data.ok_or_else(|| ApiError {
        code: "NoData".to_string(),
        message: response.error.unwrap_or_else(|| "No data returned".to_string()),
    })
}

/// Update LDAP configuration
pub async fn update_ldap_config(config: &LdapConfig) -> Result<(), ApiError> {
    let _response: ApiResponse<serde_json::Value> = put("/ldap/config", config).await?;
    Ok(())
}

/// Test LDAP connection
pub async fn test_ldap_connection(request: &TestLdapConnectionRequest) -> Result<TestLdapConnectionResponse, ApiError> {
    let response: ApiResponse<TestLdapConnectionResponse> = post("/ldap/test-connection", request).await?;
    response.data.ok_or_else(|| ApiError {
        code: "NoData".to_string(),
        message: response.error.unwrap_or_else(|| "No data returned".to_string()),
    })
}

/// Test LDAP user search
pub async fn test_ldap_search(request: &TestLdapSearchRequest) -> Result<TestLdapSearchResponse, ApiError> {
    let response: ApiResponse<TestLdapSearchResponse> = post("/ldap/test-search", request).await?;
    response.data.ok_or_else(|| ApiError {
        code: "NoData".to_string(),
        message: response.error.unwrap_or_else(|| "No data returned".to_string()),
    })
}

/// Test LDAP authentication
pub async fn test_ldap_auth(request: &TestLdapAuthRequest) -> Result<TestLdapAuthResponse, ApiError> {
    let response: ApiResponse<TestLdapAuthResponse> = post("/ldap/authenticate", request).await?;
    response.data.ok_or_else(|| ApiError {
        code: "NoData".to_string(),
        message: response.error.unwrap_or_else(|| "No data returned".to_string()),
    })
}

/// Clear LDAP cache
pub async fn clear_ldap_cache() -> Result<(), ApiError> {
    let _response: ApiResponse<serde_json::Value> = post("/ldap/clear-cache", &()).await?;
    Ok(())
}
