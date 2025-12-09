//! Metadata storage for Hafiz
//!
//! Supports both SQLite and PostgreSQL backends.
//! Database selection is automatic based on the connection URL:
//! - URLs starting with `postgres://` or `postgresql://` use PostgreSQL
//! - All other URLs use SQLite

pub mod repository;
pub mod traits;
pub mod postgres;

pub use repository::MetadataStore as SqliteStore;
pub use postgres::PostgresStore;
pub use traits::*;

use hafiz_core::Result;
use std::sync::Arc;

/// Unified metadata store that can use either SQLite or PostgreSQL
pub enum MetadataStore {
    Sqlite(SqliteStore),
    Postgres(PostgresStore),
}

impl MetadataStore {
    /// Create a new metadata store based on the database URL
    ///
    /// - URLs starting with `postgres://` or `postgresql://` → PostgreSQL
    /// - All other URLs → SQLite
    pub async fn new(database_url: &str) -> Result<Self> {
        if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
            let store = PostgresStore::new(database_url).await?;
            Ok(MetadataStore::Postgres(store))
        } else {
            let store = SqliteStore::new(database_url).await?;
            Ok(MetadataStore::Sqlite(store))
        }
    }

    /// Get a reference to the underlying repository trait
    pub fn as_repo(&self) -> &dyn MetadataRepository {
        match self {
            MetadataStore::Sqlite(s) => s as &dyn MetadataRepository,
            MetadataStore::Postgres(s) => s as &dyn MetadataRepository,
        }
    }
}

