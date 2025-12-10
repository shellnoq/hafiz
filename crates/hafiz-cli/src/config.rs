//! Configuration management for Hafiz CLI
//!
//! Config file location: ~/.hafiz/config.toml
//!
//! Example config:
//! ```toml
//! [default]
//! endpoint = "http://localhost:9000"
//! access_key = "minioadmin"
//! secret_key = "minioadmin"
//! region = "us-east-1"
//!
//! [production]
//! endpoint = "https://s3.example.com"
//! access_key = "prod-access-key"
//! secret_key = "prod-secret-key"
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// S3 endpoint URL
    pub endpoint: Option<String>,

    /// Access key ID
    pub access_key: Option<String>,

    /// Secret access key
    pub secret_key: Option<String>,

    /// AWS region
    #[serde(default = "default_region")]
    pub region: String,

    /// Path style access (use path instead of virtual hosted style)
    #[serde(default = "default_true")]
    pub path_style: bool,

    /// Signature version (v2 or v4)
    #[serde(default = "default_sig_version")]
    pub signature_version: String,

    /// Default storage class
    pub storage_class: Option<String>,

    /// Multipart upload threshold (bytes)
    #[serde(default = "default_multipart_threshold")]
    pub multipart_threshold: u64,

    /// Multipart chunk size (bytes)
    #[serde(default = "default_multipart_chunksize")]
    pub multipart_chunksize: u64,

    /// Maximum concurrent transfers
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: usize,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_region() -> String {
    "us-east-1".to_string()
}

fn default_true() -> bool {
    true
}

fn default_sig_version() -> String {
    "v4".to_string()
}

fn default_multipart_threshold() -> u64 {
    8 * 1024 * 1024 // 8MB
}

fn default_multipart_chunksize() -> u64 {
    8 * 1024 * 1024 // 8MB
}

fn default_max_concurrent() -> usize {
    10
}

fn default_timeout() -> u64 {
    300
}

/// Configuration file with multiple profiles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(flatten)]
    pub profiles: HashMap<String, Config>,
}

impl Config {
    /// Get config directory path
    pub fn config_dir() -> Result<PathBuf> {
        let home = directories::BaseDirs::new()
            .context("Could not determine home directory")?
            .home_dir()
            .to_path_buf();

        Ok(home.join(".hafiz"))
    }

    /// Get config file path
    pub fn config_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Load configuration from file and environment
    pub fn load(profile: Option<&str>) -> Result<Self> {
        let profile_name = profile.unwrap_or("default");

        // Try to load from config file
        let config_path = Self::config_path()?;
        let mut config = if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

            let config_file: ConfigFile =
                toml::from_str(&content).with_context(|| "Failed to parse config file")?;

            config_file
                .profiles
                .get(profile_name)
                .cloned()
                .unwrap_or_default()
        } else {
            Config::default()
        };

