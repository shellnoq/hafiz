//! IAM Policy types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// IAM Policy Document (AWS-compatible)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PolicyDocument {
    /// Policy version (should be "2012-10-17")
    #[serde(default = "default_version")]
    pub version: String,

    /// Policy ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Policy statements
    pub statement: Vec<Statement>,
}

fn default_version() -> String {
    "2012-10-17".to_string()
}

impl PolicyDocument {
    /// Create a new empty policy
    pub fn new() -> Self {
        Self {
            version: default_version(),
            id: None,
            statement: Vec::new(),
        }
    }

    /// Add a statement
    pub fn add_statement(mut self, statement: Statement) -> Self {
        self.statement.push(statement);
        self
    }

    /// Evaluate policy against a request
    pub fn evaluate(&self, request: &PolicyRequest) -> PolicyEffect {
        let mut explicit_allow = false;

        for statement in &self.statement {
            match statement.evaluate(request) {
                StatementResult::ExplicitDeny => return PolicyEffect::Deny,
                StatementResult::Allow => explicit_allow = true,
                StatementResult::NoMatch => continue,
            }
        }

        if explicit_allow {
            PolicyEffect::Allow
        } else {
            PolicyEffect::Deny // Default deny
        }
    }
}

impl Default for PolicyDocument {
    fn default() -> Self {
        Self::new()
    }
}

/// Policy statement
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Statement {
    /// Statement ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sid: Option<String>,

    /// Effect (Allow or Deny)
    pub effect: Effect,

    /// Principal (who this applies to)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal: Option<Principal>,

    /// Not Principal
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_principal: Option<Principal>,

    /// Actions
    #[serde(default)]
    pub action: StringOrArray,

    /// Not Actions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_action: Option<StringOrArray>,

    /// Resources
    #[serde(default)]
    pub resource: StringOrArray,

    /// Not Resources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_resource: Option<StringOrArray>,

    /// Conditions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<HashMap<String, HashMap<String, StringOrArray>>>,
}

impl Statement {
    /// Create a new allow statement
    pub fn allow() -> Self {
        Self {
            sid: None,
            effect: Effect::Allow,
            principal: None,
            not_principal: None,
            action: StringOrArray::Array(vec![]),
            not_action: None,
            resource: StringOrArray::Array(vec![]),
            not_resource: None,
            condition: None,
        }
    }

    /// Create a new deny statement
    pub fn deny() -> Self {
        Self {
            sid: None,
            effect: Effect::Deny,
            principal: None,
            not_principal: None,
            action: StringOrArray::Array(vec![]),
            not_action: None,
            resource: StringOrArray::Array(vec![]),
            not_resource: None,
            condition: None,
        }
    }

    /// Set statement ID
    pub fn with_sid(mut self, sid: impl Into<String>) -> Self {
        self.sid = Some(sid.into());
        self
    }

    /// Set actions
    pub fn with_actions(mut self, actions: Vec<String>) -> Self {
        self.action = StringOrArray::Array(actions);
        self
    }

    /// Set resources
    pub fn with_resources(mut self, resources: Vec<String>) -> Self {
        self.resource = StringOrArray::Array(resources);
        self
    }

    /// Evaluate statement against a request
    fn evaluate(&self, request: &PolicyRequest) -> StatementResult {
        // Check action match
        let action_match = self.matches_action(&request.action);
        if !action_match {
            return StatementResult::NoMatch;
        }

        // Check resource match
        let resource_match = self.matches_resource(&request.resource);
        if !resource_match {
            return StatementResult::NoMatch;
        }

        // Check principal match (if specified)
        if let Some(ref principal) = self.principal {
            if !principal.matches(&request.principal) {
                return StatementResult::NoMatch;
            }
        }

        // Check conditions (simplified)
        if self.condition.is_some() {
            // TODO: Implement condition evaluation
        }

        match self.effect {
            Effect::Allow => StatementResult::Allow,
            Effect::Deny => StatementResult::ExplicitDeny,
        }
    }

