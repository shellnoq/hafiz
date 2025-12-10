//! Event Dispatcher
//!
//! Handles dispatching S3 events to configured notification targets.

use chrono::Utc;
use hafiz_core::types::{
    NotificationConfiguration, NotificationTarget, S3EventMessage, S3EventRecord, S3EventType,
};
use reqwest::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Event to be dispatched
#[derive(Debug, Clone)]
pub struct S3Event {
    pub event_type: S3EventType,
    pub bucket: String,
    pub key: String,
    pub size: i64,
    pub etag: String,
    pub version_id: Option<String>,
    pub request_id: String,
    pub principal_id: String,
    pub source_ip: String,
    pub region: String,
}

/// Event dispatcher configuration
#[derive(Debug, Clone)]
pub struct EventDispatcherConfig {
    /// HTTP client timeout
    pub timeout: Duration,
    /// Maximum retries for failed deliveries
    pub max_retries: u32,
    /// Retry delay
    pub retry_delay: Duration,
    /// Worker count for async dispatch
    pub worker_count: usize,
    /// Queue capacity
    pub queue_capacity: usize,
}

impl Default for EventDispatcherConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            retry_delay: Duration::from_secs(1),
            worker_count: 4,
            queue_capacity: 10000,
        }
    }
}

/// Event dispatcher handle
#[derive(Clone)]
pub struct EventDispatcher {
    sender: mpsc::Sender<DispatchTask>,
    http_client: Client,
    config: EventDispatcherConfig,
}

struct DispatchTask {
    event: S3Event,
    targets: Vec<NotificationTarget>,
    config_id: String,
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new(config: EventDispatcherConfig) -> Self {
        let (sender, receiver) = mpsc::channel(config.queue_capacity);
        let http_client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        let dispatcher = Self {
            sender,
            http_client: http_client.clone(),
            config: config.clone(),
        };

        // Start worker tasks
        let worker_config = config.clone();
        tokio::spawn(Self::dispatch_worker(receiver, http_client, worker_config));

        dispatcher
    }

    /// Dispatch an event to all matching targets
    pub async fn dispatch(
        &self,
        event: S3Event,
        notification_config: &NotificationConfiguration,
    ) -> Result<(), String> {
        let targets = notification_config.get_matching_configs(&event.event_type, &event.key);

        if targets.is_empty() {
            debug!(
                "No matching notification targets for event {:?} on {}/{}",
                event.event_type, event.bucket, event.key
            );
            return Ok(());
        }

        info!(
            "Dispatching event {:?} to {} targets for {}/{}",
            event.event_type,
            targets.len(),
            event.bucket,
            event.key
        );

        let task = DispatchTask {
            event,
            targets,
            config_id: "default".to_string(),
        };

        self.sender
            .send(task)
            .await
            .map_err(|e| format!("Failed to queue event: {}", e))
    }

    /// Dispatch event synchronously (blocking)
    pub async fn dispatch_sync(
        &self,
        event: S3Event,
        notification_config: &NotificationConfiguration,
    ) -> Vec<DispatchResult> {
        let targets = notification_config.get_matching_configs(&event.event_type, &event.key);

        if targets.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();

        for target in targets {
            let config_id = match &target {
                NotificationTarget::Webhook { id, .. } => id.clone(),
                NotificationTarget::Queue { id, .. } => id.clone(),
                NotificationTarget::Topic { id, .. } => id.clone(),
            };

            let record = S3EventRecord::new(
                event.event_type.clone(),
                &event.bucket,
                &event.key,
                event.size,
                &event.etag,
                event.version_id.clone(),
                &event.request_id,
                &event.principal_id,
                &event.source_ip,
                &config_id,
                &event.region,
            );

            let message = S3EventMessage {
                records: vec![record],
            };

            let result = self.deliver_to_target(&target, &message).await;

            results.push(DispatchResult {
                config_id,
                success: result.is_ok(),
                error: result.err(),
            });
        }

        results
    }

