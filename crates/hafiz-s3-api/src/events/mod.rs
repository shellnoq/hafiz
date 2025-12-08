//! Event notification module
//!
//! Handles S3 event notifications including:
//! - Webhook notifications
//! - Queue notifications (SQS-compatible)
//! - Topic notifications (SNS-compatible)

mod dispatcher;

pub use dispatcher::{
    EventDispatcher, EventDispatcherConfig, S3Event, DispatchResult, NotificationConfigStore,
};
