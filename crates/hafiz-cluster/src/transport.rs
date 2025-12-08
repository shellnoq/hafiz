//! Cluster transport layer for node-to-node communication
//!
//! Handles HTTP-based communication between cluster nodes with:
//! - TLS support for secure communication
//! - Automatic retry with exponential backoff
//! - Connection pooling
//! - Request timeouts

use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use reqwest::{Client, ClientBuilder};
use tracing::{debug, error, warn};

use hafiz_core::types::{ClusterMessage, ClusterNode, NodeId};

use crate::error::{ClusterError, ClusterResult};

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Base delay for exponential backoff
    pub retry_base_delay: Duration,
    /// Enable TLS certificate verification
    pub verify_tls: bool,
    /// Custom CA certificate path
    pub ca_cert_path: Option<String>,
    /// Client certificate path
    pub client_cert_path: Option<String>,
    /// Client key path
    pub client_key_path: Option<String>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            retry_base_delay: Duration::from_millis(100),
            verify_tls: true,
            ca_cert_path: None,
            client_cert_path: None,
            client_key_path: None,
        }
    }
}

/// Cluster transport for node communication
pub struct ClusterTransport {
    client: Client,
    config: TransportConfig,
}

impl ClusterTransport {
    /// Create a new transport with the given configuration
    pub fn new(config: TransportConfig) -> ClusterResult<Self> {
        let mut builder = ClientBuilder::new()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90));

        if !config.verify_tls {
            builder = builder.danger_accept_invalid_certs(true);
        }

        // TODO: Add custom CA and client certificates if provided

        let client = builder
            .build()
            .map_err(|e| ClusterError::Transport(e.to_string()))?;

        Ok(Self { client, config })
    }

    /// Send a message to a node
    pub async fn send_message(
        &self,
        node: &ClusterNode,
        message: &ClusterMessage,
    ) -> ClusterResult<ClusterMessage> {
        let url = format!("{}/cluster/message", node.cluster_endpoint);
        self.send_with_retry(&url, message).await
    }

    /// Send a join request to a seed node
    pub async fn send_join_request(
        &self,
        seed_endpoint: &str,
        message: &ClusterMessage,
    ) -> ClusterResult<ClusterMessage> {
        let url = format!("{}/cluster/join", seed_endpoint);
        self.send_with_retry(&url, message).await
    }

    /// Send a heartbeat to a node
    pub async fn send_heartbeat(
        &self,
        node: &ClusterNode,
        message: &ClusterMessage,
    ) -> ClusterResult<()> {
        let url = format!("{}/cluster/heartbeat", node.cluster_endpoint);
        let _: ClusterMessage = self.send_with_retry(&url, message).await?;
        Ok(())
    }

    /// Fetch object data from a node
    pub async fn fetch_object_data(
        &self,
        node: &ClusterNode,
        bucket: &str,
        key: &str,
        version_id: Option<&str>,
    ) -> ClusterResult<(Bytes, Option<String>)> {
        let mut url = format!(
            "{}/cluster/objects/{}/{}",
            node.cluster_endpoint, bucket, key
        );

        if let Some(vid) = version_id {
            url.push_str(&format!("?versionId={}", vid));
        }

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ClusterError::Transport(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ClusterError::Transport(format!(
                "Failed to fetch object: {}",
                response.status()
            )));
        }

        let checksum = response
            .headers()
            .get("x-hafiz-checksum")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let data = response
            .bytes()
            .await
            .map_err(|e| ClusterError::Transport(e.to_string()))?;

        Ok((data, checksum))
    }

    /// Upload object data to a node
    pub async fn upload_object_data(
        &self,
        node: &ClusterNode,
        bucket: &str,
        key: &str,
        data: Bytes,
        checksum: Option<&str>,
        metadata: &std::collections::HashMap<String, String>,
    ) -> ClusterResult<()> {
        let url = format!(
            "{}/cluster/objects/{}/{}",
            node.cluster_endpoint, bucket, key
        );

        let mut request = self.client.put(&url).body(data);

        if let Some(cs) = checksum {
            request = request.header("x-hafiz-checksum", cs);
        }

        for (k, v) in metadata {
            request = request.header(format!("x-hafiz-meta-{}", k), v);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ClusterError::Transport(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ClusterError::Transport(format!(
                "Failed to upload object: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Check if a node is reachable
    pub async fn ping(&self, node: &ClusterNode) -> ClusterResult<Duration> {
        let url = format!("{}/cluster/ping", node.cluster_endpoint);
        let start = std::time::Instant::now();

        let response = self
            .client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await;

        match response {
            Ok(r) if r.status().is_success() => Ok(start.elapsed()),
            Ok(r) => Err(ClusterError::NodeUnreachable(format!(
                "Ping failed with status: {}",
                r.status()
            ))),
            Err(e) => Err(ClusterError::NodeUnreachable(e.to_string())),
        }
    }

    /// Send a message with retry logic
    async fn send_with_retry<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        message: &ClusterMessage,
    ) -> ClusterResult<T> {
        let mut last_error = None;
        let mut delay = self.config.retry_base_delay;

        for attempt in 0..=self.config.max_retries {
            if attempt > 0 {
                debug!("Retry attempt {} for {}", attempt, url);
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }

            match self.send_once::<T>(url, message).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!("Request to {} failed (attempt {}): {}", url, attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ClusterError::Transport("Unknown error".to_string())))
    }

    /// Send a single request without retry
    async fn send_once<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
        message: &ClusterMessage,
    ) -> ClusterResult<T> {
        let response = self
            .client
            .post(url)
            .json(message)
            .send()
            .await
            .map_err(|e| ClusterError::Transport(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ClusterError::Transport(format!(
                "Request failed with status {}: {}",
                status, body
            )));
        }

        response
            .json()
            .await
            .map_err(|e| ClusterError::Transport(e.to_string()))
    }
}

impl std::fmt::Debug for ClusterTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClusterTransport")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert!(config.verify_tls);
    }
}