    async fn dispatch_worker(
        mut receiver: mpsc::Receiver<DispatchTask>,
        http_client: Client,
        config: EventDispatcherConfig,
    ) {
        info!("Event dispatch worker started");

        while let Some(task) = receiver.recv().await {
            for target in task.targets {
                let config_id = match &target {
                    NotificationTarget::Webhook { id, .. } => id.clone(),
                    NotificationTarget::Queue { id, .. } => id.clone(),
                    NotificationTarget::Topic { id, .. } => id.clone(),
                };

                let record = S3EventRecord::new(
                    task.event.event_type.clone(),
                    &task.event.bucket,
                    &task.event.key,
                    task.event.size,
                    &task.event.etag,
                    task.event.version_id.clone(),
                    &task.event.request_id,
                    &task.event.principal_id,
                    &task.event.source_ip,
                    &config_id,
                    &task.event.region,
                );

                let message = S3EventMessage {
                    records: vec![record],
                };

                let mut attempts = 0;
                loop {
                    attempts += 1;

                    match Self::deliver_to_target_static(&http_client, &target, &message).await {
                        Ok(_) => {
                            debug!(
                                "Successfully delivered event to {} (attempt {})",
                                config_id, attempts
                            );
                            break;
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deliver event to {} (attempt {}): {}",
                                config_id, attempts, e
                            );

                            if attempts >= config.max_retries {
                                error!(
                                    "Giving up on event delivery to {} after {} attempts",
                                    config_id, attempts
                                );
                                break;
                            }

                            tokio::time::sleep(config.retry_delay * attempts).await;
                        }
                    }
                }
            }
        }

        info!("Event dispatch worker stopped");
    }

    async fn deliver_to_target(
        &self,
        target: &NotificationTarget,
        message: &S3EventMessage,
    ) -> Result<(), String> {
        Self::deliver_to_target_static(&self.http_client, target, message).await
    }

    async fn deliver_to_target_static(
        http_client: &Client,
        target: &NotificationTarget,
        message: &S3EventMessage,
    ) -> Result<(), String> {
        match target {
            NotificationTarget::Webhook {
                url,
                headers,
                auth_token,
                ..
            } => {
                let json = serde_json::to_string(message)
                    .map_err(|e| format!("Failed to serialize event: {}", e))?;

                let mut request = http_client
                    .post(url)
                    .header("Content-Type", "application/json")
                    .body(json);

                // Add custom headers
                if let Some(headers) = headers {
                    for (key, value) in headers {
                        request = request.header(key.as_str(), value.as_str());
                    }
                }

                // Add auth token
                if let Some(token) = auth_token {
                    request = request.header("Authorization", format!("Bearer {}", token));
                }

                let response = request
                    .send()
                    .await
                    .map_err(|e| format!("HTTP request failed: {}", e))?;

                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(format!(
                        "Webhook returned error status: {}",
                        response.status()
                    ))
                }
            }
            NotificationTarget::Queue { arn, .. } => {
                // For queue targets, we would integrate with SQS-compatible service
                // For now, log the event
                debug!("Would send event to queue: {}", arn);
                Ok(())
            }
            NotificationTarget::Topic { arn, .. } => {
                // For topic targets, we would integrate with SNS-compatible service
                // For now, log the event
                debug!("Would send event to topic: {}", arn);
                Ok(())
            }
        }
    }
}

/// Result of a dispatch operation
#[derive(Debug, Clone)]
pub struct DispatchResult {
    pub config_id: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Simple in-memory notification configuration store
#[derive(Default)]
pub struct NotificationConfigStore {
    configs: tokio::sync::RwLock<HashMap<String, NotificationConfiguration>>,
}

impl NotificationConfigStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, bucket: &str) -> Option<NotificationConfiguration> {
        self.configs.read().await.get(bucket).cloned()
    }

    pub async fn put(&self, bucket: &str, config: NotificationConfiguration) {
        self.configs
            .write()
            .await
            .insert(bucket.to_string(), config);
    }

    pub async fn delete(&self, bucket: &str) {
        self.configs.write().await.remove(bucket);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hafiz_core::types::{NotificationFilter, S3KeyFilter, WebhookConfiguration};

    #[tokio::test]
    async fn test_dispatcher_no_targets() {
        let config = EventDispatcherConfig::default();
        let dispatcher = EventDispatcher::new(config);

        let event = S3Event {
            event_type: S3EventType::ObjectCreatedPut,
            bucket: "test-bucket".to_string(),
            key: "test-key".to_string(),
            size: 100,
            etag: "abc123".to_string(),
            version_id: None,
            request_id: "req-123".to_string(),
            principal_id: "user-123".to_string(),
            source_ip: "127.0.0.1".to_string(),
            region: "us-east-1".to_string(),
        };

        let notification_config = NotificationConfiguration::new();
        let result = dispatcher.dispatch(event, &notification_config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_event_record_creation() {
        let record = S3EventRecord::new(
            S3EventType::ObjectCreatedPut,
            "my-bucket",
            "path/to/object.txt",
            1024,
            "etag-123",
            Some("version-1".to_string()),
            "request-456",
            "user-789",
            "192.168.1.1",
            "config-1",
            "us-east-1",
        );

        assert_eq!(record.event_name, "s3:ObjectCreated:Put");
        assert_eq!(record.s3.bucket.name, "my-bucket");
        assert_eq!(record.s3.object.key, "path/to/object.txt");
        assert_eq!(record.s3.object.size, 1024);
    }
}
