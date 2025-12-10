//! Event Notification types
//!
//! S3-compatible event notification configuration supporting:
//! - Webhook destinations (HTTP/HTTPS)
//! - Queue destinations (SQS-compatible)
//! - Topic destinations (SNS-compatible)
//! - Event filtering by prefix/suffix

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Event Types
// ============================================================================

/// S3 Event types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum S3EventType {
    // Object Created events
    #[serde(rename = "s3:ObjectCreated:*")]
    ObjectCreatedAll,
    #[serde(rename = "s3:ObjectCreated:Put")]
    ObjectCreatedPut,
    #[serde(rename = "s3:ObjectCreated:Post")]
    ObjectCreatedPost,
    #[serde(rename = "s3:ObjectCreated:Copy")]
    ObjectCreatedCopy,
    #[serde(rename = "s3:ObjectCreated:CompleteMultipartUpload")]
    ObjectCreatedCompleteMultipartUpload,

    // Object Removed events
    #[serde(rename = "s3:ObjectRemoved:*")]
    ObjectRemovedAll,
    #[serde(rename = "s3:ObjectRemoved:Delete")]
    ObjectRemovedDelete,
    #[serde(rename = "s3:ObjectRemoved:DeleteMarkerCreated")]
    ObjectRemovedDeleteMarkerCreated,

    // Object Restore events
    #[serde(rename = "s3:ObjectRestore:*")]
    ObjectRestoreAll,
    #[serde(rename = "s3:ObjectRestore:Post")]
    ObjectRestorePost,
    #[serde(rename = "s3:ObjectRestore:Completed")]
    ObjectRestoreCompleted,

    // Replication events
    #[serde(rename = "s3:Replication:*")]
    ReplicationAll,
    #[serde(rename = "s3:Replication:OperationFailedReplication")]
    ReplicationFailed,
    #[serde(rename = "s3:Replication:OperationMissedThreshold")]
    ReplicationMissedThreshold,
    #[serde(rename = "s3:Replication:OperationReplicatedAfterThreshold")]
    ReplicationAfterThreshold,

    // Lifecycle events
    #[serde(rename = "s3:LifecycleExpiration:*")]
    LifecycleExpirationAll,
    #[serde(rename = "s3:LifecycleExpiration:Delete")]
    LifecycleExpirationDelete,
    #[serde(rename = "s3:LifecycleExpiration:DeleteMarkerCreated")]
    LifecycleExpirationDeleteMarkerCreated,

    // Tagging events
    #[serde(rename = "s3:ObjectTagging:*")]
    ObjectTaggingAll,
    #[serde(rename = "s3:ObjectTagging:Put")]
    ObjectTaggingPut,
    #[serde(rename = "s3:ObjectTagging:Delete")]
    ObjectTaggingDelete,

    // ACL events
    #[serde(rename = "s3:ObjectAcl:Put")]
    ObjectAclPut,

    // Test event
    #[serde(rename = "s3:TestEvent")]
    TestEvent,
}

impl S3EventType {
    /// Check if this event type matches another (including wildcards)
    pub fn matches(&self, other: &S3EventType) -> bool {
        if self == other {
            return true;
        }

        // Check wildcard matches
        match self {
            S3EventType::ObjectCreatedAll => matches!(
                other,
                S3EventType::ObjectCreatedPut
                    | S3EventType::ObjectCreatedPost
                    | S3EventType::ObjectCreatedCopy
                    | S3EventType::ObjectCreatedCompleteMultipartUpload
            ),
            S3EventType::ObjectRemovedAll => matches!(
                other,
                S3EventType::ObjectRemovedDelete | S3EventType::ObjectRemovedDeleteMarkerCreated
            ),
            S3EventType::ObjectRestoreAll => matches!(
                other,
                S3EventType::ObjectRestorePost | S3EventType::ObjectRestoreCompleted
            ),
            S3EventType::ReplicationAll => matches!(
                other,
                S3EventType::ReplicationFailed
                    | S3EventType::ReplicationMissedThreshold
                    | S3EventType::ReplicationAfterThreshold
            ),
            S3EventType::LifecycleExpirationAll => matches!(
                other,
                S3EventType::LifecycleExpirationDelete
                    | S3EventType::LifecycleExpirationDeleteMarkerCreated
            ),
            S3EventType::ObjectTaggingAll => {
                matches!(
                    other,
                    S3EventType::ObjectTaggingPut | S3EventType::ObjectTaggingDelete
                )
            }
            _ => false,
        }
    }

