//! Metadata repository

use chrono::{DateTime, Utc};
use hafiz_core::types::{
    Bucket, BucketInfo, Object, ObjectInfo, User, VersioningStatus, 
    ObjectVersion, DeleteMarker, Tag, TagSet, LifecycleConfiguration, LifecycleRule,
    EncryptionInfo, EncryptionType
};
use hafiz_core::{Error, Result};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use tracing::{debug, info};

pub struct MetadataStore {
    pool: SqlitePool,
}

impl MetadataStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(100)
            .connect(database_url)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let store = Self { pool };
        store.init().await?;
        
        Ok(store)
    }

    async fn init(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                access_key TEXT UNIQUE NOT NULL,
                secret_key TEXT NOT NULL,
                display_name TEXT,
                email TEXT,
                is_admin INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS buckets (
                name TEXT PRIMARY KEY,
                owner_id TEXT NOT NULL,
                region TEXT NOT NULL,
                versioning TEXT DEFAULT '',
                object_lock_enabled INTEGER DEFAULT 0,
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Objects table with versioning support
        // version_id: "null" for non-versioned, UUID for versioned
        // is_latest: 1 for current version, 0 for old versions
        // is_delete_marker: 1 if this is a delete marker
        // encryption: JSON containing encryption info
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS objects (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                version_id TEXT NOT NULL DEFAULT 'null',
                size INTEGER NOT NULL,
                etag TEXT NOT NULL,
                content_type TEXT NOT NULL,
                metadata TEXT,
                last_modified TEXT NOT NULL,
                is_latest INTEGER DEFAULT 1,
                is_delete_marker INTEGER DEFAULT 0,
                encryption TEXT,
                PRIMARY KEY (bucket, key, version_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_objects_bucket ON objects(bucket)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_objects_latest ON objects(bucket, key, is_latest)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Object tagging table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS object_tags (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                version_id TEXT NOT NULL DEFAULT 'null',
                tag_key TEXT NOT NULL,
                tag_value TEXT NOT NULL,
                PRIMARY KEY (bucket, key, version_id, tag_key)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Bucket lifecycle configuration table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bucket_lifecycle (
                bucket TEXT PRIMARY KEY,
                configuration TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Bucket policy table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bucket_policies (
                bucket TEXT PRIMARY KEY,
                policy_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Bucket ACL table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bucket_acls (
                bucket TEXT PRIMARY KEY,
                acl_xml TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Object ACL table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS object_acls (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                version_id TEXT NOT NULL DEFAULT 'null',
                acl_xml TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                PRIMARY KEY (bucket, key, version_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        info!("Metadata store initialized with versioning, tagging, lifecycle, policy, and ACL support");
        Ok(())
    }

    // User operations
    pub async fn create_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, access_key, secret_key, display_name, email, is_admin, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&user.id)
        .bind(&user.access_key)
        .bind(&user.secret_key)
        .bind(&user.display_name)
        .bind(&user.email)
        .bind(user.is_admin)
        .bind(user.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Created user: {}", user.access_key);
        Ok(())
    }

    pub async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<User>> {
        let row: Option<(String, String, String, Option<String>, Option<String>, bool, String)> =
            sqlx::query_as(
                r#"
                SELECT id, access_key, secret_key, display_name, email, is_admin, created_at
                FROM users WHERE access_key = ?
                "#,
            )
            .bind(access_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| User {
            id: r.0,
            access_key: r.1,
            secret_key: r.2,
            display_name: r.3,
            email: r.4,
            is_admin: r.5,
            created_at: DateTime::parse_from_rfc3339(&r.6)
                .unwrap()
                .with_timezone(&Utc),
        }))
    }

    // Bucket operations
    pub async fn create_bucket(&self, bucket: &Bucket) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO buckets (name, owner_id, region, versioning, object_lock_enabled, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&bucket.name)
        .bind(&bucket.owner_id)
        .bind(&bucket.region)
        .bind(bucket.versioning.as_str())
        .bind(bucket.object_lock_enabled as i32)
        .bind(bucket.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint") {
                Error::BucketAlreadyExists
            } else {
                Error::DatabaseError(e.to_string())
            }
        })?;

        debug!("Created bucket: {}", bucket.name);
        Ok(())
    }

    pub async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>> {
        let row: Option<(String, String, String, Option<String>, Option<i32>, String)> = sqlx::query_as(
            r#"
            SELECT name, owner_id, region, versioning, object_lock_enabled, created_at
            FROM buckets WHERE name = ?
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Bucket {
            name: r.0,
            owner_id: r.1,
            region: r.2,
            versioning: VersioningStatus::from_str(r.3.as_deref().unwrap_or("")),
            object_lock_enabled: r.4.unwrap_or(0) != 0,
            created_at: DateTime::parse_from_rfc3339(&r.5)
                .unwrap()
                .with_timezone(&Utc),
        }))
    }

    /// Update bucket versioning status
    pub async fn set_bucket_versioning(&self, name: &str, status: VersioningStatus) -> Result<()> {
        sqlx::query(
            r#"UPDATE buckets SET versioning = ? WHERE name = ?"#,
        )
        .bind(status.as_str())
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Set bucket {} versioning to {:?}", name, status);
        Ok(())
    }

    pub async fn delete_bucket(&self, name: &str) -> Result<()> {
        // Check if bucket has objects (including delete markers)
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM objects WHERE bucket = ?"#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        if count.0 > 0 {
            return Err(Error::BucketNotEmpty);
        }

        sqlx::query(r#"DELETE FROM buckets WHERE name = ?"#)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted bucket: {}", name);
        Ok(())
    }

    pub async fn list_buckets(&self, owner_id: &str) -> Result<Vec<BucketInfo>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT name, created_at FROM buckets WHERE owner_id = ?
            ORDER BY name
            "#,
        )
        .bind(owner_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| BucketInfo {
                name: r.0,
                creation_date: DateTime::parse_from_rfc3339(&r.1)
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect())
    }

    // ============= Object operations (with versioning) =============

    /// Put object - handles both versioned and non-versioned buckets
    pub async fn put_object(&self, object: &Object) -> Result<()> {
        let metadata_json = serde_json::to_string(&object.metadata)
            .map_err(|e| Error::InternalError(e.to_string()))?;
        
        let encryption_json = serde_json::to_string(&object.encryption)
            .map_err(|e| Error::InternalError(e.to_string()))?;

        // Mark all existing versions of this key as non-latest
        sqlx::query(
            r#"UPDATE objects SET is_latest = 0 WHERE bucket = ? AND key = ?"#,
        )
        .bind(&object.bucket)
        .bind(&object.key)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO objects 
            (bucket, key, version_id, size, etag, content_type, metadata, last_modified, is_latest, is_delete_marker, encryption)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&object.bucket)
        .bind(&object.key)
        .bind(&object.version_id)
        .bind(object.size)
        .bind(&object.etag)
        .bind(&object.content_type)
        .bind(&metadata_json)
        .bind(object.last_modified.to_rfc3339())
        .bind(object.is_latest as i32)
        .bind(object.is_delete_marker as i32)
        .bind(&encryption_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Put object: {}/{} version={} encrypted={}", 
            object.bucket, object.key, object.version_id, object.encryption.is_encrypted());
        Ok(())
    }

    /// Get the latest version of an object
    pub async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<Object>> {
        self.get_object_version(bucket, key, None).await
    }

    /// Get a specific version of an object
    pub async fn get_object_version(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<Option<Object>> {
        let row: Option<(String, String, String, i64, String, String, Option<String>, String, i32, i32, Option<String>)> = 
            if let Some(vid) = version_id {
                sqlx::query_as(
                    r#"
                    SELECT bucket, key, version_id, size, etag, content_type, metadata, last_modified, is_latest, is_delete_marker, encryption
                    FROM objects WHERE bucket = ? AND key = ? AND version_id = ?
                    "#,
                )
                .bind(bucket)
                .bind(key)
                .bind(vid)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::DatabaseError(e.to_string()))?
            } else {
                sqlx::query_as(
                    r#"
                    SELECT bucket, key, version_id, size, etag, content_type, metadata, last_modified, is_latest, is_delete_marker, encryption
                    FROM objects WHERE bucket = ? AND key = ? AND is_latest = 1
                    "#,
                )
                .bind(bucket)
                .bind(key)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| Error::DatabaseError(e.to_string()))?
            };

        Ok(row.map(|r| {
            let metadata: HashMap<String, String> = r
                .6
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or_default();
            
            let encryption: EncryptionInfo = r
                .10
                .and_then(|e| serde_json::from_str(&e).ok())
                .unwrap_or_default();

            Object {
                bucket: r.0,
                key: r.1,
                version_id: r.2,
                size: r.3,
                etag: r.4,
                content_type: r.5,
                metadata,
                last_modified: DateTime::parse_from_rfc3339(&r.7)
                    .unwrap()
                    .with_timezone(&Utc),
                is_latest: r.8 != 0,
                is_delete_marker: r.9 != 0,
                encryption,
            }
        }))
    }

    /// Delete object - for non-versioned buckets, removes the object
    /// For versioned buckets, creates a delete marker
    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
        sqlx::query(r#"DELETE FROM objects WHERE bucket = ? AND key = ? AND version_id = 'null'"#)
            .bind(bucket)
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted object: {}/{}", bucket, key);
        Ok(())
    }

    /// List objects - only returns latest non-deleted versions
    pub async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: i32,
        continuation_token: Option<&str>,
    ) -> Result<(Vec<ObjectInfo>, Vec<String>, bool, Option<String>)> {
        let prefix = prefix.unwrap_or("");
        let start_after = continuation_token.unwrap_or("");

        // Only get latest versions that are not delete markers
        let rows: Vec<(String, String, i64, String, String)> = sqlx::query_as(
            r#"
            SELECT key, version_id, size, etag, last_modified
            FROM objects 
            WHERE bucket = ? AND key LIKE ? AND key > ? AND is_latest = 1 AND is_delete_marker = 0
            ORDER BY key
            LIMIT ?
            "#,
        )
        .bind(bucket)
        .bind(format!("{}%", prefix))
        .bind(start_after)
        .bind(max_keys + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let is_truncated = rows.len() > max_keys as usize;
        let rows: Vec<_> = rows.into_iter().take(max_keys as usize).collect();
        
        let next_token = if is_truncated {
            rows.last().map(|r| r.0.clone())
        } else {
            None
        };

        let mut objects = Vec::new();
        let mut common_prefixes = std::collections::HashSet::new();

        for row in rows {
            let key = row.0;
            
            if let Some(delim) = delimiter {
                let suffix = key.strip_prefix(prefix).unwrap_or(&key);
                if let Some(idx) = suffix.find(delim) {
                    let prefix_key = format!("{}{}{}", prefix, &suffix[..idx], delim);
                    common_prefixes.insert(prefix_key);
                    continue;
                }
            }

            objects.push(ObjectInfo {
                key,
                size: row.2,
                etag: row.3,
                last_modified: DateTime::parse_from_rfc3339(&row.4)
                    .unwrap()
                    .with_timezone(&Utc),
                storage_class: "STANDARD".to_string(),
                version_id: Some(row.1),
                is_latest: Some(true),
            });
        }

        let common_prefixes: Vec<String> = common_prefixes.into_iter().collect();

        Ok((objects, common_prefixes, is_truncated, next_token))
    }

    /// List all versions of objects (for versioned buckets)
    pub async fn list_object_versions(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: i32,
        key_marker: Option<&str>,
        version_id_marker: Option<&str>,
    ) -> Result<(Vec<ObjectVersion>, Vec<DeleteMarker>, Vec<String>, bool, Option<String>, Option<String>)> {
        let prefix = prefix.unwrap_or("");
        let key_marker = key_marker.unwrap_or("");

        // Get all versions including delete markers
        let rows: Vec<(String, String, i64, String, String, i32, i32)> = sqlx::query_as(
            r#"
            SELECT key, version_id, size, etag, last_modified, is_latest, is_delete_marker
            FROM objects 
            WHERE bucket = ? AND key LIKE ? AND key >= ?
            ORDER BY key, last_modified DESC
            LIMIT ?
            "#,
        )
        .bind(bucket)
        .bind(format!("{}%", prefix))
        .bind(key_marker)
        .bind(max_keys + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let is_truncated = rows.len() > max_keys as usize;
        let rows: Vec<_> = rows.into_iter().take(max_keys as usize).collect();
        
        let (next_key_marker, next_version_id_marker) = if is_truncated {
            rows.last().map(|r| (Some(r.0.clone()), Some(r.1.clone()))).unwrap_or((None, None))
        } else {
            (None, None)
        };

        let mut versions = Vec::new();
        let mut delete_markers = Vec::new();
        let mut common_prefixes = std::collections::HashSet::new();

        for row in rows {
            let key = row.0;
            let version_id = row.1;
            let is_delete_marker = row.6 != 0;
            
            if let Some(delim) = delimiter {
                let suffix = key.strip_prefix(prefix).unwrap_or(&key);
                if let Some(idx) = suffix.find(delim) {
                    let prefix_key = format!("{}{}{}", prefix, &suffix[..idx], delim);
                    common_prefixes.insert(prefix_key);
                    continue;
                }
            }

            let last_modified = DateTime::parse_from_rfc3339(&row.4)
                .unwrap()
                .with_timezone(&Utc);

            if is_delete_marker {
                delete_markers.push(DeleteMarker {
                    key,
                    version_id,
                    is_latest: row.5 != 0,
                    last_modified,
                    owner_id: "root".to_string(),
                });
            } else {
                versions.push(ObjectVersion {
                    key,
                    version_id,
                    is_latest: row.5 != 0,
                    last_modified,
                    etag: row.3,
                    size: row.2,
                    storage_class: "STANDARD".to_string(),
                    owner_id: "root".to_string(),
                });
            }
        }

        let common_prefixes: Vec<String> = common_prefixes.into_iter().collect();

        Ok((versions, delete_markers, common_prefixes, is_truncated, next_key_marker, next_version_id_marker))
    }

    /// Delete a specific version of an object
    pub async fn delete_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"DELETE FROM objects WHERE bucket = ? AND key = ? AND version_id = ?"#
        )
        .bind(bucket)
        .bind(key)
        .bind(version_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // If we deleted the latest version, mark the next most recent as latest
        if result.rows_affected() > 0 {
            sqlx::query(
                r#"
                UPDATE objects SET is_latest = 1 
                WHERE bucket = ? AND key = ? AND version_id = (
                    SELECT version_id FROM objects 
                    WHERE bucket = ? AND key = ?
                    ORDER BY last_modified DESC
                    LIMIT 1
                )
                "#,
            )
            .bind(bucket)
            .bind(key)
            .bind(bucket)
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;
        }

        debug!("Deleted object version: {}/{} version={}", bucket, key, version_id);
        Ok(result.rows_affected() > 0)
    }

    /// Create a delete marker for versioned delete
    pub async fn create_delete_marker(&self, bucket: &str, key: &str) -> Result<String> {
        let version_id = Object::generate_version_id();
        let delete_marker = Object::as_delete_marker(
            bucket.to_string(),
            key.to_string(),
            version_id.clone(),
        );
        self.put_object(&delete_marker).await?;
        Ok(version_id)
    }

    // ============= Phase 2: Multipart Upload Operations =============

    /// Initialize multipart upload tables
    pub async fn init_multipart_tables(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multipart_uploads (
                upload_id TEXT PRIMARY KEY,
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                content_type TEXT NOT NULL,
                metadata TEXT,
                storage_class TEXT DEFAULT 'STANDARD',
                initiator_id TEXT DEFAULT 'root',
                created_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS upload_parts (
                upload_id TEXT NOT NULL,
                part_number INTEGER NOT NULL,
                size INTEGER NOT NULL,
                etag TEXT NOT NULL,
                created_at TEXT NOT NULL,
                PRIMARY KEY (upload_id, part_number)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_multipart_bucket ON multipart_uploads(bucket, key)
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        info!("Multipart upload tables initialized");
        Ok(())
    }

    /// Create a new multipart upload
    pub async fn create_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        content_type: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String> {
        // Ensure tables exist
        self.init_multipart_tables().await?;

        let upload_id = uuid::Uuid::new_v4().to_string().replace("-", "");
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| Error::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO multipart_uploads (upload_id, bucket, key, content_type, metadata, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&upload_id)
        .bind(bucket)
        .bind(key)
        .bind(content_type)
        .bind(&metadata_json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Created multipart upload: {} for {}/{}", upload_id, bucket, key);
        Ok(upload_id)
    }

    /// Get multipart upload info
    pub async fn get_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        upload_id: &str,
    ) -> Result<Option<MultipartUpload>> {
        let row: Option<(String, String, String, String, Option<String>, String, String, String)> =
            sqlx::query_as(
                r#"
                SELECT upload_id, bucket, key, content_type, metadata, storage_class, initiator_id, created_at
                FROM multipart_uploads
                WHERE upload_id = ? AND bucket = ? AND key = ?
                "#,
            )
            .bind(upload_id)
            .bind(bucket)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            let metadata: HashMap<String, String> = r
                .4
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or_default();

            MultipartUpload {
                upload_id: r.0,
                bucket: r.1,
                key: r.2,
                content_type: r.3,
                metadata,
                storage_class: r.5,
                initiator_id: r.6,
                created_at: DateTime::parse_from_rfc3339(&r.7)
                    .unwrap()
                    .with_timezone(&Utc),
            }
        }))
    }

    /// Delete multipart upload
    pub async fn delete_multipart_upload(&self, upload_id: &str) -> Result<()> {
        // Delete parts first
        sqlx::query(r#"DELETE FROM upload_parts WHERE upload_id = ?"#)
            .bind(upload_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Delete upload record
        sqlx::query(r#"DELETE FROM multipart_uploads WHERE upload_id = ?"#)
            .bind(upload_id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted multipart upload: {}", upload_id);
        Ok(())
    }

    /// Store upload part
    pub async fn put_upload_part(
        &self,
        upload_id: &str,
        part_number: i32,
        size: i64,
        etag: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO upload_parts (upload_id, part_number, size, etag, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(upload_id)
        .bind(part_number)
        .bind(size)
        .bind(etag)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Put upload part: {} part {}", upload_id, part_number);
        Ok(())
    }

    /// List upload parts
    pub async fn list_upload_parts(&self, upload_id: &str) -> Result<Vec<UploadPart>> {
        let rows: Vec<(i32, i64, String, String)> = sqlx::query_as(
            r#"
            SELECT part_number, size, etag, created_at
            FROM upload_parts
            WHERE upload_id = ?
            ORDER BY part_number
            "#,
        )
        .bind(upload_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| UploadPart {
                part_number: r.0,
                size: r.1,
                etag: r.2,
                last_modified: DateTime::parse_from_rfc3339(&r.3)
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect())
    }

    /// List multipart uploads for a bucket
    pub async fn list_multipart_uploads(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        key_marker: Option<&str>,
        upload_id_marker: Option<&str>,
        max_uploads: i32,
    ) -> Result<(Vec<MultipartUploadInfo>, bool)> {
        let prefix = prefix.unwrap_or("");
        let key_marker = key_marker.unwrap_or("");

        let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
            r#"
            SELECT upload_id, key, initiator_id, storage_class, created_at
            FROM multipart_uploads
            WHERE bucket = ? AND key LIKE ? AND key > ?
            ORDER BY key, upload_id
            LIMIT ?
            "#,
        )
        .bind(bucket)
        .bind(format!("{}%", prefix))
        .bind(key_marker)
        .bind(max_uploads + 1)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let is_truncated = rows.len() > max_uploads as usize;
        let rows: Vec<_> = rows.into_iter().take(max_uploads as usize).collect();

        let uploads = rows
            .into_iter()
            .map(|r| MultipartUploadInfo {
                upload_id: r.0,
                key: r.1,
                initiator_id: r.2,
                storage_class: r.3,
                initiated: DateTime::parse_from_rfc3339(&r.4)
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect();

        Ok((uploads, is_truncated))
    }
}

// ============= Phase 2: Multipart Upload Types =============

/// Multipart upload record
#[derive(Debug, Clone)]
pub struct MultipartUpload {
    pub upload_id: String,
    pub bucket: String,
    pub key: String,
    pub content_type: String,
    pub metadata: HashMap<String, String>,
    pub storage_class: String,
    pub initiator_id: String,
    pub created_at: DateTime<Utc>,
}

/// Upload part record
#[derive(Debug, Clone)]
pub struct UploadPart {
    pub part_number: i32,
    pub size: i64,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
}

/// Multipart upload info for listing
#[derive(Debug, Clone)]
pub struct MultipartUploadInfo {
    pub upload_id: String,
    pub key: String,
    pub initiator_id: String,
    pub storage_class: String,
    pub initiated: DateTime<Utc>,
}

// ============= Object Tagging Operations =============

impl MetadataStore {
    /// Put object tags (replaces existing tags)
    pub async fn put_object_tags(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<&str>,
        tags: &TagSet,
    ) -> Result<()> {
        let vid = version_id.unwrap_or("null");

        // Delete existing tags
        sqlx::query(
            r#"DELETE FROM object_tags WHERE bucket = ? AND key = ? AND version_id = ?"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(vid)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Insert new tags
        for tag in &tags.tags {
            sqlx::query(
                r#"
                INSERT INTO object_tags (bucket, key, version_id, tag_key, tag_value)
                VALUES (?, ?, ?, ?, ?)
                "#,
            )
            .bind(bucket)
            .bind(key)
            .bind(vid)
            .bind(&tag.key)
            .bind(&tag.value)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;
        }

        debug!("Put {} tags for {}/{}", tags.len(), bucket, key);
        Ok(())
    }

    /// Get object tags
    pub async fn get_object_tags(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<&str>,
    ) -> Result<TagSet> {
        let vid = version_id.unwrap_or("null");

        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tag_key, tag_value FROM object_tags
            WHERE bucket = ? AND key = ? AND version_id = ?
            "#,
        )
        .bind(bucket)
        .bind(key)
        .bind(vid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let mut tag_set = TagSet::new();
        for (k, v) in rows {
            tag_set.tags.push(Tag::new(k, v));
        }

        Ok(tag_set)
    }

    /// Delete object tags
    pub async fn delete_object_tags(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<&str>,
    ) -> Result<()> {
        let vid = version_id.unwrap_or("null");

        sqlx::query(
            r#"DELETE FROM object_tags WHERE bucket = ? AND key = ? AND version_id = ?"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(vid)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted tags for {}/{}", bucket, key);
        Ok(())
    }
}

// ============= Bucket Lifecycle Operations =============

impl MetadataStore {
    /// Put bucket lifecycle configuration
    pub async fn put_bucket_lifecycle(
        &self,
        bucket: &str,
        config: &LifecycleConfiguration,
    ) -> Result<()> {
        let config_json = serde_json::to_string(config)
            .map_err(|e| Error::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO bucket_lifecycle (bucket, configuration, updated_at)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(bucket)
        .bind(&config_json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Put lifecycle config for bucket {} with {} rules", bucket, config.rules.len());
        Ok(())
    }

    /// Get bucket lifecycle configuration
    pub async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<LifecycleConfiguration>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"SELECT configuration FROM bucket_lifecycle WHERE bucket = ?"#,
        )
        .bind(bucket)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        match row {
            Some((config_json,)) => {
                let config: LifecycleConfiguration = serde_json::from_str(&config_json)
                    .map_err(|e| Error::InternalError(e.to_string()))?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    /// Delete bucket lifecycle configuration
    pub async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<()> {
        sqlx::query(r#"DELETE FROM bucket_lifecycle WHERE bucket = ?"#)
            .bind(bucket)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted lifecycle config for bucket {}", bucket);
        Ok(())
    }

    /// Get all buckets with lifecycle configurations (for lifecycle worker)
    pub async fn get_buckets_with_lifecycle(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT bucket FROM bucket_lifecycle"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// Get objects matching a lifecycle rule filter (for lifecycle worker)
    pub async fn get_objects_for_lifecycle(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        limit: i32,
    ) -> Result<Vec<ObjectWithTags>> {
        let prefix = prefix.unwrap_or("");

        let rows: Vec<(String, String, i64, String, i32, i32)> = sqlx::query_as(
            r#"
            SELECT key, version_id, size, last_modified, is_latest, is_delete_marker
            FROM objects
            WHERE bucket = ? AND key LIKE ? AND is_delete_marker = 0
            ORDER BY key
            LIMIT ?
            "#,
        )
        .bind(bucket)
        .bind(format!("{}%", prefix))
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let mut objects = Vec::new();
        for row in rows {
            let tags = self.get_object_tags(bucket, &row.0, Some(&row.1)).await?;
            objects.push(ObjectWithTags {
                bucket: bucket.to_string(),
                key: row.0,
                version_id: row.1,
                size: row.2,
                last_modified: DateTime::parse_from_rfc3339(&row.3)
                    .unwrap()
                    .with_timezone(&Utc),
                is_latest: row.4 != 0,
                is_delete_marker: row.5 != 0,
                tags,
            });
        }

        Ok(objects)
    }
}

/// Object with tags for lifecycle processing
#[derive(Debug, Clone)]
pub struct ObjectWithTags {
    pub bucket: String,
    pub key: String,
    pub version_id: String,
    pub size: i64,
    pub last_modified: DateTime<Utc>,
    pub is_latest: bool,
    pub is_delete_marker: bool,
    pub tags: TagSet,
}

// ============= Policy and ACL Operations =============

impl MetadataStore {
    /// Store bucket policy JSON
    pub async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        sqlx::query(
            r#"
            INSERT INTO bucket_policies (bucket, policy_json, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(bucket) DO UPDATE SET policy_json = ?, updated_at = ?
            "#,
        )
        .bind(bucket)
        .bind(policy_json)
        .bind(&now)
        .bind(policy_json)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Stored bucket policy for: {}", bucket);
        Ok(())
    }

    /// Get bucket policy JSON
    pub async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"SELECT policy_json FROM bucket_policies WHERE bucket = ?"#,
        )
        .bind(bucket)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| r.0))
    }

    /// Delete bucket policy
    pub async fn delete_bucket_policy(&self, bucket: &str) -> Result<()> {
        sqlx::query(r#"DELETE FROM bucket_policies WHERE bucket = ?"#)
            .bind(bucket)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted bucket policy for: {}", bucket);
        Ok(())
    }

    /// Store bucket ACL XML
    pub async fn put_bucket_acl(&self, bucket: &str, acl_xml: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        
        sqlx::query(
            r#"
            INSERT INTO bucket_acls (bucket, acl_xml, updated_at)
            VALUES (?, ?, ?)
            ON CONFLICT(bucket) DO UPDATE SET acl_xml = ?, updated_at = ?
            "#,
        )
        .bind(bucket)
        .bind(acl_xml)
        .bind(&now)
        .bind(acl_xml)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Stored bucket ACL for: {}", bucket);
        Ok(())
    }

    /// Get bucket ACL XML
    pub async fn get_bucket_acl(&self, bucket: &str) -> Result<Option<String>> {
        let row: Option<(String,)> = sqlx::query_as(
            r#"SELECT acl_xml FROM bucket_acls WHERE bucket = ?"#,
        )
        .bind(bucket)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| r.0))
    }

    /// Store object ACL XML
    pub async fn put_object_acl(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<&str>,
        acl_xml: &str,
    ) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let version = version_id.unwrap_or("null");
        
        sqlx::query(
            r#"
            INSERT INTO object_acls (bucket, key, version_id, acl_xml, updated_at)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(bucket, key, version_id) DO UPDATE SET acl_xml = ?, updated_at = ?
            "#,
        )
        .bind(bucket)
        .bind(key)
        .bind(version)
        .bind(acl_xml)
        .bind(&now)
        .bind(acl_xml)
        .bind(&now)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Stored object ACL for: {}/{}", bucket, key);
        Ok(())
    }

    /// Get object ACL XML
    pub async fn get_object_acl(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<&str>,
    ) -> Result<Option<String>> {
        let version = version_id.unwrap_or("null");
        
        let row: Option<(String,)> = sqlx::query_as(
            r#"SELECT acl_xml FROM object_acls WHERE bucket = ? AND key = ? AND version_id = ?"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(version)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| r.0))
    }
}

// ============= Credentials Operations for Admin API =============

use hafiz_core::types::Credentials;

impl MetadataStore {
    /// List all credentials (users)
    pub async fn list_credentials(&self) -> Result<Vec<Credentials>> {
        let rows: Vec<(String, String, Option<String>, Option<String>, bool, String)> =
            sqlx::query_as(
                r#"
                SELECT access_key, secret_key, display_name, email, is_admin, created_at
                FROM users
                ORDER BY created_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Credentials {
                access_key: r.0,
                secret_key: r.1,
                name: r.2,
                email: r.3,
                enabled: true, // Default to enabled for existing users
                created_at: DateTime::parse_from_rfc3339(&r.5)
                    .unwrap()
                    .with_timezone(&Utc),
                last_used: None,
                policies: if r.4 {
                    vec!["admin".to_string()]
                } else {
                    Vec::new()
                },
            })
            .collect())
    }

    /// Get credentials by access key
    pub async fn get_credentials(&self, access_key: &str) -> Result<Option<Credentials>> {
        let row: Option<(String, String, Option<String>, Option<String>, bool, String)> =
            sqlx::query_as(
                r#"
                SELECT access_key, secret_key, display_name, email, is_admin, created_at
                FROM users WHERE access_key = ?
                "#,
            )
            .bind(access_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| Credentials {
            access_key: r.0,
            secret_key: r.1,
            name: r.2,
            email: r.3,
            enabled: true,
            created_at: DateTime::parse_from_rfc3339(&r.5)
                .unwrap()
                .with_timezone(&Utc),
            last_used: None,
            policies: if r.4 {
                vec!["admin".to_string()]
            } else {
                Vec::new()
            },
        }))
    }

    /// Create new credentials
    pub async fn create_credentials(&self, cred: &Credentials) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let is_admin = cred.policies.contains(&"admin".to_string());

        sqlx::query(
            r#"
            INSERT INTO users (id, access_key, secret_key, display_name, email, is_admin, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(&cred.access_key)
        .bind(&cred.secret_key)
        .bind(&cred.name)
        .bind(&cred.email)
        .bind(is_admin)
        .bind(cred.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint") {
                Error::InternalError("User with this access key already exists".to_string())
            } else {
                Error::DatabaseError(e.to_string())
            }
        })?;

        debug!("Created credentials for: {}", cred.name.as_deref().unwrap_or(&cred.access_key));
        Ok(())
    }

    /// Update existing credentials
    pub async fn update_credentials(&self, cred: &Credentials) -> Result<()> {
        let is_admin = cred.policies.contains(&"admin".to_string());

        sqlx::query(
            r#"
            UPDATE users
            SET display_name = ?, email = ?, is_admin = ?
            WHERE access_key = ?
            "#,
        )
        .bind(&cred.name)
        .bind(&cred.email)
        .bind(is_admin)
        .bind(&cred.access_key)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Updated credentials for: {}", cred.access_key);
        Ok(())
    }

    /// Delete credentials
    pub async fn delete_credentials(&self, access_key: &str) -> Result<()> {
        sqlx::query(r#"DELETE FROM users WHERE access_key = ?"#)
            .bind(access_key)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted credentials for: {}", access_key);
        Ok(())
    }

    /// Get bucket versioning status
    pub async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            r#"SELECT versioning FROM buckets WHERE name = ?"#,
        )
        .bind(bucket)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.and_then(|r| r.0).filter(|s| !s.is_empty()))
    }

    /// Get bucket tags
    pub async fn get_bucket_tags(&self, bucket: &str) -> Result<HashMap<String, String>> {
        // Check if bucket_tags table exists, if not return empty
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tag_key, tag_value FROM bucket_tags
            WHERE bucket = ?
            "#,
        )
        .bind(bucket)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(rows.into_iter().collect())
    }

    /// Get lifecycle rules for a bucket
    pub async fn get_lifecycle_rules(&self, bucket: &str) -> Result<Vec<LifecycleRule>> {
        let config = self.get_bucket_lifecycle(bucket).await?;
        Ok(config.map(|c| c.rules).unwrap_or_default())
    }

    /// List delete markers in a bucket
    pub async fn list_delete_markers(&self, bucket: &str, prefix: &str, max_keys: i32) -> Result<Vec<DeleteMarker>> {
        let rows: Vec<(String, String, String)> = sqlx::query_as(
            r#"
            SELECT key, version_id, last_modified
            FROM objects
            WHERE bucket = ? AND key LIKE ? AND is_delete_marker = 1
            ORDER BY key
            LIMIT ?
            "#,
        )
        .bind(bucket)
        .bind(format!("{}%", prefix))
        .bind(max_keys)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| DeleteMarker {
                key: r.0,
                version_id: r.1,
                is_latest: true,
                last_modified: DateTime::parse_from_rfc3339(&r.2)
                    .unwrap()
                    .with_timezone(&Utc),
                owner: None,
            })
            .collect())
    }
}