    fn matches_action(&self, action: &str) -> bool {
        let actions = self.action.as_slice();
        
        for pattern in actions {
            if pattern == "*" || pattern == "s3:*" {
                return true;
            }
            if matches_wildcard(pattern, action) {
                return true;
            }
        }
        
        false
    }

    fn matches_resource(&self, resource: &str) -> bool {
        let resources = self.resource.as_slice();
        
        if resources.is_empty() {
            return true; // No resource restriction
        }

        for pattern in resources {
            if pattern == "*" {
                return true;
            }
            if matches_wildcard(pattern, resource) {
                return true;
            }
        }
        
        false
    }
}

/// Effect type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effect {
    Allow,
    Deny,
}

/// Principal specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Principal {
    /// Wildcard (anyone)
    Wildcard(String),
    /// Specific principals
    Specific(HashMap<String, StringOrArray>),
}

impl Principal {
    /// Match against a principal identifier
    pub fn matches(&self, principal: &str) -> bool {
        match self {
            Principal::Wildcard(s) if s == "*" => true,
            Principal::Wildcard(_) => false,
            Principal::Specific(map) => {
                for (_key, values) in map {
                    for value in values.as_slice() {
                        if value == "*" || value == principal {
                            return true;
                        }
                        if matches_wildcard(value, principal) {
                            return true;
                        }
                    }
                }
                false
            }
        }
    }
}

/// String or array of strings (common in IAM policies)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrArray {
    String(String),
    Array(Vec<String>),
}

impl StringOrArray {
    pub fn as_slice(&self) -> &[String] {
        match self {
            StringOrArray::String(s) => std::slice::from_ref(s),
            StringOrArray::Array(arr) => arr.as_slice(),
        }
    }
}

impl Default for StringOrArray {
    fn default() -> Self {
        StringOrArray::Array(vec![])
    }
}

/// Policy evaluation request
#[derive(Debug, Clone)]
pub struct PolicyRequest {
    /// Action being performed (e.g., "s3:GetObject")
    pub action: String,
    /// Resource ARN
    pub resource: String,
    /// Principal identifier
    pub principal: String,
    /// Request context (for conditions)
    pub context: HashMap<String, String>,
}

impl PolicyRequest {
    pub fn new(action: impl Into<String>, resource: impl Into<String>, principal: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            resource: resource.into(),
            principal: principal.into(),
            context: HashMap::new(),
        }
    }

    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }
}

/// Statement evaluation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatementResult {
    Allow,
    ExplicitDeny,
    NoMatch,
}

/// Final policy effect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

/// S3 Actions
pub mod actions {
    // Bucket operations
    pub const CREATE_BUCKET: &str = "s3:CreateBucket";
    pub const DELETE_BUCKET: &str = "s3:DeleteBucket";
    pub const LIST_BUCKET: &str = "s3:ListBucket";
    pub const LIST_ALL_MY_BUCKETS: &str = "s3:ListAllMyBuckets";
    pub const GET_BUCKET_LOCATION: &str = "s3:GetBucketLocation";
    pub const GET_BUCKET_POLICY: &str = "s3:GetBucketPolicy";
    pub const PUT_BUCKET_POLICY: &str = "s3:PutBucketPolicy";
    pub const DELETE_BUCKET_POLICY: &str = "s3:DeleteBucketPolicy";
    pub const GET_BUCKET_ACL: &str = "s3:GetBucketAcl";
    pub const PUT_BUCKET_ACL: &str = "s3:PutBucketAcl";

