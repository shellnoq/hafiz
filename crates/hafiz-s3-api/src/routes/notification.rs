//! Bucket Notification Configuration handlers
//!
//! S3-compatible notification configuration management.

use axum::{
    body::Body,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use hafiz_core::{
    types::NotificationConfiguration,
    utils::generate_request_id,
    Error,
};
use tracing::{debug, error, info};

use crate::server::AppState;

// ============================================================================
// Response Helpers
// ============================================================================

fn error_response(err: Error, request_id: &str) -> Response {
    let status = StatusCode::from_u16(err.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let s3_error = hafiz_core::error::S3Error::from(err).with_request_id(request_id);

    Response::builder()
        .status(status)
        .header("Content-Type", "application/xml")
        .header("x-amz-request-id", request_id)
        .body(Body::from(s3_error.to_xml()))
        .unwrap()
}

fn success_response_xml(status: StatusCode, body: String, request_id: &str) -> Response {
    Response::builder()
        .status(status)
        .header("Content-Type", "application/xml")
        .header("x-amz-request-id", request_id)
        .body(Body::from(body))
        .unwrap()
}

fn no_content_response(request_id: &str) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header("x-amz-request-id", request_id)
        .body(Body::empty())
        .unwrap()
}

// ============================================================================
// Notification Configuration Handlers
// ============================================================================

/// GET /{bucket}?notification - Get bucket notification configuration
pub async fn get_bucket_notification(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("GetBucketNotificationConfiguration bucket={} request_id={}", bucket, request_id);

    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return error_response(Error::NoSuchBucketNamed(bucket), &request_id);
        }
        Err(e) => {
            error!("Error checking bucket: {}", e);
            return error_response(e, &request_id);
        }
    }

    // Get notification configuration from metadata
    match state.metadata.get_bucket_notification(&bucket).await {
        Ok(Some(config_json)) => {
            // Convert JSON to XML response
            match serde_json::from_str::<NotificationConfiguration>(&config_json) {
                Ok(config) => {
                    let xml = notification_config_to_xml(&config);
                    success_response_xml(StatusCode::OK, xml, &request_id)
                }
                Err(e) => {
                    error!("Failed to parse notification config: {}", e);
                    // Return empty config on parse error
                    let xml = notification_config_to_xml(&NotificationConfiguration::default());
                    success_response_xml(StatusCode::OK, xml, &request_id)
                }
            }
        }
        Ok(None) => {
            // Return empty configuration
            let xml = notification_config_to_xml(&NotificationConfiguration::default());
            success_response_xml(StatusCode::OK, xml, &request_id)
        }
        Err(e) => {
            error!("Error getting notification config: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// PUT /{bucket}?notification - Put bucket notification configuration
pub async fn put_bucket_notification(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    body: Bytes,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("PutBucketNotificationConfiguration bucket={} request_id={}", bucket, request_id);

    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return error_response(Error::NoSuchBucketNamed(bucket), &request_id);
        }
        Err(e) => {
            error!("Error checking bucket: {}", e);
            return error_response(e, &request_id);
        }
    }

    // Parse notification configuration
    let body_str = match String::from_utf8(body.to_vec()) {
        Ok(s) => s,
        Err(_) => {
            return error_response(
                Error::MalformedXML("Invalid UTF-8 in notification configuration".into()),
                &request_id,
            );
        }
    };

    // Parse XML to NotificationConfiguration
    let config = match parse_notification_config_xml(&body_str) {
        Ok(c) => c,
        Err(e) => {
            return error_response(
                Error::MalformedXML(format!("Invalid notification configuration: {}", e)),
                &request_id,
            );
        }
    };

    // Validate configuration
    if let Err(e) = validate_notification_config(&config) {
        return error_response(Error::InvalidArgument(e), &request_id);
    }

    // Convert to JSON for storage
    let config_json = match serde_json::to_string(&config) {
        Ok(j) => j,
        Err(e) => {
            error!("Failed to serialize notification config: {}", e);
            return error_response(Error::InternalError(e.to_string()), &request_id);
        }
    };

    // Store notification configuration
    match state.metadata.put_bucket_notification(&bucket, &config_json).await {
        Ok(_) => {
            info!("Bucket notification configuration set for {}", bucket);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error setting notification config: {}", e);
            error_response(e, &request_id)
        }
    }
}

/// DELETE /{bucket}?notification - Delete bucket notification configuration
pub async fn delete_bucket_notification(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
) -> impl IntoResponse {
    let request_id = generate_request_id();
    debug!("DeleteBucketNotificationConfiguration bucket={} request_id={}", bucket, request_id);

    // Check if bucket exists
    match state.metadata.get_bucket(&bucket).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return error_response(Error::NoSuchBucketNamed(bucket), &request_id);
        }
        Err(e) => {
            error!("Error checking bucket: {}", e);
            return error_response(e, &request_id);
        }
    }

    // Delete notification configuration (by setting empty config)
    let empty_config = serde_json::to_string(&NotificationConfiguration::default()).unwrap();
    match state.metadata.put_bucket_notification(&bucket, &empty_config).await {
        Ok(_) => {
            info!("Bucket notification configuration deleted for {}", bucket);
            no_content_response(&request_id)
        }
        Err(e) => {
            error!("Error deleting notification config: {}", e);
            error_response(e, &request_id)
        }
    }
}