// ============= MetadataRepository Trait Implementation =============

use crate::traits::{MetadataRepository, MultipartUpload as TraitMultipartUpload, MultipartUploadInfo as TraitMultipartUploadInfo, ObjectWithTags as TraitObjectWithTags, UploadPart as TraitUploadPart};
use async_trait::async_trait;

#[async_trait]
impl MetadataRepository for MetadataStore {
    async fn create_user(&self, user: &User) -> Result<()> {
        MetadataStore::create_user(self, user).await
    }

    async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<User>> {
        MetadataStore::get_user_by_access_key(self, access_key).await
    }

    async fn list_credentials(&self) -> Result<Vec<Credentials>> {
        MetadataStore::list_credentials(self).await
    }

    async fn get_credentials(&self, access_key: &str) -> Result<Option<Credentials>> {
        MetadataStore::get_credentials(self, access_key).await
    }

    async fn create_credentials(&self, cred: &Credentials) -> Result<()> {
        MetadataStore::create_credentials(self, cred).await
    }

    async fn update_credentials(&self, cred: &Credentials) -> Result<()> {
        MetadataStore::update_credentials(self, cred).await
    }

    async fn delete_credentials(&self, access_key: &str) -> Result<()> {
        MetadataStore::delete_credentials(self, access_key).await
    }