    // Object operations
    pub const GET_OBJECT: &str = "s3:GetObject";
    pub const PUT_OBJECT: &str = "s3:PutObject";
    pub const DELETE_OBJECT: &str = "s3:DeleteObject";
    pub const GET_OBJECT_ACL: &str = "s3:GetObjectAcl";
    pub const PUT_OBJECT_ACL: &str = "s3:PutObjectAcl";
    pub const LIST_MULTIPART_UPLOAD_PARTS: &str = "s3:ListMultipartUploadParts";
    pub const ABORT_MULTIPART_UPLOAD: &str = "s3:AbortMultipartUpload";
}

/// Simple wildcard matching (supports * and ?)
fn matches_wildcard(pattern: &str, text: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // * matches zero or more characters
                if pattern_chars.peek().is_none() {
                    return true; // Trailing * matches everything
                }
                // Try to match remaining pattern at each position
                let remaining_pattern: String = pattern_chars.collect();
                let mut remaining_text = String::new();
                while text_chars.peek().is_some() {
                    remaining_text.push(text_chars.next().unwrap());
                    let test_text: String = text_chars.clone().collect();
                    if matches_wildcard(&remaining_pattern, &test_text) {
                        return true;
                    }
                }
                return matches_wildcard(&remaining_pattern, "");
            }
            '?' => {
                // ? matches exactly one character
                if text_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                // Literal character match
                match text_chars.next() {
                    Some(t) if t == c => continue,
                    _ => return false,
                }
            }
        }
    }

    // Both should be exhausted
    text_chars.peek().is_none()
}

/// Create an ARN for a bucket
pub fn bucket_arn(bucket: &str) -> String {
    format!("arn:hafiz:s3:::{}", bucket)
}

/// Create an ARN for an object
pub fn object_arn(bucket: &str, key: &str) -> String {
    format!("arn:hafiz:s3:::{}/{}", bucket, key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_matching() {
        assert!(matches_wildcard("*", "anything"));
        assert!(matches_wildcard("s3:*", "s3:GetObject"));
        assert!(matches_wildcard("s3:Get*", "s3:GetObject"));
        assert!(matches_wildcard("s3:GetObject", "s3:GetObject"));
        assert!(!matches_wildcard("s3:Get*", "s3:PutObject"));
        assert!(matches_wildcard("arn:*:s3:::bucket/*", "arn:hafiz:s3:::bucket/key"));
    }

    #[test]
    fn test_policy_evaluation() {
        let policy = PolicyDocument::new()
            .add_statement(
                Statement::allow()
                    .with_actions(vec!["s3:GetObject".to_string()])
                    .with_resources(vec!["arn:hafiz:s3:::my-bucket/*".to_string()])
            );

        let request = PolicyRequest::new(
            "s3:GetObject",
            "arn:hafiz:s3:::my-bucket/test.txt",
            "user123"
        );

        assert_eq!(policy.evaluate(&request), PolicyEffect::Allow);
    }

    #[test]
    fn test_explicit_deny() {
        let policy = PolicyDocument::new()
            .add_statement(
                Statement::allow()
                    .with_actions(vec!["s3:*".to_string()])
                    .with_resources(vec!["*".to_string()])
            )
            .add_statement(
                Statement::deny()
                    .with_actions(vec!["s3:DeleteObject".to_string()])
                    .with_resources(vec!["*".to_string()])
            );

        let get_request = PolicyRequest::new("s3:GetObject", "arn:hafiz:s3:::bucket/key", "user");
        let delete_request = PolicyRequest::new("s3:DeleteObject", "arn:hafiz:s3:::bucket/key", "user");

        assert_eq!(policy.evaluate(&get_request), PolicyEffect::Allow);
        assert_eq!(policy.evaluate(&delete_request), PolicyEffect::Deny);
    }

    #[test]
    fn test_bucket_arn() {
        assert_eq!(bucket_arn("my-bucket"), "arn:hafiz:s3:::my-bucket");
    }

    #[test]
    fn test_object_arn() {
        assert_eq!(object_arn("my-bucket", "path/to/key"), "arn:hafiz:s3:::my-bucket/path/to/key");
    }
}
