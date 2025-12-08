//! Configuration for Hafiz

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HafizConfig {
    #[serde(default)]
    pub server: ServerConfig,
    
    #[serde(default)]
    pub tls: TlsConfig,
    
    #[serde(default)]
    pub storage: StorageConfig,
    
    #[serde(default)]
    pub database: DatabaseConfig,
    
    #[serde(default)]
    pub auth: AuthConfig,
    
    #[serde(default)]
    pub encryption: EncryptionConfig,
    
    #[serde(default)]
    pub logging: LoggingConfig,
    
    #[serde(default)]
    pub lifecycle: LifecycleWorkerConfig,
}

impl Default for HafizConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            tls: TlsConfig::default(),
            storage: StorageConfig::default(),
            database: DatabaseConfig::default(),
            auth: AuthConfig::default(),
            encryption: EncryptionConfig::default(),
            logging: LoggingConfig::default(),
            lifecycle: LifecycleWorkerConfig::default(),
        }
    }
}

impl HafizConfig {
    pub fn from_file(path: &str) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::InternalError(format!("Failed to read config: {}", e)))?;
        
        toml::from_str(&content)
            .map_err(|e| crate::Error::InternalError(format!("Failed to parse config: {}", e)))
    }

    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        if let Ok(addr) = std::env::var("HAFIZ_BIND_ADDRESS") {
            config.server.bind_address = addr;
        }
        if let Ok(port) = std::env::var("HAFIZ_PORT") {
            if let Ok(p) = port.parse() {
                config.server.port = p;
            }
        }
        if let Ok(dir) = std::env::var("HAFIZ_DATA_DIR") {
            config.storage.data_dir = PathBuf::from(dir);
        }
        if let Ok(url) = std::env::var("HAFIZ_DATABASE_URL") {
            config.database.url = url;
        }
        if let Ok(key) = std::env::var("HAFIZ_ROOT_ACCESS_KEY") {
            config.auth.root_access_key = key;
        }
        if let Ok(secret) = std::env::var("HAFIZ_ROOT_SECRET_KEY") {
            config.auth.root_secret_key = secret;
        }
        if let Ok(level) = std::env::var("HAFIZ_LOG_LEVEL") {
            config.logging.level = level;
        }
        
        // TLS from environment
        if let Ok(cert) = std::env::var("HAFIZ_TLS_CERT") {
            config.tls.enabled = true;
            config.tls.cert_file = Some(PathBuf::from(cert));
        }
        if let Ok(key) = std::env::var("HAFIZ_TLS_KEY") {
            config.tls.key_file = Some(PathBuf::from(key));
        }
        
        // Encryption from environment
        if let Ok(key) = std::env::var("HAFIZ_ENCRYPTION_KEY") {
            config.encryption.enabled = true;
            config.encryption.master_key = Some(key);
        }
        if std::env::var("HAFIZ_SSE_S3_ENABLED").map(|v| v == "true").unwrap_or(false) {
            config.encryption.sse_s3_enabled = true;
        }
        if std::env::var("HAFIZ_SSE_C_ENABLED").map(|v| v == "true").unwrap_or(false) {
            config.encryption.sse_c_enabled = true;
        }
        
        config
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub admin_port: u16,
    pub workers: usize,
    pub max_connections: usize,
    pub request_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0".to_string(),
            port: 9000,
            admin_port: 9001,
            workers: num_cpus::get(),
            max_connections: 10000,
            request_timeout_secs: 300,
        }
    }
}

/// TLS/HTTPS Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Enable TLS
    pub enabled: bool,
    /// Path to certificate file (PEM format)
    pub cert_file: Option<PathBuf>,
    /// Path to private key file (PEM format)
    pub key_file: Option<PathBuf>,
    /// Path to CA certificate for client verification (mTLS)
    pub client_ca_file: Option<PathBuf>,
    /// Require client certificate (mTLS)
    pub require_client_cert: bool,
    /// Minimum TLS version (1.2 or 1.3)
    pub min_version: TlsVersion,
    /// Enable HSTS header
    pub hsts_enabled: bool,
    /// HSTS max age in seconds
    pub hsts_max_age: u64,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cert_file: None,
            key_file: None,
            client_ca_file: None,
            require_client_cert: false,
            min_version: TlsVersion::Tls12,
            hsts_enabled: true,
            hsts_max_age: 31536000, // 1 year
        }
    }
}