// Implement MetadataRepository for MetadataStore by delegating
#[async_trait::async_trait]
impl MetadataRepository for MetadataStore {
    async fn create_user(&self, user: &hafiz_core::types::User) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_user(user).await,
            MetadataStore::Postgres(s) => s.create_user(user).await,
        }
    }

    async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<hafiz_core::types::User>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_user_by_access_key(access_key).await,
            MetadataStore::Postgres(s) => s.get_user_by_access_key(access_key).await,
        }
    }

    async fn list_credentials(&self) -> Result<Vec<hafiz_core::types::Credentials>> {
        match self {
            MetadataStore::Sqlite(s) => s.list_credentials().await,
            MetadataStore::Postgres(s) => s.list_credentials().await,
        }
    }

    async fn get_credentials(&self, access_key: &str) -> Result<Option<hafiz_core::types::Credentials>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_credentials(access_key).await,
            MetadataStore::Postgres(s) => s.get_credentials(access_key).await,
        }
    }

    async fn create_credentials(&self, cred: &hafiz_core::types::Credentials) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_credentials(cred).await,
            MetadataStore::Postgres(s) => s.create_credentials(cred).await,
        }
    }

    async fn update_credentials(&self, cred: &hafiz_core::types::Credentials) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.update_credentials(cred).await,
            MetadataStore::Postgres(s) => s.update_credentials(cred).await,
        }
    }

    async fn delete_credentials(&self, access_key: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.delete_credentials(access_key).await,
            MetadataStore::Postgres(s) => s.delete_credentials(access_key).await,
        }
    }

    async fn create_bucket(&self, bucket: &hafiz_core::types::Bucket) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_bucket(bucket).await,
            MetadataStore::Postgres(s) => s.create_bucket(bucket).await,
        }
    }

    async fn get_bucket(&self, name: &str) -> Result<Option<hafiz_core::types::Bucket>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_bucket(name).await,
            MetadataStore::Postgres(s) => s.get_bucket(name).await,
        }
    }

    async fn list_buckets(&self) -> Result<Vec<hafiz_core::types::Bucket>> {
        match self {
            MetadataStore::Sqlite(s) => s.list_buckets().await,
            MetadataStore::Postgres(s) => s.list_buckets().await,
        }
    }

    async fn delete_bucket(&self, name: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.delete_bucket(name).await,
            MetadataStore::Postgres(s) => s.delete_bucket(name).await,
        }
    }

    async fn set_bucket_versioning(&self, name: &str, status: hafiz_core::types::VersioningStatus) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.set_bucket_versioning(name, status).await,
            MetadataStore::Postgres(s) => s.set_bucket_versioning(name, status).await,
        }
    }

    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<String>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_bucket_versioning(bucket).await,
            MetadataStore::Postgres(s) => s.get_bucket_versioning(bucket).await,
        }
    }

    async fn get_bucket_tags(&self, bucket: &str) -> Result<std::collections::HashMap<String, String>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_bucket_tags(bucket).await,
            MetadataStore::Postgres(s) => s.get_bucket_tags(bucket).await,
        }
    }

    async fn create_object(&self, object: &hafiz_core::types::Object) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_object(object).await,
            MetadataStore::Postgres(s) => s.create_object(object).await,
        }
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<hafiz_core::types::Object>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_object(bucket, key).await,
            MetadataStore::Postgres(s) => s.get_object(bucket, key).await,
        }
    }

    async fn get_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<Option<hafiz_core::types::Object>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_object_version(bucket, key, version_id).await,
            MetadataStore::Postgres(s) => s.get_object_version(bucket, key, version_id).await,
        }
    }

    async fn list_objects(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<hafiz_core::types::Object>> {
        match self {
            MetadataStore::Sqlite(s) => s.list_objects(bucket, prefix, marker, max_keys).await,
            MetadataStore::Postgres(s) => s.list_objects(bucket, prefix, marker, max_keys).await,
        }
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.delete_object(bucket, key).await,
            MetadataStore::Postgres(s) => s.delete_object(bucket, key).await,
        }
    }

    async fn delete_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.delete_object_version(bucket, key, version_id).await,
            MetadataStore::Postgres(s) => s.delete_object_version(bucket, key, version_id).await,
        }
    }

    async fn create_object_version(&self, object: &hafiz_core::types::Object, version_id: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_object_version(object, version_id).await,
            MetadataStore::Postgres(s) => s.create_object_version(object, version_id).await,
        }
    }

    async fn list_object_versions(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<hafiz_core::types::ObjectVersion>> {
        match self {
            MetadataStore::Sqlite(s) => s.list_object_versions(bucket, prefix, marker, max_keys).await,
            MetadataStore::Postgres(s) => s.list_object_versions(bucket, prefix, marker, max_keys).await,
        }
    }

    async fn create_delete_marker(&self, bucket: &str, key: &str, version_id: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_delete_marker(bucket, key, version_id).await,
            MetadataStore::Postgres(s) => s.create_delete_marker(bucket, key, version_id).await,
        }
    }

    async fn list_delete_markers(&self, bucket: &str, prefix: &str, max_keys: i32) -> Result<Vec<hafiz_core::types::DeleteMarker>> {
        match self {
            MetadataStore::Sqlite(s) => s.list_delete_markers(bucket, prefix, max_keys).await,
            MetadataStore::Postgres(s) => s.list_delete_markers(bucket, prefix, max_keys).await,
        }
    }

    async fn put_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>, tags: &hafiz_core::types::TagSet) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.put_object_tags(bucket, key, version_id, tags).await,
            MetadataStore::Postgres(s) => s.put_object_tags(bucket, key, version_id, tags).await,
        }
    }

    async fn get_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<hafiz_core::types::TagSet> {
        match self {
            MetadataStore::Sqlite(s) => s.get_object_tags(bucket, key, version_id).await,
            MetadataStore::Postgres(s) => s.get_object_tags(bucket, key, version_id).await,
        }
    }

    async fn delete_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.delete_object_tags(bucket, key, version_id).await,
            MetadataStore::Postgres(s) => s.delete_object_tags(bucket, key, version_id).await,
        }
    }

    async fn put_bucket_lifecycle(&self, bucket: &str, config: &hafiz_core::types::LifecycleConfiguration) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.put_bucket_lifecycle(bucket, config).await,
            MetadataStore::Postgres(s) => s.put_bucket_lifecycle(bucket, config).await,
        }
    }

    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<hafiz_core::types::LifecycleConfiguration>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_bucket_lifecycle(bucket).await,
            MetadataStore::Postgres(s) => s.get_bucket_lifecycle(bucket).await,
        }
    }

    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.delete_bucket_lifecycle(bucket).await,
            MetadataStore::Postgres(s) => s.delete_bucket_lifecycle(bucket).await,
        }
    }

    async fn get_buckets_with_lifecycle(&self) -> Result<Vec<String>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_buckets_with_lifecycle().await,
            MetadataStore::Postgres(s) => s.get_buckets_with_lifecycle().await,
        }
    }

    async fn get_lifecycle_rules(&self, bucket: &str) -> Result<Vec<hafiz_core::types::LifecycleRule>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_lifecycle_rules(bucket).await,
            MetadataStore::Postgres(s) => s.get_lifecycle_rules(bucket).await,
        }
    }

    async fn get_objects_for_lifecycle(&self, bucket: &str, prefix: Option<&str>, limit: i32) -> Result<Vec<ObjectWithTags>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_objects_for_lifecycle(bucket, prefix, limit).await,
            MetadataStore::Postgres(s) => s.get_objects_for_lifecycle(bucket, prefix, limit).await,
        }
    }

    async fn create_multipart_upload(&self, upload: &MultipartUpload) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_multipart_upload(upload).await,
            MetadataStore::Postgres(s) => s.create_multipart_upload(upload).await,
        }
    }

    async fn get_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Option<MultipartUpload>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_multipart_upload(bucket, key, upload_id).await,
            MetadataStore::Postgres(s) => s.get_multipart_upload(bucket, key, upload_id).await,
        }
    }

    async fn list_multipart_uploads(&self, bucket: &str, prefix: &str, marker: &str, max_uploads: i32) -> Result<Vec<MultipartUploadInfo>> {
        match self {
            MetadataStore::Sqlite(s) => s.list_multipart_uploads(bucket, prefix, marker, max_uploads).await,
            MetadataStore::Postgres(s) => s.list_multipart_uploads(bucket, prefix, marker, max_uploads).await,
        }
    }

    async fn delete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.delete_multipart_upload(bucket, key, upload_id).await,
            MetadataStore::Postgres(s) => s.delete_multipart_upload(bucket, key, upload_id).await,
        }
    }

    async fn create_upload_part(&self, bucket: &str, key: &str, upload_id: &str, part: &UploadPart) -> Result<()> {
        match self {
            MetadataStore::Sqlite(s) => s.create_upload_part(bucket, key, upload_id, part).await,
            MetadataStore::Postgres(s) => s.create_upload_part(bucket, key, upload_id, part).await,
        }
    }

    async fn get_upload_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<UploadPart>> {
        match self {
            MetadataStore::Sqlite(s) => s.get_upload_parts(bucket, key, upload_id).await,
            MetadataStore::Postgres(s) => s.get_upload_parts(bucket, key, upload_id).await,
        }
    }
}