    /// Get event name for JSON
    pub fn as_str(&self) -> &'static str {
        match self {
            S3EventType::ObjectCreatedAll => "s3:ObjectCreated:*",
            S3EventType::ObjectCreatedPut => "s3:ObjectCreated:Put",
            S3EventType::ObjectCreatedPost => "s3:ObjectCreated:Post",
            S3EventType::ObjectCreatedCopy => "s3:ObjectCreated:Copy",
            S3EventType::ObjectCreatedCompleteMultipartUpload => {
                "s3:ObjectCreated:CompleteMultipartUpload"
            }
            S3EventType::ObjectRemovedAll => "s3:ObjectRemoved:*",
            S3EventType::ObjectRemovedDelete => "s3:ObjectRemoved:Delete",
            S3EventType::ObjectRemovedDeleteMarkerCreated => "s3:ObjectRemoved:DeleteMarkerCreated",
            S3EventType::ObjectRestoreAll => "s3:ObjectRestore:*",
            S3EventType::ObjectRestorePost => "s3:ObjectRestore:Post",
            S3EventType::ObjectRestoreCompleted => "s3:ObjectRestore:Completed",
            S3EventType::ReplicationAll => "s3:Replication:*",
            S3EventType::ReplicationFailed => "s3:Replication:OperationFailedReplication",
            S3EventType::ReplicationMissedThreshold => "s3:Replication:OperationMissedThreshold",
            S3EventType::ReplicationAfterThreshold => {
                "s3:Replication:OperationReplicatedAfterThreshold"
            }
            S3EventType::LifecycleExpirationAll => "s3:LifecycleExpiration:*",
            S3EventType::LifecycleExpirationDelete => "s3:LifecycleExpiration:Delete",
            S3EventType::LifecycleExpirationDeleteMarkerCreated => {
                "s3:LifecycleExpiration:DeleteMarkerCreated"
            }
            S3EventType::ObjectTaggingAll => "s3:ObjectTagging:*",
            S3EventType::ObjectTaggingPut => "s3:ObjectTagging:Put",
            S3EventType::ObjectTaggingDelete => "s3:ObjectTagging:Delete",
            S3EventType::ObjectAclPut => "s3:ObjectAcl:Put",
            S3EventType::TestEvent => "s3:TestEvent",
        }
    }
}

impl std::fmt::Display for S3EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Filter Rules
// ============================================================================

/// Filter rule for key name
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct FilterRule {
    /// Filter name: "prefix" or "suffix"
    pub name: String,
    /// Filter value
    pub value: String,
}

/// S3 Key filter
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct S3KeyFilter {
    /// Filter rules
    #[serde(default)]
    pub filter_rules: Vec<FilterRule>,
}

impl S3KeyFilter {
    /// Create a new prefix filter
    pub fn prefix(value: impl Into<String>) -> Self {
        Self {
            filter_rules: vec![FilterRule {
                name: "prefix".to_string(),
                value: value.into(),
            }],
        }
    }

    /// Create a new suffix filter
    pub fn suffix(value: impl Into<String>) -> Self {
        Self {
            filter_rules: vec![FilterRule {
                name: "suffix".to_string(),
                value: value.into(),
            }],
        }
    }

    /// Add prefix filter
    pub fn with_prefix(mut self, value: impl Into<String>) -> Self {
        self.filter_rules.push(FilterRule {
            name: "prefix".to_string(),
            value: value.into(),
        });
        self
    }

    /// Add suffix filter
    pub fn with_suffix(mut self, value: impl Into<String>) -> Self {
        self.filter_rules.push(FilterRule {
            name: "suffix".to_string(),
            value: value.into(),
        });
        self
    }

    /// Check if a key matches the filter
    pub fn matches(&self, key: &str) -> bool {
        for rule in &self.filter_rules {
            match rule.name.to_lowercase().as_str() {
                "prefix" => {
                    if !key.starts_with(&rule.value) {
                        return false;
                    }
                }
                "suffix" => {
                    if !key.ends_with(&rule.value) {
                        return false;
                    }
                }
                _ => {}
            }
        }
        true
    }
}

/// Notification filter
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NotificationFilter {
    /// Key filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<S3KeyFilter>,
}