// ============================================================================
// XML Helpers
// ============================================================================

/// Convert NotificationConfiguration to XML
fn notification_config_to_xml(config: &NotificationConfiguration) -> String {
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(r#"<NotificationConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);

    // Webhook configurations (custom extension)
    for webhook in &config.webhook_configurations {
        xml.push_str("<WebhookConfiguration>");
        xml.push_str(&format!("<Id>{}</Id>", xml_escape(&webhook.id)));
        xml.push_str(&format!("<Url>{}</Url>", xml_escape(&webhook.url)));

        for event in &webhook.events {
            xml.push_str(&format!("<Event>{}</Event>", event));
        }

        if let Some(ref filter) = webhook.filter {
            xml.push_str("<Filter>");
            if let Some(ref key_filter) = filter.key {
                xml.push_str("<S3Key>");
                for rule in &key_filter.filter_rules {
                    xml.push_str("<FilterRule>");
                    xml.push_str(&format!("<Name>{}</Name>", xml_escape(&rule.name)));
                    xml.push_str(&format!("<Value>{}</Value>", xml_escape(&rule.value)));
                    xml.push_str("</FilterRule>");
                }
                xml.push_str("</S3Key>");
            }
            xml.push_str("</Filter>");
        }

        xml.push_str("</WebhookConfiguration>");
    }

    // Queue configurations
    for queue in &config.queue_configurations {
        xml.push_str("<QueueConfiguration>");
        xml.push_str(&format!("<Id>{}</Id>", xml_escape(&queue.id)));
        xml.push_str(&format!("<Queue>{}</Queue>", xml_escape(&queue.queue_arn)));

        for event in &queue.events {
            xml.push_str(&format!("<Event>{}</Event>", event));
        }

        if let Some(ref filter) = queue.filter {
            xml.push_str("<Filter>");
            if let Some(ref key_filter) = filter.key {
                xml.push_str("<S3Key>");
                for rule in &key_filter.filter_rules {
                    xml.push_str("<FilterRule>");
                    xml.push_str(&format!("<Name>{}</Name>", xml_escape(&rule.name)));
                    xml.push_str(&format!("<Value>{}</Value>", xml_escape(&rule.value)));
                    xml.push_str("</FilterRule>");
                }
                xml.push_str("</S3Key>");
            }
            xml.push_str("</Filter>");
        }

        xml.push_str("</QueueConfiguration>");
    }

    // Topic configurations
    for topic in &config.topic_configurations {
        xml.push_str("<TopicConfiguration>");
        xml.push_str(&format!("<Id>{}</Id>", xml_escape(&topic.id)));
        xml.push_str(&format!("<Topic>{}</Topic>", xml_escape(&topic.topic_arn)));

        for event in &topic.events {
            xml.push_str(&format!("<Event>{}</Event>", event));
        }

        if let Some(ref filter) = topic.filter {
            xml.push_str("<Filter>");
            if let Some(ref key_filter) = filter.key {
                xml.push_str("<S3Key>");
                for rule in &key_filter.filter_rules {
                    xml.push_str("<FilterRule>");
                    xml.push_str(&format!("<Name>{}</Name>", xml_escape(&rule.name)));
                    xml.push_str(&format!("<Value>{}</Value>", xml_escape(&rule.value)));
                    xml.push_str("</FilterRule>");
                }
                xml.push_str("</S3Key>");
            }
            xml.push_str("</Filter>");
        }

        xml.push_str("</TopicConfiguration>");
    }

    xml.push_str("</NotificationConfiguration>");
    xml
}

/// Parse NotificationConfiguration from XML (simplified parser)
fn parse_notification_config_xml(xml: &str) -> Result<NotificationConfiguration, String> {
    use hafiz_core::types::{
        FilterRule, NotificationFilter, QueueConfiguration, S3EventType, S3KeyFilter,
        TopicConfiguration, WebhookConfiguration,
    };

    let mut config = NotificationConfiguration::default();

    // Simple regex-based parsing for demonstration
    // In production, use a proper XML parser

    // Parse WebhookConfigurations
    let webhook_re = regex::Regex::new(r"<WebhookConfiguration>(.*?)</WebhookConfiguration>")
        .map_err(|e| e.to_string())?;

    for cap in webhook_re.captures_iter(xml) {
        let content = &cap[1];

        let id = extract_xml_value(content, "Id").unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let url = extract_xml_value(content, "Url").ok_or("Missing Url in WebhookConfiguration")?;
        let events = extract_events(content)?;
        let filter = extract_filter(content);

        config.webhook_configurations.push(WebhookConfiguration {
            id,
            url,
            events,
            filter,
            headers: None,
            auth_token: extract_xml_value(content, "AuthToken"),
        });
    }

    // Parse QueueConfigurations
    let queue_re = regex::Regex::new(r"<QueueConfiguration>(.*?)</QueueConfiguration>")
        .map_err(|e| e.to_string())?;

    for cap in queue_re.captures_iter(xml) {
        let content = &cap[1];

        let id = extract_xml_value(content, "Id").unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let queue_arn = extract_xml_value(content, "Queue").ok_or("Missing Queue in QueueConfiguration")?;
        let events = extract_events(content)?;
        let filter = extract_filter(content);

        config.queue_configurations.push(QueueConfiguration {
            id,
            queue_arn,
            events,
            filter,
        });
    }

    // Parse TopicConfigurations
    let topic_re = regex::Regex::new(r"<TopicConfiguration>(.*?)</TopicConfiguration>")
        .map_err(|e| e.to_string())?;

    for cap in topic_re.captures_iter(xml) {
        let content = &cap[1];

        let id = extract_xml_value(content, "Id").unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let topic_arn = extract_xml_value(content, "Topic").ok_or("Missing Topic in TopicConfiguration")?;
        let events = extract_events(content)?;
        let filter = extract_filter(content);

        config.topic_configurations.push(TopicConfiguration {
            id,
            topic_arn,
            events,
            filter,
        });
    }

    Ok(config)
}

fn extract_xml_value(content: &str, tag: &str) -> Option<String> {
    let re = regex::Regex::new(&format!(r"<{}>(.*?)</{}>", tag, tag)).ok()?;
    re.captures(content).map(|cap| cap[1].to_string())
}

fn extract_events(content: &str) -> Result<Vec<hafiz_core::types::S3EventType>, String> {
    use hafiz_core::types::S3EventType;

    let re = regex::Regex::new(r"<Event>(.*?)</Event>").map_err(|e| e.to_string())?;
    let mut events = Vec::new();

    for cap in re.captures_iter(content) {
        let event_str = &cap[1];
        let event = match event_str {
            "s3:ObjectCreated:*" => S3EventType::ObjectCreatedAll,
            "s3:ObjectCreated:Put" => S3EventType::ObjectCreatedPut,
            "s3:ObjectCreated:Post" => S3EventType::ObjectCreatedPost,
            "s3:ObjectCreated:Copy" => S3EventType::ObjectCreatedCopy,
            "s3:ObjectCreated:CompleteMultipartUpload" => S3EventType::ObjectCreatedCompleteMultipartUpload,
            "s3:ObjectRemoved:*" => S3EventType::ObjectRemovedAll,
            "s3:ObjectRemoved:Delete" => S3EventType::ObjectRemovedDelete,
            "s3:ObjectRemoved:DeleteMarkerCreated" => S3EventType::ObjectRemovedDeleteMarkerCreated,
            "s3:ObjectTagging:*" => S3EventType::ObjectTaggingAll,
            "s3:ObjectTagging:Put" => S3EventType::ObjectTaggingPut,
            "s3:ObjectTagging:Delete" => S3EventType::ObjectTaggingDelete,
            "s3:ObjectAcl:Put" => S3EventType::ObjectAclPut,
            _ => return Err(format!("Unknown event type: {}", event_str)),
        };
        events.push(event);
    }

    if events.is_empty() {
        return Err("No events specified".to_string());
    }

    Ok(events)
}

fn extract_filter(content: &str) -> Option<hafiz_core::types::NotificationFilter> {
    use hafiz_core::types::{FilterRule, NotificationFilter, S3KeyFilter};

    let filter_re = regex::Regex::new(r"<Filter>(.*?)</Filter>").ok()?;
    let filter_content = filter_re.captures(content)?[1].to_string();

    let s3key_re = regex::Regex::new(r"<S3Key>(.*?)</S3Key>").ok()?;
    let s3key_content = s3key_re.captures(&filter_content)?[1].to_string();

    let rule_re = regex::Regex::new(r"<FilterRule>.*?<Name>(.*?)</Name>.*?<Value>(.*?)</Value>.*?</FilterRule>").ok()?;

    let mut filter_rules = Vec::new();
    for cap in rule_re.captures_iter(&s3key_content) {
        filter_rules.push(FilterRule {
            name: cap[1].to_string(),
            value: cap[2].to_string(),
        });
    }

    if filter_rules.is_empty() {
        return None;
    }

    Some(NotificationFilter {
        key: Some(S3KeyFilter { filter_rules }),
    })
}

fn validate_notification_config(config: &NotificationConfiguration) -> Result<(), String> {
    // Validate webhook URLs
    for webhook in &config.webhook_configurations {
        if !webhook.url.starts_with("http://") && !webhook.url.starts_with("https://") {
            return Err(format!("Invalid webhook URL: {}", webhook.url));
        }

        if webhook.events.is_empty() {
            return Err(format!("Webhook {} has no events configured", webhook.id));
        }
    }

    // Validate queue ARNs
    for queue in &config.queue_configurations {
        if !queue.queue_arn.starts_with("arn:") {
            return Err(format!("Invalid queue ARN: {}", queue.queue_arn));
        }

        if queue.events.is_empty() {
            return Err(format!("Queue {} has no events configured", queue.id));
        }
    }

    // Validate topic ARNs
    for topic in &config.topic_configurations {
        if !topic.topic_arn.starts_with("arn:") {
            return Err(format!("Invalid topic ARN: {}", topic.topic_arn));
        }

        if topic.events.is_empty() {
            return Err(format!("Topic {} has no events configured", topic.id));
        }
    }

    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