    async fn create_bucket(&self, bucket: &Bucket) -> Result<()> {
        MetadataStore::create_bucket(self, bucket).await
    }

    async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>> {
        MetadataStore::get_bucket(self, name).await
    }

    async fn list_buckets(&self) -> Result<Vec<Bucket>> {
        // Get all buckets (not just for owner)
        let rows: Vec<(String, String, String, Option<String>, Option<i32>, String)> = sqlx::query_as(
            r#"
            SELECT name, owner_id, region, versioning, object_lock_enabled, created_at
            FROM buckets ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Bucket {
                name: r.0,
                owner_id: r.1,
                region: r.2,
                versioning: VersioningStatus::from_str(r.3.as_deref().unwrap_or("")),
                object_lock_enabled: r.4.unwrap_or(0) != 0,
                created_at: DateTime::parse_from_rfc3339(&r.5)
                    .unwrap()
                    .with_timezone(&Utc),
            })
            .collect())
    }

    async fn delete_bucket(&self, name: &str) -> Result<()> {
        MetadataStore::delete_bucket(self, name).await
    }

    async fn set_bucket_versioning(&self, name: &str, status: VersioningStatus) -> Result<()> {
        MetadataStore::set_bucket_versioning(self, name, status).await
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<String>> {
        MetadataStore::get_bucket_versioning(self, bucket).await
    }

    async fn get_bucket_tags(&self, bucket: &str) -> Result<HashMap<String, String>> {
        MetadataStore::get_bucket_tags(self, bucket).await
    }

    async fn create_object(&self, object: &Object) -> Result<()> {
        // Convert Object to internal format for put_object
        let internal_obj = crate::repository::Object {
            bucket: object.bucket.clone(),
            key: object.key.clone(),
            version_id: "null".to_string(),
            size: object.size,
            etag: object.etag.clone(),
            content_type: object.content_type.clone(),
            metadata: object.metadata.clone(),
            last_modified: object.last_modified,
            is_latest: true,
            is_delete_marker: false,
            encryption: object.encryption.clone().unwrap_or_default(),
            owner: object.owner.clone(),
        };
        MetadataStore::put_object(self, &internal_obj).await
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<hafiz_core::types::Object>> {
        let result = MetadataStore::get_object(self, bucket, key).await?;
        Ok(result.map(|o| hafiz_core::types::Object {
            bucket: o.bucket,
            key: o.key,
            size: o.size,
            etag: o.etag,
            content_type: o.content_type,
            metadata: o.metadata,
            last_modified: o.last_modified,
            owner: o.owner,
            encryption: if o.encryption.is_encrypted() { Some(o.encryption) } else { None },
        }))
    }

    async fn get_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<Option<hafiz_core::types::Object>> {
        let result = MetadataStore::get_object_version(self, bucket, key, Some(version_id)).await?;
        Ok(result.map(|o| hafiz_core::types::Object {
            bucket: o.bucket,
            key: o.key,
            size: o.size,
            etag: o.etag,
            content_type: o.content_type,
            metadata: o.metadata,
            last_modified: o.last_modified,
            owner: o.owner,
            encryption: if o.encryption.is_encrypted() { Some(o.encryption) } else { None },
        }))
    }

    async fn list_objects(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<hafiz_core::types::Object>> {
        let results = MetadataStore::list_objects(self, bucket, prefix, marker, max_keys).await?;
        Ok(results
            .into_iter()
            .map(|o| hafiz_core::types::Object {
                bucket: o.bucket,
                key: o.key,
                size: o.size,
                etag: o.etag,
                content_type: o.content_type,
                metadata: o.metadata,
                last_modified: o.last_modified,
                owner: o.owner,
                encryption: if o.encryption.is_encrypted() { Some(o.encryption) } else { None },
            })
            .collect())
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
        MetadataStore::delete_object(self, bucket, key, None).await
    }

    async fn delete_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<()> {
        MetadataStore::delete_object(self, bucket, key, Some(version_id)).await
    }

    async fn create_object_version(&self, object: &hafiz_core::types::Object, version_id: &str) -> Result<()> {
        let internal_obj = crate::repository::Object {
            bucket: object.bucket.clone(),
            key: object.key.clone(),
            version_id: version_id.to_string(),
            size: object.size,
            etag: object.etag.clone(),
            content_type: object.content_type.clone(),
            metadata: object.metadata.clone(),
            last_modified: object.last_modified,
            is_latest: true,
            is_delete_marker: false,
            encryption: object.encryption.clone().unwrap_or_default(),
            owner: object.owner.clone(),
        };
        MetadataStore::put_object(self, &internal_obj).await
    }

    async fn list_object_versions(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<ObjectVersion>> {
        MetadataStore::list_object_versions(self, bucket, prefix, marker, max_keys).await
    }

    async fn create_delete_marker(&self, bucket: &str, key: &str, version_id: &str) -> Result<()> {
        MetadataStore::create_delete_marker(self, bucket, key, version_id).await
    }

    async fn list_delete_markers(&self, bucket: &str, prefix: &str, max_keys: i32) -> Result<Vec<DeleteMarker>> {
        MetadataStore::list_delete_markers(self, bucket, prefix, max_keys).await
    }

    async fn put_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>, tags: &TagSet) -> Result<()> {
        MetadataStore::put_object_tags(self, bucket, key, version_id, tags).await
    }

    async fn get_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<TagSet> {
        MetadataStore::get_object_tags(self, bucket, key, version_id).await
    }

    async fn delete_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<()> {
        MetadataStore::delete_object_tags(self, bucket, key, version_id).await
    }

    async fn put_bucket_lifecycle(&self, bucket: &str, config: &LifecycleConfiguration) -> Result<()> {
        MetadataStore::put_bucket_lifecycle(self, bucket, config).await
    }

    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<LifecycleConfiguration>> {
        MetadataStore::get_bucket_lifecycle(self, bucket).await
    }

    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<()> {
        MetadataStore::delete_bucket_lifecycle(self, bucket).await
    }

    async fn get_buckets_with_lifecycle(&self) -> Result<Vec<String>> {
        MetadataStore::get_buckets_with_lifecycle(self).await
    }

    async fn get_lifecycle_rules(&self, bucket: &str) -> Result<Vec<LifecycleRule>> {
        MetadataStore::get_lifecycle_rules(self, bucket).await
    }

    async fn get_objects_for_lifecycle(&self, bucket: &str, prefix: Option<&str>, limit: i32) -> Result<Vec<TraitObjectWithTags>> {
        let results = MetadataStore::get_objects_for_lifecycle(self, bucket, prefix, limit).await?;
        Ok(results
            .into_iter()
            .map(|o| TraitObjectWithTags {
                bucket: o.bucket,
                key: o.key,
                version_id: o.version_id,
                size: o.size,
                last_modified: o.last_modified,
                is_latest: o.is_latest,
                is_delete_marker: o.is_delete_marker,
                tags: o.tags,
            })
            .collect())
    }

    async fn create_multipart_upload(&self, upload: &TraitMultipartUpload) -> Result<()> {
        let internal = MultipartUpload {
            upload_id: upload.upload_id.clone(),
            bucket: upload.bucket.clone(),
            key: upload.key.clone(),
            content_type: upload.content_type.clone(),
            metadata: upload.metadata.clone(),
            storage_class: upload.storage_class.clone(),
            initiator_id: upload.initiator_id.clone(),
            created_at: upload.created_at,
        };
        MetadataStore::create_multipart_upload(self, &internal).await
    }

    async fn get_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Option<TraitMultipartUpload>> {
        let result = MetadataStore::get_multipart_upload(self, bucket, key, upload_id).await?;
        Ok(result.map(|u| TraitMultipartUpload {
            upload_id: u.upload_id,
            bucket: u.bucket,
            key: u.key,
            content_type: u.content_type,
            metadata: u.metadata,
            storage_class: u.storage_class,
            initiator_id: u.initiator_id,
            created_at: u.created_at,
        }))
    }

    async fn list_multipart_uploads(&self, bucket: &str, prefix: &str, marker: &str, max_uploads: i32) -> Result<Vec<TraitMultipartUploadInfo>> {
        let results = MetadataStore::list_multipart_uploads(self, bucket, prefix, marker, max_uploads).await?;
        Ok(results
            .into_iter()
            .map(|u| TraitMultipartUploadInfo {
                upload_id: u.upload_id,
                key: u.key,
                initiator_id: u.initiator_id,
                storage_class: u.storage_class,
                initiated: u.initiated,
            })
            .collect())
    }

    async fn delete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<()> {
        MetadataStore::delete_multipart_upload(self, bucket, key, upload_id).await
    }

    async fn create_upload_part(&self, bucket: &str, key: &str, upload_id: &str, part: &TraitUploadPart) -> Result<()> {
        let internal = UploadPart {
            part_number: part.part_number,
            size: part.size,
            etag: part.etag.clone(),
            last_modified: part.last_modified,
        };
        MetadataStore::create_upload_part(self, bucket, key, upload_id, &internal).await
    }

    async fn get_upload_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<TraitUploadPart>> {
        let results = MetadataStore::get_upload_parts(self, bucket, key, upload_id).await?;
        Ok(results
            .into_iter()
            .map(|p| TraitUploadPart {
                part_number: p.part_number,
                size: p.size,
                etag: p.etag,
                last_modified: p.last_modified,
            })
            .collect())
    }

    // ============= Policy Operations =============

    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<()> {
        MetadataStore::put_bucket_policy(self, bucket, policy_json).await
    }

    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>> {
        MetadataStore::get_bucket_policy(self, bucket).await
    }

    async fn delete_bucket_policy(&self, bucket: &str) -> Result<()> {
        MetadataStore::delete_bucket_policy(self, bucket).await
    }

    // ============= ACL Operations =============

    async fn put_bucket_acl(&self, bucket: &str, acl_xml: &str) -> Result<()> {
        MetadataStore::put_bucket_acl(self, bucket, acl_xml).await
    }

    async fn get_bucket_acl(&self, bucket: &str) -> Result<Option<String>> {
        MetadataStore::get_bucket_acl(self, bucket).await
    }

    async fn put_object_acl(&self, bucket: &str, key: &str, version_id: Option<&str>, acl_xml: &str) -> Result<()> {
        MetadataStore::put_object_acl(self, bucket, key, version_id, acl_xml).await
    }

    async fn get_object_acl(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<Option<String>> {
        MetadataStore::get_object_acl(self, bucket, key, version_id).await
    }
}