// ============================================================================
// Notification Configurations
// ============================================================================

/// Webhook notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct WebhookConfiguration {
    /// Configuration ID
    pub id: String,
    /// Webhook URL (HTTP/HTTPS)
    pub url: String,
    /// Events to notify
    pub events: Vec<S3EventType>,
    /// Optional filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<NotificationFilter>,
    /// Optional headers to send
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Authentication token (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

/// Queue notification configuration (SQS-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct QueueConfiguration {
    /// Configuration ID
    pub id: String,
    /// Queue ARN
    pub queue_arn: String,
    /// Events to notify
    pub events: Vec<S3EventType>,
    /// Optional filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<NotificationFilter>,
}

/// Topic notification configuration (SNS-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TopicConfiguration {
    /// Configuration ID
    pub id: String,
    /// Topic ARN
    pub topic_arn: String,
    /// Events to notify
    pub events: Vec<S3EventType>,
    /// Optional filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<NotificationFilter>,
}

/// Complete bucket notification configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct NotificationConfiguration {
    /// Webhook configurations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub webhook_configurations: Vec<WebhookConfiguration>,
    /// Queue configurations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queue_configurations: Vec<QueueConfiguration>,
    /// Topic configurations
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topic_configurations: Vec<TopicConfiguration>,
}

impl NotificationConfiguration {
    /// Create empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add webhook configuration
    pub fn add_webhook(mut self, config: WebhookConfiguration) -> Self {
        self.webhook_configurations.push(config);
        self
    }

    /// Add queue configuration
    pub fn add_queue(mut self, config: QueueConfiguration) -> Self {
        self.queue_configurations.push(config);
        self
    }

    /// Add topic configuration
    pub fn add_topic(mut self, config: TopicConfiguration) -> Self {
        self.topic_configurations.push(config);
        self
    }

    /// Check if configuration is empty
    pub fn is_empty(&self) -> bool {
        self.webhook_configurations.is_empty()
            && self.queue_configurations.is_empty()
            && self.topic_configurations.is_empty()
    }

    /// Get all configurations that match an event and key
    pub fn get_matching_configs(
        &self,
        event_type: &S3EventType,
        key: &str,
    ) -> Vec<NotificationTarget> {
        let mut targets = Vec::new();

        // Check webhooks
        for webhook in &self.webhook_configurations {
            if self.config_matches(&webhook.events, webhook.filter.as_ref(), event_type, key) {
                targets.push(NotificationTarget::Webhook {
                    id: webhook.id.clone(),
                    url: webhook.url.clone(),
                    headers: webhook.headers.clone(),
                    auth_token: webhook.auth_token.clone(),
                });
            }
        }

        // Check queues
        for queue in &self.queue_configurations {
            if self.config_matches(&queue.events, queue.filter.as_ref(), event_type, key) {
                targets.push(NotificationTarget::Queue {
                    id: queue.id.clone(),
                    arn: queue.queue_arn.clone(),
                });
            }
        }

        // Check topics
        for topic in &self.topic_configurations {
            if self.config_matches(&topic.events, topic.filter.as_ref(), event_type, key) {
                targets.push(NotificationTarget::Topic {
                    id: topic.id.clone(),
                    arn: topic.topic_arn.clone(),
                });
            }
        }

        targets
    }

    fn config_matches(
        &self,
        events: &[S3EventType],
        filter: Option<&NotificationFilter>,
        event_type: &S3EventType,
        key: &str,
    ) -> bool {
        // Check event type
        let event_matches = events
            .iter()
            .any(|e| e.matches(event_type) || event_type.matches(e));
        if !event_matches {
            return false;
        }

        // Check filter
        if let Some(filter) = filter {
            if let Some(ref key_filter) = filter.key {
                if !key_filter.matches(key) {
                    return false;
                }
            }
        }

        true
    }
}

/// Notification target for event dispatch
#[derive(Debug, Clone)]
pub enum NotificationTarget {
    Webhook {
        id: String,
        url: String,
        headers: Option<HashMap<String, String>>,
        auth_token: Option<String>,
    },
    Queue {
        id: String,
        arn: String,
    },
    Topic {
        id: String,
        arn: String,
    },
}

// ============================================================================
// Event Record (S3 Event Message Format)
// ============================================================================

