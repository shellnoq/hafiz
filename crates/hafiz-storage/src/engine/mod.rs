//! Storage engine implementations

use async_trait::async_trait;
use bytes::Bytes;
use hafiz_core::{Error, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn};

/// Storage engine trait
#[async_trait]
pub trait StorageEngine: Send + Sync {
    /// Store object data
    async fn put(&self, bucket: &str, key: &str, data: Bytes) -> Result<String>;
    
    /// Retrieve object data
    async fn get(&self, bucket: &str, key: &str) -> Result<Bytes>;
    
    /// Retrieve partial object data
    async fn get_range(&self, bucket: &str, key: &str, start: i64, end: i64) -> Result<Bytes>;
    
    /// Delete object
    async fn delete(&self, bucket: &str, key: &str) -> Result<()>;
    
    /// Check if object exists
    async fn exists(&self, bucket: &str, key: &str) -> Result<bool>;
    
    /// Get object size
    async fn size(&self, bucket: &str, key: &str) -> Result<i64>;
    
    /// Create bucket directory
    async fn create_bucket(&self, bucket: &str) -> Result<()>;
    
    /// Delete bucket directory
    async fn delete_bucket(&self, bucket: &str) -> Result<()>;
    
    /// Check if bucket exists
    async fn bucket_exists(&self, bucket: &str) -> Result<bool>;
}

/// Local filesystem storage engine
pub struct LocalStorage {
    data_dir: PathBuf,
}

impl LocalStorage {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            data_dir: data_dir.as_ref().to_path_buf(),
        }
    }

    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.data_dir).await?;
        info!("Storage initialized at {:?}", self.data_dir);
        Ok(())
    }

    fn object_path(&self, bucket: &str, key: &str) -> PathBuf {
        // Hash-based directory structure to avoid too many files in one dir
        let hash = hafiz_crypto::md5_hash(key.as_bytes());
        let prefix = &hash[..2];
        self.data_dir
            .join(bucket)
            .join("objects")
            .join(prefix)
            .join(&hash)
    }

    fn bucket_path(&self, bucket: &str) -> PathBuf {
        self.data_dir.join(bucket)
    }

    /// Health check - verify storage is accessible
    pub async fn health_check(&self) -> Result<()> {
        // Check if data directory exists and is writable
        if !self.data_dir.exists() {
            return Err(Error::InternalError("Data directory does not exist".to_string()));
        }

        // Try to create a temp file to verify write access
        let test_file = self.data_dir.join(".health_check");
        match fs::write(&test_file, "ok").await {
            Ok(_) => {
                let _ = fs::remove_file(&test_file).await;
                Ok(())
            }
            Err(e) => Err(Error::InternalError(format!("Storage not writable: {}", e))),
        }
    }
}

#[async_trait]
impl StorageEngine for LocalStorage {
    async fn put(&self, bucket: &str, key: &str, data: Bytes) -> Result<String> {
        let path = self.object_path(bucket, key);
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let mut file = fs::File::create(&path).await?;
        file.write_all(&data).await?;
        file.sync_all().await?;
        
        let etag = hafiz_crypto::md5_hash(&data);
        debug!("Stored object {}/{} ({} bytes)", bucket, key, data.len());
        
        Ok(etag)
    }

    async fn get(&self, bucket: &str, key: &str) -> Result<Bytes> {
        let path = self.object_path(bucket, key);
        
        if !path.exists() {
            return Err(Error::NoSuchKey);
        }
        
        let data = fs::read(&path).await?;
        debug!("Retrieved object {}/{} ({} bytes)", bucket, key, data.len());
        
        Ok(Bytes::from(data))
    }

    async fn get_range(&self, bucket: &str, key: &str, start: i64, end: i64) -> Result<Bytes> {
        let path = self.object_path(bucket, key);
        
        if !path.exists() {
            return Err(Error::NoSuchKey);
        }
        
        let mut file = fs::File::open(&path).await?;
        let len = (end - start + 1) as usize;
        
        file.seek(std::io::SeekFrom::Start(start as u64)).await?;
        
        let mut buffer = vec![0u8; len];
        file.read_exact(&mut buffer).await?;
        
        Ok(Bytes::from(buffer))
    }

    async fn delete(&self, bucket: &str, key: &str) -> Result<()> {
        let path = self.object_path(bucket, key);
        
        if path.exists() {
            fs::remove_file(&path).await?;
            debug!("Deleted object {}/{}", bucket, key);
        }
        
        Ok(())
    }

    async fn exists(&self, bucket: &str, key: &str) -> Result<bool> {
        let path = self.object_path(bucket, key);
        Ok(path.exists())
    }

    async fn size(&self, bucket: &str, key: &str) -> Result<i64> {
        let path = self.object_path(bucket, key);
        
        if !path.exists() {
            return Err(Error::NoSuchKey);
        }
        
        let metadata = fs::metadata(&path).await?;
        Ok(metadata.len() as i64)
    }

    async fn create_bucket(&self, bucket: &str) -> Result<()> {
        let path = self.bucket_path(bucket);
        fs::create_dir_all(path.join("objects")).await?;
        info!("Created bucket {}", bucket);
        Ok(())
    }

    async fn delete_bucket(&self, bucket: &str) -> Result<()> {
        let path = self.bucket_path(bucket);
        
        if path.exists() {
            // Check if bucket is empty
            let objects_path = path.join("objects");
            if objects_path.exists() {
                let mut entries = fs::read_dir(&objects_path).await?;
                if entries.next_entry().await?.is_some() {
                    return Err(Error::BucketNotEmpty);
                }
            }
            
            fs::remove_dir_all(&path).await?;
            info!("Deleted bucket {}", bucket);
        }
        
        Ok(())
    }

    async fn bucket_exists(&self, bucket: &str) -> Result<bool> {
        let path = self.bucket_path(bucket);
        Ok(path.exists())
    }
}

// Add seek import
use tokio::io::AsyncSeekExt;