        // Override with environment variables if present
        if let Ok(endpoint) = std::env::var("HAFIZ_ENDPOINT") {
            config.endpoint = Some(endpoint);
        }
        if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
            config.endpoint = Some(endpoint);
        }
        if let Ok(access_key) = std::env::var("HAFIZ_ACCESS_KEY") {
            config.access_key = Some(access_key);
        }
        if let Ok(access_key) = std::env::var("AWS_ACCESS_KEY_ID") {
            config.access_key = Some(access_key);
        }
        if let Ok(secret_key) = std::env::var("HAFIZ_SECRET_KEY") {
            config.secret_key = Some(secret_key);
        }
        if let Ok(secret_key) = std::env::var("AWS_SECRET_ACCESS_KEY") {
            config.secret_key = Some(secret_key);
        }
        if let Ok(region) = std::env::var("HAFIZ_REGION") {
            config.region = region;
        }
        if let Ok(region) = std::env::var("AWS_REGION") {
            config.region = region;
        }

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self, profile: Option<&str>) -> Result<()> {
        let profile_name = profile.unwrap_or("default");
        let config_path = Self::config_path()?;

        // Create config directory if needed
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load existing config or create new
        let mut config_file = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            toml::from_str(&content).unwrap_or_default()
        } else {
            ConfigFile::default()
        };

        // Update profile
        config_file
            .profiles
            .insert(profile_name.to_string(), self.clone());

        // Write back
        let content = toml::to_string_pretty(&config_file)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    /// List all profiles
    pub fn list_profiles() -> Result<Vec<String>> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(vec![]);
        }

        let content = fs::read_to_string(&config_path)?;
        let config_file: ConfigFile = toml::from_str(&content)?;

        Ok(config_file.profiles.keys().cloned().collect())
    }

    /// Delete a profile
    pub fn delete_profile(profile: &str) -> Result<()> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(&config_path)?;
        let mut config_file: ConfigFile = toml::from_str(&content)?;

        config_file.profiles.remove(profile);

        let content = toml::to_string_pretty(&config_file)?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.endpoint.is_none() {
            anyhow::bail!("Endpoint not configured. Set HAFIZ_ENDPOINT or use 'hafiz configure'");
        }

        if self.access_key.is_none() {
            anyhow::bail!(
                "Access key not configured. Set HAFIZ_ACCESS_KEY or use 'hafiz configure'"
            );
        }

        if self.secret_key.is_none() {
            anyhow::bail!(
                "Secret key not configured. Set HAFIZ_SECRET_KEY or use 'hafiz configure'"
            );
        }

        Ok(())
    }

    /// Get a config value by key name
    pub fn get_value(&self, key: &str) -> Option<String> {
        match key {
            "endpoint" => self.endpoint.clone(),
            "access_key" => self.access_key.clone(),
            "secret_key" => self.secret_key.as_ref().map(|_| "***".to_string()), // Hide secret
            "region" => Some(self.region.clone()),
            "path_style" => Some(self.path_style.to_string()),
            "signature_version" => Some(self.signature_version.clone()),
            "storage_class" => self.storage_class.clone(),
            "multipart_threshold" => Some(self.multipart_threshold.to_string()),
            "multipart_chunksize" => Some(self.multipart_chunksize.to_string()),
            "max_concurrent_requests" => Some(self.max_concurrent_requests.to_string()),
            "timeout" => Some(self.timeout.to_string()),
            _ => None,
        }
    }

    /// Set a config value by key name
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "endpoint" => self.endpoint = Some(value.to_string()),
            "access_key" => self.access_key = Some(value.to_string()),
            "secret_key" => self.secret_key = Some(value.to_string()),
            "region" => self.region = value.to_string(),
            "path_style" => self.path_style = value.parse()?,
            "signature_version" => self.signature_version = value.to_string(),
            "storage_class" => self.storage_class = Some(value.to_string()),
            "multipart_threshold" => self.multipart_threshold = value.parse()?,
            "multipart_chunksize" => self.multipart_chunksize = value.parse()?,
            "max_concurrent_requests" => self.max_concurrent_requests = value.parse()?,
            "timeout" => self.timeout = value.parse()?,
            _ => anyhow::bail!("Unknown config key: {}", key),
        }
        Ok(())
    }

    /// Get all config keys
    pub fn keys() -> &'static [&'static str] {
        &[
            "endpoint",
            "access_key",
            "secret_key",
            "region",
            "path_style",
            "signature_version",
            "storage_class",
            "multipart_threshold",
            "multipart_chunksize",
            "max_concurrent_requests",
            "timeout",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.region, "us-east-1");
        assert!(config.path_style);
        assert_eq!(config.signature_version, "v4");
    }

    #[test]
    fn test_config_serialization() {
        let mut config = Config::default();
        config.endpoint = Some("http://localhost:9000".to_string());
        config.access_key = Some("test".to_string());
        config.secret_key = Some("secret".to_string());

        let mut profiles = HashMap::new();
        profiles.insert("default".to_string(), config);

        let config_file = ConfigFile { profiles };
        let toml = toml::to_string(&config_file).unwrap();

        assert!(toml.contains("endpoint"));
        assert!(toml.contains("localhost:9000"));
    }
}