/// S3 Event record (AWS-compatible format)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3EventRecord {
    /// Event version
    pub event_version: String,
    /// Event source
    pub event_source: String,
    /// AWS region (or our region)
    pub aws_region: String,
    /// Event time
    pub event_time: DateTime<Utc>,
    /// Event name
    pub event_name: String,
    /// User identity
    pub user_identity: UserIdentity,
    /// Request parameters
    pub request_parameters: RequestParameters,
    /// Response elements
    pub response_elements: ResponseElements,
    /// S3 info
    pub s3: S3Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIdentity {
    pub principal_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestParameters {
    pub source_ip_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseElements {
    #[serde(rename = "x-amz-request-id")]
    pub x_amz_request_id: String,
    #[serde(rename = "x-amz-id-2")]
    pub x_amz_id_2: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Info {
    pub s3_schema_version: String,
    pub configuration_id: String,
    pub bucket: S3BucketInfo,
    pub object: S3ObjectInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3BucketInfo {
    pub name: String,
    pub owner_identity: UserIdentity,
    pub arn: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3ObjectInfo {
    pub key: String,
    pub size: i64,
    pub e_tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_id: Option<String>,
    pub sequencer: String,
}

/// S3 Event message (contains multiple records)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct S3EventMessage {
    pub records: Vec<S3EventRecord>,
}

impl S3EventRecord {
    /// Create a new event record
    pub fn new(
        event_type: S3EventType,
        bucket: &str,
        key: &str,
        size: i64,
        etag: &str,
        version_id: Option<String>,
        request_id: &str,
        principal_id: &str,
        source_ip: &str,
        config_id: &str,
        region: &str,
    ) -> Self {
        Self {
            event_version: "2.1".to_string(),
            event_source: "hafiz:s3".to_string(),
            aws_region: region.to_string(),
            event_time: Utc::now(),
            event_name: event_type.to_string(),
            user_identity: UserIdentity {
                principal_id: principal_id.to_string(),
            },
            request_parameters: RequestParameters {
                source_ip_address: source_ip.to_string(),
            },
            response_elements: ResponseElements {
                x_amz_request_id: request_id.to_string(),
                x_amz_id_2: format!("{}-extended", request_id),
            },
            s3: S3Info {
                s3_schema_version: "1.0".to_string(),
                configuration_id: config_id.to_string(),
                bucket: S3BucketInfo {
                    name: bucket.to_string(),
                    owner_identity: UserIdentity {
                        principal_id: principal_id.to_string(),
                    },
                    arn: format!("arn:hafiz:s3:::{}", bucket),
                },
                object: S3ObjectInfo {
                    key: key.to_string(),
                    size,
                    e_tag: etag.to_string(),
                    version_id,
                    sequencer: format!("{:016X}", Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_matches() {
        assert!(S3EventType::ObjectCreatedAll.matches(&S3EventType::ObjectCreatedPut));
        assert!(S3EventType::ObjectCreatedAll.matches(&S3EventType::ObjectCreatedCopy));
        assert!(!S3EventType::ObjectCreatedAll.matches(&S3EventType::ObjectRemovedDelete));
        assert!(S3EventType::ObjectCreatedPut.matches(&S3EventType::ObjectCreatedPut));
    }

    #[test]
    fn test_key_filter() {
        let filter = S3KeyFilter::prefix("logs/").with_suffix(".json");

        assert!(filter.matches("logs/app.json"));
        assert!(!filter.matches("data/app.json"));
        assert!(!filter.matches("logs/app.txt"));
    }

    #[test]
    fn test_notification_config_matching() {
        let config = NotificationConfiguration::new().add_webhook(WebhookConfiguration {
            id: "test".to_string(),
            url: "http://example.com/webhook".to_string(),
            events: vec![S3EventType::ObjectCreatedAll],
            filter: Some(NotificationFilter {
                key: Some(S3KeyFilter::prefix("uploads/")),
            }),
            headers: None,
            auth_token: None,
        });

        let targets =
            config.get_matching_configs(&S3EventType::ObjectCreatedPut, "uploads/file.txt");
        assert_eq!(targets.len(), 1);

        let targets = config.get_matching_configs(&S3EventType::ObjectCreatedPut, "other/file.txt");
        assert_eq!(targets.len(), 0);

        let targets =
            config.get_matching_configs(&S3EventType::ObjectRemovedDelete, "uploads/file.txt");
        assert_eq!(targets.len(), 0);
    }
}