impl TlsConfig {
    pub fn validate(&self) -> crate::Result<()> {
        if self.enabled {
            if self.cert_file.is_none() {
                return Err(crate::Error::InvalidArgument(
                    "TLS enabled but cert_file not specified".into(),
                ));
            }
            if self.key_file.is_none() {
                return Err(crate::Error::InvalidArgument(
                    "TLS enabled but key_file not specified".into(),
                ));
            }
            
            // Check files exist
            if let Some(ref cert) = self.cert_file {
                if !cert.exists() {
                    return Err(crate::Error::InvalidArgument(format!(
                        "Certificate file not found: {:?}",
                        cert
                    )));
                }
            }
            if let Some(ref key) = self.key_file {
                if !key.exists() {
                    return Err(crate::Error::InvalidArgument(format!(
                        "Key file not found: {:?}",
                        key
                    )));
                }
            }
            if self.require_client_cert {
                if let Some(ref ca) = self.client_ca_file {
                    if !ca.exists() {
                        return Err(crate::Error::InvalidArgument(format!(
                            "Client CA file not found: {:?}",
                            ca
                        )));
                    }
                } else {
                    return Err(crate::Error::InvalidArgument(
                        "Client cert required but client_ca_file not specified".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    #[serde(rename = "1.2")]
    Tls12,
    #[serde(rename = "1.3")]
    Tls13,
}

impl Default for TlsVersion {
    fn default() -> Self {
        Self::Tls12
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub max_object_size: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("/data/hafiz"),
            temp_dir: PathBuf::from("/tmp/hafiz"),
            max_object_size: crate::MAX_OBJECT_SIZE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:///data/hafiz/hafiz.db?mode=rwc".to_string(),
            max_connections: 100,
            min_connections: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub root_access_key: String,
    pub root_secret_key: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            root_access_key: "minioadmin".to_string(),
            root_secret_key: "minioadmin".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
        }
    }
}

/// Server-Side Encryption Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Enable encryption subsystem
    pub enabled: bool,
    /// Enable SSE-S3 (server-managed keys)
    pub sse_s3_enabled: bool,
    /// Enable SSE-C (customer-provided keys)
    pub sse_c_enabled: bool,
    /// Master encryption key (hex encoded, 32 bytes = 64 hex chars)
    /// For production: use key_file or key_env
    pub master_key: Option<String>,
    /// Path to file containing master key
    pub master_key_file: Option<PathBuf>,
    /// Environment variable containing master key
    pub master_key_env: Option<String>,
    /// Default encryption for new objects (none, AES256)
    pub default_encryption: DefaultEncryption,
}

impl Default for EncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            sse_s3_enabled: true,
            sse_c_enabled: true,
            master_key: None,
            master_key_file: None,
            master_key_env: None,
            default_encryption: DefaultEncryption::None,
        }
    }
}

impl EncryptionConfig {
    /// Get master key from configured source
    pub fn get_master_key(&self) -> crate::Result<Option<Vec<u8>>> {
        if !self.enabled {
            return Ok(None);
        }
        
        // Try direct key first
        if let Some(ref key) = self.master_key {
            let bytes = hex::decode(key)
                .map_err(|e| crate::Error::InvalidArgument(format!("Invalid master key hex: {}", e)))?;
            if bytes.len() != 32 {
                return Err(crate::Error::InvalidArgument(
                    "Master key must be 32 bytes (64 hex characters)".into(),
                ));
            }
            return Ok(Some(bytes));
        }
        
        // Try key file
        if let Some(ref path) = self.master_key_file {
            let content = std::fs::read_to_string(path)
                .map_err(|e| crate::Error::InternalError(format!("Failed to read key file: {}", e)))?;
            let bytes = hex::decode(content.trim())
                .map_err(|e| crate::Error::InvalidArgument(format!("Invalid master key in file: {}", e)))?;
            if bytes.len() != 32 {
                return Err(crate::Error::InvalidArgument(
                    "Master key must be 32 bytes (64 hex characters)".into(),
                ));
            }
            return Ok(Some(bytes));
        }
        
        // Try environment variable
        if let Some(ref env_var) = self.master_key_env {
            if let Ok(key) = std::env::var(env_var) {
                let bytes = hex::decode(&key)
                    .map_err(|e| crate::Error::InvalidArgument(format!("Invalid master key in env: {}", e)))?;
                if bytes.len() != 32 {
                    return Err(crate::Error::InvalidArgument(
                        "Master key must be 32 bytes (64 hex characters)".into(),
                    ));
                }
                return Ok(Some(bytes));
            }
        }
        
        Err(crate::Error::InvalidArgument(
            "Encryption enabled but no master key configured".into(),
        ))
    }
    
    pub fn validate(&self) -> crate::Result<()> {
        if self.enabled {
            // Ensure at least one key source is configured
            if self.master_key.is_none() 
                && self.master_key_file.is_none() 
                && self.master_key_env.is_none() 
            {
                return Err(crate::Error::InvalidArgument(
                    "Encryption enabled but no master key source configured".into(),
                ));
            }
        }
        Ok(())
    }
}

/// Default encryption type for new objects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DefaultEncryption {
    /// No encryption by default
    None,
    /// AES-256 encryption by default
    #[serde(rename = "AES256")]
    Aes256,
}

impl Default for DefaultEncryption {
    fn default() -> Self {
        Self::None
    }
}

/// Lifecycle worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleWorkerConfig {
    /// Enable lifecycle worker
    pub enabled: bool,
    /// Interval between scans in seconds
    pub scan_interval_secs: u64,
    /// Batch size for processing objects
    pub batch_size: usize,
}

impl Default for LifecycleWorkerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scan_interval_secs: 3600, // 1 hour
            batch_size: 1000,
        }
    }
}

// Helper for num_cpus in default
mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4)
    }
}
