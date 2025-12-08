//! Metadata repository trait
//!
//! Defines the interface for metadata storage operations.
//! Implementations exist for SQLite and PostgreSQL.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use hafiz_core::types::{
    Bucket, Object, User, VersioningStatus, ObjectVersion, DeleteMarker, 
    TagSet, LifecycleConfiguration, LifecycleRule, Credentials,
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

/// Metadata repository trait
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
    
    async fn create_object(&self, object: &Object) -> Result<()>;
    async fn get_object(&self, bucket: &str, key: &str) -> Result<Option<Object>>;
    async fn get_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<Option<Object>>;
    async fn list_objects(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<Object>>;
    async fn delete_object(&self, bucket: &str, key: &str) -> Result<()>;
    async fn delete_object_version(&self, bucket: &str, key: &str, version_id: &str) -> Result<()>;

    // ============= Versioning Operations =============
    
    async fn create_object_version(&self, object: &Object, version_id: &str) -> Result<()>;
    async fn list_object_versions(&self, bucket: &str, prefix: &str, marker: &str, max_keys: i32) -> Result<Vec<ObjectVersion>>;
    async fn create_delete_marker(&self, bucket: &str, key: &str, version_id: &str) -> Result<()>;
    async fn list_delete_markers(&self, bucket: &str, prefix: &str, max_keys: i32) -> Result<Vec<DeleteMarker>>;

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
    
    async fn create_multipart_upload(&self, upload: &MultipartUpload) -> Result<()>;
    async fn get_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Option<MultipartUpload>>;
    async fn list_multipart_uploads(&self, bucket: &str, prefix: &str, marker: &str, max_uploads: i32) -> Result<Vec<MultipartUploadInfo>>;
    async fn delete_multipart_upload(&self, bucket: &str, key: &str, upload_id: &str) -> Result<()>;
    async fn create_upload_part(&self, bucket: &str, key: &str, upload_id: &str, part: &UploadPart) -> Result<()>;
    async fn get_upload_parts(&self, bucket: &str, key: &str, upload_id: &str) -> Result<Vec<UploadPart>>;

    // ============= Policy Operations =============
    
    /// Store bucket policy JSON
    async fn put_bucket_policy(&self, bucket: &str, policy_json: &str) -> Result<()>;
    
    /// Get bucket policy JSON
    async fn get_bucket_policy(&self, bucket: &str) -> Result<Option<String>>;
    
    /// Delete bucket policy
    async fn delete_bucket_policy(&self, bucket: &str) -> Result<()>;

    // ============= ACL Operations =============
    
    /// Store bucket ACL XML
    async fn put_bucket_acl(&self, bucket: &str, acl_xml: &str) -> Result<()>;
    
    /// Get bucket ACL XML
    async fn get_bucket_acl(&self, bucket: &str) -> Result<Option<String>>;
    
    /// Store object ACL XML
    async fn put_object_acl(&self, bucket: &str, key: &str, version_id: Option<&str>, acl_xml: &str) -> Result<()>;
    
    /// Get object ACL XML
    async fn get_object_acl(&self, bucket: &str, key: &str, version_id: Option<&str>) -> Result<Option<String>>;

    // ============= Notification Operations =============
    
    /// Store bucket notification configuration JSON
    async fn put_bucket_notification(&self, bucket: &str, config_json: &str) -> Result<()>;
    
    /// Get bucket notification configuration JSON
    async fn get_bucket_notification(&self, bucket: &str) -> Result<Option<String>>;
}
