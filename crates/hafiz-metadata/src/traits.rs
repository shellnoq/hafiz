//! Metadata repository trait
//!
//! Defines the interface for metadata storage operations.
//! Implementations exist for SQLite and PostgreSQL.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hafiz_core::types::{
    Bucket, ObjectInternal, User, VersioningStatus, ObjectVersion, DeleteMarker,
    TagSet, LifecycleConfiguration, LifecycleRule, Credentials, Owner,
};
use hafiz_core::Result;
use std::collections::HashMap;

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

/// Object info for listing (simplified)
#[derive(Debug, Clone)]
pub struct ObjectInfo {
    pub key: String,
    pub size: i64,
    pub etag: String,
    pub last_modified: DateTime<Utc>,
    pub storage_class: Option<String>,
    pub owner: Option<Owner>,
}

/// Bucket info for listing
#[derive(Debug, Clone)]
pub struct BucketInfo {
    pub name: String,
    pub creation_date: DateTime<Utc>,
}

/// Metadata repository trait - simplified version
#[async_trait]
pub trait MetadataRepository: Send + Sync {
    // ============= User Operations =============

    async fn create_user(&self, user: &User) -> Result<()>;
    async fn get_user_by_access_key(&self, access_key: &str) -> Result<Option<User>>;
    async fn list_credentials(&self) -> Result<Vec<Credentials>>;
    async fn get_credentials(&self, access_key: &str) -> Result<Option<Credentials>>;
    async fn create_credentials(&self, cred: &Credentials) -> Result<()>;
    async fn update_credentials(&self, cred: &Credentials) -> Result<()>;
    async fn delete_credentials(&self, access_key: &str) -> Result<()>;

    // ============= Bucket Operations =============

    async fn create_bucket(&self, bucket: &Bucket) -> Result<()>;
    async fn get_bucket(&self, name: &str) -> Result<Option<Bucket>>;
    async fn list_buckets(&self) -> Result<Vec<Bucket>>;
    async fn delete_bucket(&self, name: &str) -> Result<()>;
    async fn set_bucket_versioning(&self, name: &str, status: VersioningStatus) -> Result<()>;
    async fn get_bucket_versioning(&self, bucket: &str) -> Result<Option<String>>;
    async fn get_bucket_tags(&self, bucket: &str) -> Result<HashMap<String, String>>;

    // ============= Object Operations =============

    async fn create_object(&self, object: &ObjectInternal) -> Result<()>;
    async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<ObjectInternal>>;
    async fn get_object_version(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<Option<ObjectInternal>>;
    async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: i32,
        continuation_token: Option<&str>,
    ) -> Result<(Vec<ObjectInfo>, Vec<String>, bool, Option<String>)>;
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()>;
    async fn delete_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<bool>;

    // ============= Versioning Operations =============

    async fn list_object_versions(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        delimiter: Option<&str>,
        max_keys: i32,
        key_marker: Option<&str>,
        version_id_marker: Option<&str>,
    ) -> Result<(Vec<ObjectVersion>, Vec<DeleteMarker>, Vec<String>, bool, Option<String>, Option<String>)>;
    
    async fn create_delete_marker(&self, bucket: &str, key: &str) -> Result<String>;

    // ============= Tagging Operations =============

    async fn put_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>, tags: &TagSet) -> Result<()>;
    async fn get_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<TagSet>;
    async fn delete_object_tags(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<()>;

    // ============= Lifecycle Operations =============

    async fn put_bucket_lifecycle(&self, bucket: &str, config: &LifecycleConfiguration) -> Result<()>;
    async fn get_bucket_lifecycle(&self, bucket: &str) -> Result<Option<LifecycleConfiguration>>;
    async fn delete_bucket_lifecycle(&self, bucket: &str) -> Result<()>;
    async fn get_buckets_with_lifecycle(&self) -> Result<Vec<String>>;
    async fn get_lifecycle_rules(&self, bucket: &str) -> Result<Vec<LifecycleRule>>;
    async fn get_objects_for_lifecycle(&self, bucket: &str, prefix: Option<&str>, limit: i32) -> Result<Vec<ObjectWithTags>>;

    // ============= Multipart Operations =============

    async fn create_multipart_upload(
        &self,
        bucket: &str,
        key: &str,
        content_type: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String>;
    
    async fn get_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Option<MultipartUpload>>;
    
    async fn list_multipart_uploads(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        key_marker: Option<&str>,
        upload_id_marker: Option<&str>,
        max_uploads: i32,
    ) -> Result<(Vec<MultipartUploadInfo>, bool)>;
    
    async fn delete_multipart_upload(&self, upload_id: &str) -> Result<()>;
    async fn create_upload_part(&self, upload_id: &str, part: &UploadPart) -> Result<()>;
    async fn get_upload_parts(&self, upload_id: &str) -> Result<Vec<UploadPart>>;
}
