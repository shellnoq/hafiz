//! PostgreSQL metadata repository implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hafiz_core::types::{
    Bucket, Object, User, VersioningStatus, ObjectVersion, DeleteMarker, 
    Tag, TagSet, LifecycleConfiguration, LifecycleRule, Credentials,
    Owner,
};
use hafiz_core::{Error, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::HashMap;
use tracing::{debug, info};

use crate::traits::{
    MetadataRepository, MultipartUpload, MultipartUploadInfo, 
    ObjectWithTags, UploadPart,
};

/// PostgreSQL metadata store
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(100)
            .connect(database_url)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let store = Self { pool };
        store.init().await?;
        
        Ok(store)
    }

    async fn init(&self) -> Result<()> {
        // Users table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                access_key TEXT UNIQUE NOT NULL,
                secret_key TEXT NOT NULL,
                display_name TEXT,
                email TEXT,
                is_admin BOOLEAN DEFAULT FALSE,
                enabled BOOLEAN DEFAULT TRUE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Buckets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS buckets (
                name TEXT PRIMARY KEY,
                owner_id TEXT NOT NULL,
                region TEXT NOT NULL DEFAULT 'us-east-1',
                versioning TEXT DEFAULT '',
                object_lock_enabled BOOLEAN DEFAULT FALSE,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Objects table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS objects (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                version_id TEXT NOT NULL DEFAULT 'null',
                size BIGINT NOT NULL,
                etag TEXT NOT NULL,
                content_type TEXT NOT NULL,
                metadata JSONB,
                last_modified TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                is_latest BOOLEAN DEFAULT TRUE,
                is_delete_marker BOOLEAN DEFAULT FALSE,
                encryption JSONB,
                PRIMARY KEY (bucket, key, version_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Indexes
        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_objects_bucket ON objects(bucket)"#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_objects_latest ON objects(bucket, key, is_latest)"#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"CREATE INDEX IF NOT EXISTS idx_objects_prefix ON objects(bucket, key text_pattern_ops)"#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Object tags table
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

        // Bucket lifecycle table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bucket_lifecycle (
                bucket TEXT PRIMARY KEY,
                configuration JSONB NOT NULL,
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Multipart uploads table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multipart_uploads (
                upload_id TEXT NOT NULL,
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                content_type TEXT NOT NULL DEFAULT 'application/octet-stream',
                metadata JSONB,
                storage_class TEXT NOT NULL DEFAULT 'STANDARD',
                initiator_id TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (bucket, key, upload_id)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Upload parts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS upload_parts (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                upload_id TEXT NOT NULL,
                part_number INTEGER NOT NULL,
                size BIGINT NOT NULL,
                etag TEXT NOT NULL,
                last_modified TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                PRIMARY KEY (bucket, key, upload_id, part_number)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Bucket tags table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bucket_tags (
                bucket TEXT NOT NULL,
                tag_key TEXT NOT NULL,
                tag_value TEXT NOT NULL,
                PRIMARY KEY (bucket, tag_key)
            )
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        info!("PostgreSQL metadata store initialized");
        Ok(())
    }
}

#[async_trait]
impl MetadataRepository for PostgresStore {
    // ============= User Operations =============

    async fn create_user(&self, user: &User) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, access_key, secret_key, display_name, email, is_admin, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&user.id)
        .bind(&user.access_key)
        .bind(&user.secret_key)
        .bind(&user.display_name)
        .bind(&user.email)
        .bind(user.is_admin)
        .bind(user.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Created user: {}", user.access_key);
        Ok(())
    }

    async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<User>> {
        let row: Option<(String, String, String, Option<String>, Option<String>, bool, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT id, access_key, secret_key, display_name, email, is_admin, created_at
                FROM users WHERE access_key = $1
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
            created_at: r.6,
        }))
    }

    async fn list_credentials(&self) -> Result<Vec<Credentials>> {
        let rows: Vec<(String, String, Option<String>, Option<String>, bool, bool, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT access_key, secret_key, display_name, email, is_admin, COALESCE(enabled, true), created_at
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
                enabled: r.5,
                created_at: r.6,
                last_used: None,
                policies: if r.4 { vec!["admin".to_string()] } else { Vec::new() },
            })
            .collect())
    }

    async fn get_credentials(&self, access_key: &str) -> Result<Option<Credentials>> {
        let row: Option<(String, String, Option<String>, Option<String>, bool, bool, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT access_key, secret_key, display_name, email, is_admin, COALESCE(enabled, true), created_at
                FROM users WHERE access_key = $1
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
            enabled: r.5,
            created_at: r.6,
            last_used: None,
            policies: if r.4 { vec!["admin".to_string()] } else { Vec::new() },
        }))
    }

    async fn create_credentials(&self, cred: &Credentials) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let is_admin = cred.policies.contains(&"admin".to_string());

        sqlx::query(
            r#"
            INSERT INTO users (id, access_key, secret_key, display_name, email, is_admin, enabled, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&id)
        .bind(&cred.access_key)
        .bind(&cred.secret_key)
        .bind(&cred.name)
        .bind(&cred.email)
        .bind(is_admin)
        .bind(cred.enabled)
        .bind(cred.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn update_credentials(&self, cred: &Credentials) -> Result<()> {
        let is_admin = cred.policies.contains(&"admin".to_string());

        sqlx::query(
            r#"
            UPDATE users
            SET display_name = $1, email = $2, is_admin = $3, enabled = $4
            WHERE access_key = $5
            "#,
        )
        .bind(&cred.name)
        .bind(&cred.email)
        .bind(is_admin)
        .bind(cred.enabled)
        .bind(&cred.access_key)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn delete_credentials(&self, access_key: &str) -> Result<()> {
        sqlx::query(r#"DELETE FROM users WHERE access_key = $1"#)
            .bind(access_key)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ============= Bucket Operations =============

    async fn create_bucket(&self, bucket: &Bucket) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO buckets (name, owner_id, region, versioning, object_lock_enabled, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(&bucket.name)
        .bind(&bucket.owner_id)
        .bind(&bucket.region)
        .bind(bucket.versioning.as_str())
        .bind(bucket.object_lock_enabled)
        .bind(bucket.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("duplicate key") || e.to_string().contains("unique") {
                Error::BucketAlreadyExists
            } else {
                Error::DatabaseError(e.to_string())
            }
        })?;

        debug!("Created bucket: {}", bucket.name);
        Ok(())
    }

    async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>> {
        let row: Option<(String, String, String, Option<String>, Option<bool>, DateTime<Utc>)> = 
            sqlx::query_as(
                r#"
                SELECT name, owner_id, region, versioning, object_lock_enabled, created_at
                FROM buckets WHERE name = $1
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
            object_lock_enabled: r.4.unwrap_or(false),
            created_at: r.5,
        }))
    }

    async fn list_buckets(&self) -> Result<Vec<Bucket>> {
        let rows: Vec<(String, String, String, Option<String>, Option<bool>, DateTime<Utc>)> = 
            sqlx::query_as(
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
                object_lock_enabled: r.4.unwrap_or(false),
                created_at: r.5,
            })
            .collect())
    }

    async fn delete_bucket(&self, name: &str) -> Result<()> {
        // Check if bucket has objects
        let count: (i64,) = sqlx::query_as(
            r#"SELECT COUNT(*) FROM objects WHERE bucket = $1"#,
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        if count.0 > 0 {
            return Err(Error::BucketNotEmpty);
        }

        sqlx::query(r#"DELETE FROM buckets WHERE name = $1"#)
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted bucket: {}", name);
        Ok(())
    }

    async fn set_bucket_versioning(&self, name: &str, status: VersioningStatus) -> Result<()> {
        sqlx::query(r#"UPDATE buckets SET versioning = $1 WHERE name = $2"#)
            .bind(status.as_str())
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<String>> {
        let row: Option<(Option<String>,)> = sqlx::query_as(
            r#"SELECT versioning FROM buckets WHERE name = $1"#,
        )
        .bind(bucket)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.and_then(|r| r.0).filter(|s| !s.is_empty()))
    }

    async fn get_bucket_tags(&self, bucket: &str) -> Result<HashMap<String, String>> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"SELECT tag_key, tag_value FROM bucket_tags WHERE bucket = $1"#,
        )
        .bind(bucket)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_default();

        Ok(rows.into_iter().collect())
    }

    // ============= Object Operations =============

    async fn create_object(&self, object: &Object) -> Result<()> {
        let metadata_json = serde_json::to_value(&object.metadata).ok();
        let encryption_json = object.encryption.as_ref()
            .and_then(|e| serde_json::to_value(e).ok());

        sqlx::query(
            r#"
            INSERT INTO objects (bucket, key, version_id, size, etag, content_type, metadata, last_modified, is_latest, is_delete_marker, encryption)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (bucket, key, version_id) DO UPDATE SET
                size = EXCLUDED.size,
                etag = EXCLUDED.etag,
                content_type = EXCLUDED.content_type,
                metadata = EXCLUDED.metadata,
                last_modified = EXCLUDED.last_modified,
                is_latest = EXCLUDED.is_latest,
                encryption = EXCLUDED.encryption
            "#,
        )
        .bind(&object.bucket)
        .bind(&object.key)
        .bind("null")
        .bind(object.size)
        .bind(&object.etag)
        .bind(object.content_type.as_deref().unwrap_or("application/octet-stream"))
        .bind(&metadata_json)
        .bind(object.last_modified)
        .bind(true)
        .bind(false)
        .bind(&encryption_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Created object: {}/{}", object.bucket, object.key);
        Ok(())
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<Object>> {
        let row: Option<(String, String, String, i64, String, String, Option<serde_json::Value>, DateTime<Utc>, Option<serde_json::Value>)> =
            sqlx::query_as(
                r#"
                SELECT bucket, key, version_id, size, etag, content_type, metadata, last_modified, encryption
                FROM objects
                WHERE bucket = $1 AND key = $2 AND is_latest = true AND is_delete_marker = false
                "#,
            )
            .bind(bucket)
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            let metadata: HashMap<String, String> = r.6
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let encryption = r.8.and_then(|v| serde_json::from_value(v).ok());

            Object {
                bucket: r.0,
                key: r.1,
                size: r.3,
                etag: r.4,
                content_type: Some(r.5),
                metadata,
                last_modified: r.7,
                owner: None,
                encryption,
            }
        }))
    }

    async fn get_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<Option<Object>> {
        let row: Option<(String, String, String, i64, String, String, Option<serde_json::Value>, DateTime<Utc>, Option<serde_json::Value>)> =
            sqlx::query_as(
                r#"
                SELECT bucket, key, version_id, size, etag, content_type, metadata, last_modified, encryption
                FROM objects
                WHERE bucket = $1 AND key = $2 AND version_id = $3 AND is_delete_marker = false
                "#,
            )
            .bind(bucket)
            .bind(key)
            .bind(version_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            let metadata: HashMap<String, String> = r.6
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();
            let encryption = r.8.and_then(|v| serde_json::from_value(v).ok());

            Object {
                bucket: r.0,
                key: r.1,
                size: r.3,
                etag: r.4,
                content_type: Some(r.5),
                metadata,
                last_modified: r.7,
                owner: None,
                encryption,
            }
        }))
    }

    async fn list_objects(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<Object>> {
        let rows: Vec<(String, String, i64, String, String, Option<serde_json::Value>, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT bucket, key, size, etag, content_type, metadata, last_modified
                FROM objects
                WHERE bucket = $1 AND key LIKE $2 AND key > $3 AND is_latest = true AND is_delete_marker = false
                ORDER BY key
                LIMIT $4
                "#,
            )
            .bind(bucket)
            .bind(format!("{}%", prefix))
            .bind(marker)
            .bind(max_keys)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let metadata: HashMap<String, String> = r.5
                    .and_then(|v| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                Object {
                    bucket: r.0,
                    key: r.1,
                    size: r.2,
                    etag: r.3,
                    content_type: Some(r.4),
                    metadata,
                    last_modified: r.6,
                    owner: None,
                    encryption: None,
                }
            })
            .collect())
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
        sqlx::query(
            r#"DELETE FROM objects WHERE bucket = $1 AND key = $2 AND version_id = 'null'"#,
        )
        .bind(bucket)
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Also delete tags
        sqlx::query(
            r#"DELETE FROM object_tags WHERE bucket = $1 AND key = $2 AND version_id = 'null'"#,
        )
        .bind(bucket)
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        debug!("Deleted object: {}/{}", bucket, key);
        Ok(())
    }

    async fn delete_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<()> {
        sqlx::query(
            r#"DELETE FROM objects WHERE bucket = $1 AND key = $2 AND version_id = $3"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(version_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ============= Versioning Operations =============

    async fn create_object_version(&self, object: &Object, version_id: &str) -> Result<()> {
        // Mark previous versions as not latest
        sqlx::query(
            r#"UPDATE objects SET is_latest = false WHERE bucket = $1 AND key = $2"#,
        )
        .bind(&object.bucket)
        .bind(&object.key)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        let metadata_json = serde_json::to_value(&object.metadata).ok();
        let encryption_json = object.encryption.as_ref()
            .and_then(|e| serde_json::to_value(e).ok());

        sqlx::query(
            r#"
            INSERT INTO objects (bucket, key, version_id, size, etag, content_type, metadata, last_modified, is_latest, is_delete_marker, encryption)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, true, false, $9)
            "#,
        )
        .bind(&object.bucket)
        .bind(&object.key)
        .bind(version_id)
        .bind(object.size)
        .bind(&object.etag)
        .bind(object.content_type.as_deref().unwrap_or("application/octet-stream"))
        .bind(&metadata_json)
        .bind(object.last_modified)
        .bind(&encryption_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn list_object_versions(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<ObjectVersion>> {
        let rows: Vec<(String, String, i64, String, DateTime<Utc>, bool)> =
            sqlx::query_as(
                r#"
                SELECT key, version_id, size, etag, last_modified, is_latest
                FROM objects
                WHERE bucket = $1 AND key LIKE $2 AND key > $3 AND is_delete_marker = false
                ORDER BY key, last_modified DESC
                LIMIT $4
                "#,
            )
            .bind(bucket)
            .bind(format!("{}%", prefix))
            .bind(marker)
            .bind(max_keys)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| ObjectVersion {
                key: r.0,
                version_id: r.1,
                size: r.2,
                etag: r.3,
                last_modified: r.4,
                is_latest: r.5,
                owner: None,
            })
            .collect())
    }

    async fn create_delete_marker(&self, bucket: &str, key: &str, version_id: &str) -> Result<()> {
        // Mark previous as not latest
        sqlx::query(
            r#"UPDATE objects SET is_latest = false WHERE bucket = $1 AND key = $2"#,
        )
        .bind(bucket)
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO objects (bucket, key, version_id, size, etag, content_type, last_modified, is_latest, is_delete_marker)
            VALUES ($1, $2, $3, 0, '', '', NOW(), true, true)
            "#,
        )
        .bind(bucket)
        .bind(key)
        .bind(version_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn list_delete_markers(&self, bucket: &str, prefix: &str, max_keys: i32) -> Result<Vec<DeleteMarker>> {
        let rows: Vec<(String, String, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT key, version_id, last_modified
                FROM objects
                WHERE bucket = $1 AND key LIKE $2 AND is_delete_marker = true
                ORDER BY key
                LIMIT $3
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
                last_modified: r.2,
                owner: None,
            })
            .collect())
    }

    // ============= Tagging Operations =============

    async fn put_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>, tags: &TagSet) -> Result<()> {
        let vid = version_id.unwrap_or("null");

        // Delete existing
        sqlx::query(
            r#"DELETE FROM object_tags WHERE bucket = $1 AND key = $2 AND version_id = $3"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(vid)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        // Insert new
        for tag in &tags.tags {
            sqlx::query(
                r#"
                INSERT INTO object_tags (bucket, key, version_id, tag_key, tag_value)
                VALUES ($1, $2, $3, $4, $5)
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

        Ok(())
    }

    async fn get_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<TagSet> {
        let vid = version_id.unwrap_or("null");

        let rows: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT tag_key, tag_value FROM object_tags
            WHERE bucket = $1 AND key = $2 AND version_id = $3
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

    async fn delete_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<()> {
        let vid = version_id.unwrap_or("null");

        sqlx::query(
            r#"DELETE FROM object_tags WHERE bucket = $1 AND key = $2 AND version_id = $3"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(vid)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // ============= Lifecycle Operations =============

    async fn put_bucket_lifecycle(&self, bucket: &str, config: &LifecycleConfiguration) -> Result<()> {
        let config_json = serde_json::to_value(config)
            .map_err(|e| Error::InternalError(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO bucket_lifecycle (bucket, configuration, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (bucket) DO UPDATE SET
                configuration = EXCLUDED.configuration,
                updated_at = NOW()
            "#,
        )
        .bind(bucket)
        .bind(&config_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<LifecycleConfiguration>> {
        let row: Option<(serde_json::Value,)> = sqlx::query_as(
            r#"SELECT configuration FROM bucket_lifecycle WHERE bucket = $1"#,
        )
        .bind(bucket)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        match row {
            Some((config_json,)) => {
                let config: LifecycleConfiguration = serde_json::from_value(config_json)
                    .map_err(|e| Error::InternalError(e.to_string()))?;
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<()> {
        sqlx::query(r#"DELETE FROM bucket_lifecycle WHERE bucket = $1"#)
            .bind(bucket)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_buckets_with_lifecycle(&self) -> Result<Vec<String>> {
        let rows: Vec<(String,)> = sqlx::query_as(
            r#"SELECT bucket FROM bucket_lifecycle"#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    async fn get_lifecycle_rules(&self, bucket: &str) -> Result<Vec<LifecycleRule>> {
        let config = self.get_bucket_lifecycle(bucket).await?;
        Ok(config.map(|c| c.rules).unwrap_or_default())
    }

    async fn get_objects_for_lifecycle(&self, bucket: &str, prefix: Option<&str>, limit: i32) -> Result<Vec<ObjectWithTags>> {
        let prefix = prefix.unwrap_or("");

        let rows: Vec<(String, String, i64, DateTime<Utc>, bool, bool)> = sqlx::query_as(
            r#"
            SELECT key, version_id, size, last_modified, is_latest, is_delete_marker
            FROM objects
            WHERE bucket = $1 AND key LIKE $2 AND is_delete_marker = false
            ORDER BY key
            LIMIT $3
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
                last_modified: row.3,
                is_latest: row.4,
                is_delete_marker: row.5,
                tags,
            });
        }

        Ok(objects)
    }

    // ============= Multipart Operations =============

    async fn create_multipart_upload(&self, upload: &MultipartUpload) -> Result<()> {
        let metadata_json = serde_json::to_value(&upload.metadata).ok();

        sqlx::query(
            r#"
            INSERT INTO multipart_uploads (upload_id, bucket, key, content_type, metadata, storage_class, initiator_id, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&upload.upload_id)
        .bind(&upload.bucket)
        .bind(&upload.key)
        .bind(&upload.content_type)
        .bind(&metadata_json)
        .bind(&upload.storage_class)
        .bind(&upload.initiator_id)
        .bind(upload.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Option<MultipartUpload>> {
        let row: Option<(String, String, String, String, Option<serde_json::Value>, String, String, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT upload_id, bucket, key, content_type, metadata, storage_class, initiator_id, created_at
                FROM multipart_uploads
                WHERE bucket = $1 AND key = $2 AND upload_id = $3
                "#,
            )
            .bind(bucket)
            .bind(key)
            .bind(upload_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(row.map(|r| {
            let metadata: HashMap<String, String> = r.4
                .and_then(|v| serde_json::from_value(v).ok())
                .unwrap_or_default();

            MultipartUpload {
                upload_id: r.0,
                bucket: r.1,
                key: r.2,
                content_type: r.3,
                metadata,
                storage_class: r.5,
                initiator_id: r.6,
                created_at: r.7,
            }
        }))
    }

    async fn list_multipart_uploads(&self, bucket: &str, prefix: &str, marker: &str, max_uploads: i32) -> Result<Vec<MultipartUploadInfo>> {
        let rows: Vec<(String, String, String, String, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT upload_id, key, initiator_id, storage_class, created_at
                FROM multipart_uploads
                WHERE bucket = $1 AND key LIKE $2 AND key > $3
                ORDER BY key
                LIMIT $4
                "#,
            )
            .bind(bucket)
            .bind(format!("{}%", prefix))
            .bind(marker)
            .bind(max_uploads)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| MultipartUploadInfo {
                upload_id: r.0,
                key: r.1,
                initiator_id: r.2,
                storage_class: r.3,
                initiated: r.4,
            })
            .collect())
    }

    async fn delete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<()> {
        sqlx::query(
            r#"DELETE FROM multipart_uploads WHERE bucket = $1 AND key = $2 AND upload_id = $3"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(upload_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        sqlx::query(
            r#"DELETE FROM upload_parts WHERE bucket = $1 AND key = $2 AND upload_id = $3"#,
        )
        .bind(bucket)
        .bind(key)
        .bind(upload_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn create_upload_part(&self, bucket: &str, key: &str, upload_id: &str, part: &UploadPart) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO upload_parts (bucket, key, upload_id, part_number, size, etag, last_modified)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (bucket, key, upload_id, part_number) DO UPDATE SET
                size = EXCLUDED.size,
                etag = EXCLUDED.etag,
                last_modified = EXCLUDED.last_modified
            "#,
        )
        .bind(bucket)
        .bind(key)
        .bind(upload_id)
        .bind(part.part_number)
        .bind(part.size)
        .bind(&part.etag)
        .bind(part.last_modified)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_upload_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<UploadPart>> {
        let rows: Vec<(i32, i64, String, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
                SELECT part_number, size, etag, last_modified
                FROM upload_parts
                WHERE bucket = $1 AND key = $2 AND upload_id = $3
                ORDER BY part_number
                "#,
            )
            .bind(bucket)
            .bind(key)
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
                last_modified: r.3,
            })
            .collect())
    }
}
