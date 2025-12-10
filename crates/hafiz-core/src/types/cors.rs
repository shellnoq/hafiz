//! CORS (Cross-Origin Resource Sharing) types
//!
//! Implements S3-compatible CORS configuration for enabling
//! cross-origin requests from web browsers.
//!
//! Reference: https://docs.aws.amazon.com/AmazonS3/latest/userguide/cors.html

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

// ============================================================================
// CORS Configuration
// ============================================================================

/// CORS configuration for a bucket
///
/// Defines rules that identify origins and HTTP methods allowed
/// for cross-origin requests to the bucket.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename = "CORSConfiguration")]
pub struct CorsConfiguration {
    /// List of CORS rules (max 100 rules per bucket)
    #[serde(rename = "CORSRule", default)]
    pub cors_rules: Vec<CorsRule>,
}

/// Individual CORS rule
///
/// Each rule specifies allowed origins, methods, headers, and other options.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "CORSRule")]
pub struct CorsRule {
    /// Unique identifier for the rule (optional, for management purposes)
    #[serde(rename = "ID", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Origins allowed to make cross-origin requests
    /// Supports wildcards: "*" allows all origins
    /// Example: ["https://www.example.com", "https://*.example.org"]
    #[serde(rename = "AllowedOrigin")]
    pub allowed_origins: Vec<String>,

    /// HTTP methods allowed for cross-origin requests
    /// Valid values: GET, PUT, POST, DELETE, HEAD
    #[serde(rename = "AllowedMethod")]
    pub allowed_methods: Vec<CorsMethod>,

    /// Headers allowed in the actual request
    /// Supports wildcards: "*" allows all headers
    #[serde(
        rename = "AllowedHeader",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub allowed_headers: Vec<String>,

    /// Headers that browsers can access from the response
    #[serde(
        rename = "ExposeHeader",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub expose_headers: Vec<String>,

    /// Time in seconds that browser can cache preflight response
    /// Default: 0 (no caching)
    #[serde(rename = "MaxAgeSeconds", skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<u32>,
}

/// Allowed HTTP methods for CORS
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorsMethod {
    GET,
    PUT,
    POST,
    DELETE,
    HEAD,
}

impl std::fmt::Display for CorsMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CorsMethod::GET => write!(f, "GET"),
            CorsMethod::PUT => write!(f, "PUT"),
            CorsMethod::POST => write!(f, "POST"),
            CorsMethod::DELETE => write!(f, "DELETE"),
            CorsMethod::HEAD => write!(f, "HEAD"),
        }
    }
}

impl std::str::FromStr for CorsMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(CorsMethod::GET),
            "PUT" => Ok(CorsMethod::PUT),
            "POST" => Ok(CorsMethod::POST),
            "DELETE" => Ok(CorsMethod::DELETE),
            "HEAD" => Ok(CorsMethod::HEAD),
            _ => Err(format!("Invalid CORS method: {}", s)),
        }
    }
}

// ============================================================================
// CORS Validation
// ============================================================================

impl CorsConfiguration {
    /// Maximum number of CORS rules per bucket
    pub const MAX_RULES: usize = 100;

    /// Validate the CORS configuration
    pub fn validate(&self) -> Result<(), CorsValidationError> {
        // Check rule count
        if self.cors_rules.len() > Self::MAX_RULES {
            return Err(CorsValidationError::TooManyRules {
                count: self.cors_rules.len(),
                max: Self::MAX_RULES,
            });
        }

        // Validate each rule
        for (index, rule) in self.cors_rules.iter().enumerate() {
            rule.validate()
                .map_err(|e| CorsValidationError::InvalidRule {
                    index,
                    error: e.to_string(),
                })?;
        }

        Ok(())
    }

    /// Find matching CORS rule for a request
    pub fn find_matching_rule(&self, origin: &str, method: &str) -> Option<&CorsRule> {
        let method = method.parse::<CorsMethod>().ok()?;

        self.cors_rules
            .iter()
            .find(|rule| rule.matches_origin(origin) && rule.allowed_methods.contains(&method))
    }

    /// Check if configuration is empty
    pub fn is_empty(&self) -> bool {
        self.cors_rules.is_empty()
    }
}

impl CorsRule {
    /// Validate a single CORS rule
    pub fn validate(&self) -> Result<(), CorsRuleError> {
        // Must have at least one origin
        if self.allowed_origins.is_empty() {
            return Err(CorsRuleError::NoAllowedOrigins);
        }

        // Must have at least one method
        if self.allowed_methods.is_empty() {
            return Err(CorsRuleError::NoAllowedMethods);
        }

        // Validate origins
        for origin in &self.allowed_origins {
            Self::validate_origin(origin)?;
        }

        // Validate max age (S3 allows 0-86400)
        if let Some(max_age) = self.max_age_seconds {
            if max_age > 86400 {
                return Err(CorsRuleError::InvalidMaxAge(max_age));
            }
        }

        // Validate ID length if present
        if let Some(ref id) = self.id {
            if id.len() > 255 {
                return Err(CorsRuleError::IdTooLong(id.len()));
            }
        }

        Ok(())
    }

    /// Validate an origin pattern
    fn validate_origin(origin: &str) -> Result<(), CorsRuleError> {
        if origin.is_empty() {
            return Err(CorsRuleError::EmptyOrigin);
        }

        // Wildcard is valid
        if origin == "*" {
            return Ok(());
        }

        // Must be a valid URL pattern
        // Allow: https://example.com, http://localhost:3000, https://*.example.com
        if !origin.starts_with("http://") && !origin.starts_with("https://") {
            return Err(CorsRuleError::InvalidOriginScheme(origin.to_string()));
        }

        // Check for invalid wildcard usage (only allowed at start of hostname)
        let without_scheme = origin
            .strip_prefix("http://")
            .or_else(|| origin.strip_prefix("https://"))
            .unwrap_or(origin);

        if without_scheme.contains('*') && !without_scheme.starts_with("*.") {
            return Err(CorsRuleError::InvalidWildcard(origin.to_string()));
        }

        Ok(())
    }

    /// Check if this rule matches an origin
    pub fn matches_origin(&self, origin: &str) -> bool {
        self.allowed_origins
            .iter()
            .any(|pattern| Self::origin_matches_pattern(origin, pattern))
    }

    /// Check if an origin matches a pattern
    fn origin_matches_pattern(origin: &str, pattern: &str) -> bool {
        // Wildcard matches everything
        if pattern == "*" {
            return true;
        }

        // Check for wildcard subdomain pattern
        if let Some(suffix) = pattern.strip_prefix("https://*.") {
            if let Some(origin_host) = origin.strip_prefix("https://") {
                // Must be a subdomain of the suffix
                return origin_host.ends_with(suffix)
                    && origin_host.len() > suffix.len()
                    && origin_host.as_bytes()[origin_host.len() - suffix.len() - 1] == b'.';
            }
        }
        if let Some(suffix) = pattern.strip_prefix("http://*.") {
            if let Some(origin_host) = origin.strip_prefix("http://") {
                return origin_host.ends_with(suffix)
                    && origin_host.len() > suffix.len()
                    && origin_host.as_bytes()[origin_host.len() - suffix.len() - 1] == b'.';
            }
        }

        // Exact match (case-insensitive for hostname)
        origin.eq_ignore_ascii_case(pattern)
    }

    /// Check if a header is allowed
    pub fn is_header_allowed(&self, header: &str) -> bool {
        // Simple headers are always allowed
        let simple_headers = [
            "accept",
            "accept-language",
            "content-language",
            "content-type",
        ];
        if simple_headers.contains(&header.to_lowercase().as_str()) {
            return true;
        }

        // Check against allowed headers
        self.allowed_headers.iter().any(|pattern| {
            if pattern == "*" {
                true
            } else {
                pattern.eq_ignore_ascii_case(header)
            }
        })
    }

    /// Get the Access-Control-Allow-Methods header value
    pub fn allowed_methods_header(&self) -> String {
        self.allowed_methods
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Get the Access-Control-Allow-Headers header value
    pub fn allowed_headers_header(&self) -> String {
        if self.allowed_headers.is_empty() {
            String::new()
        } else {
            self.allowed_headers.join(", ")
        }
    }

    /// Get the Access-Control-Expose-Headers header value
    pub fn expose_headers_header(&self) -> String {
        if self.expose_headers.is_empty() {
            String::new()
        } else {
            self.expose_headers.join(", ")
        }
    }
}

// ============================================================================
// CORS Errors
// ============================================================================

/// CORS configuration validation error
#[derive(Debug, Clone)]
pub enum CorsValidationError {
    /// Too many rules in configuration
    TooManyRules { count: usize, max: usize },
    /// Invalid rule at index
    InvalidRule { index: usize, error: String },
}

impl std::fmt::Display for CorsValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyRules { count, max } => {
                write!(f, "Too many CORS rules: {} (max: {})", count, max)
            }
            Self::InvalidRule { index, error } => {
                write!(f, "Invalid CORS rule at index {}: {}", index, error)
            }
        }
    }
}

impl std::error::Error for CorsValidationError {}

/// CORS rule validation error
#[derive(Debug, Clone)]
pub enum CorsRuleError {
    /// No allowed origins specified
    NoAllowedOrigins,
    /// No allowed methods specified
    NoAllowedMethods,
    /// Empty origin string
    EmptyOrigin,
    /// Invalid origin scheme (must be http or https)
    InvalidOriginScheme(String),
    /// Invalid wildcard usage
    InvalidWildcard(String),
    /// Max age too large
    InvalidMaxAge(u32),
    /// Rule ID too long
    IdTooLong(usize),
}

impl std::fmt::Display for CorsRuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAllowedOrigins => write!(f, "At least one AllowedOrigin is required"),
            Self::NoAllowedMethods => write!(f, "At least one AllowedMethod is required"),
            Self::EmptyOrigin => write!(f, "AllowedOrigin cannot be empty"),
            Self::InvalidOriginScheme(origin) => {
                write!(
                    f,
                    "Invalid origin scheme (must be http or https): {}",
                    origin
                )
            }
            Self::InvalidWildcard(origin) => {
                write!(
                    f,
                    "Wildcard (*) is only allowed at the start of hostname: {}",
                    origin
                )
            }
            Self::InvalidMaxAge(age) => {
                write!(f, "MaxAgeSeconds must be <= 86400: {}", age)
            }
            Self::IdTooLong(len) => {
                write!(f, "Rule ID too long: {} (max: 255)", len)
            }
        }
    }
}

impl std::error::Error for CorsRuleError {}

// ============================================================================
// CORS Response Builder
// ============================================================================

/// Builder for CORS response headers
#[derive(Debug, Clone, Default)]
pub struct CorsResponseHeaders {
    /// Access-Control-Allow-Origin
    pub allow_origin: Option<String>,
    /// Access-Control-Allow-Methods
    pub allow_methods: Option<String>,
    /// Access-Control-Allow-Headers
    pub allow_headers: Option<String>,
    /// Access-Control-Expose-Headers
    pub expose_headers: Option<String>,
    /// Access-Control-Max-Age
    pub max_age: Option<u32>,
    /// Access-Control-Allow-Credentials
    pub allow_credentials: bool,
    /// Vary header value
    pub vary: Option<String>,
}

impl CorsResponseHeaders {
    /// Build CORS headers for a preflight (OPTIONS) request
    pub fn for_preflight(rule: &CorsRule, origin: &str, request_headers: Option<&str>) -> Self {
        let mut headers = Self {
            allow_origin: Some(origin.to_string()),
            allow_methods: Some(rule.allowed_methods_header()),
            max_age: rule.max_age_seconds,
            allow_credentials: true,
            vary: Some(
                "Origin, Access-Control-Request-Method, Access-Control-Request-Headers".to_string(),
            ),
            ..Default::default()
        };

        // Set allowed headers
        if !rule.allowed_headers.is_empty() {
            if rule.allowed_headers.contains(&"*".to_string()) {
                // If wildcard, echo back the requested headers
                headers.allow_headers = request_headers.map(|h| h.to_string());
            } else {
                headers.allow_headers = Some(rule.allowed_headers_header());
            }
        }

        headers
    }

    /// Build CORS headers for an actual request
    pub fn for_actual_request(rule: &CorsRule, origin: &str) -> Self {
        let mut headers = Self {
            allow_origin: Some(origin.to_string()),
            allow_credentials: true,
            vary: Some("Origin".to_string()),
            ..Default::default()
        };

        // Set expose headers
        if !rule.expose_headers.is_empty() {
            headers.expose_headers = Some(rule.expose_headers_header());
        }

        headers
    }

    /// Convert to HTTP header tuples
    pub fn to_header_vec(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        if let Some(ref origin) = self.allow_origin {
            headers.push(("Access-Control-Allow-Origin".to_string(), origin.clone()));
        }
        if let Some(ref methods) = self.allow_methods {
            headers.push(("Access-Control-Allow-Methods".to_string(), methods.clone()));
        }
        if let Some(ref hdrs) = self.allow_headers {
            headers.push(("Access-Control-Allow-Headers".to_string(), hdrs.clone()));
        }
        if let Some(ref expose) = self.expose_headers {
            headers.push(("Access-Control-Expose-Headers".to_string(), expose.clone()));
        }
        if let Some(max_age) = self.max_age {
            headers.push(("Access-Control-Max-Age".to_string(), max_age.to_string()));
        }
        if self.allow_credentials {
            headers.push((
                "Access-Control-Allow-Credentials".to_string(),
                "true".to_string(),
            ));
        }
        if let Some(ref vary) = self.vary {
            headers.push(("Vary".to_string(), vary.clone()));
        }

        headers
    }
}

// ============================================================================
// XML Serialization Helpers
// ============================================================================

impl CorsConfiguration {
    /// Parse from XML
    pub fn from_xml(xml: &str) -> Result<Self, String> {
        quick_xml::de::from_str(xml).map_err(|e| format!("Invalid CORS XML: {}", e))
    }

    /// Serialize to XML
    pub fn to_xml(&self) -> Result<String, String> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        let body = quick_xml::se::to_string(self)
            .map_err(|e| format!("Failed to serialize CORS: {}", e))?;
        xml.push_str(&body);

        Ok(xml)
    }
}

// ============================================================================
// Default Implementations
// ============================================================================

impl Default for CorsRule {
    fn default() -> Self {
        Self {
            id: None,
            allowed_origins: Vec::new(),
            allowed_methods: Vec::new(),
            allowed_headers: Vec::new(),
            expose_headers: Vec::new(),
            max_age_seconds: None,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_origin_matching_exact() {
        let rule = CorsRule {
            allowed_origins: vec!["https://example.com".to_string()],
            allowed_methods: vec![CorsMethod::GET],
            ..Default::default()
        };

        assert!(rule.matches_origin("https://example.com"));
        assert!(!rule.matches_origin("https://other.com"));
        assert!(!rule.matches_origin("http://example.com")); // Different scheme
    }

    #[test]
    fn test_origin_matching_wildcard() {
        let rule = CorsRule {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![CorsMethod::GET],
            ..Default::default()
        };

        assert!(rule.matches_origin("https://example.com"));
        assert!(rule.matches_origin("http://localhost:3000"));
        assert!(rule.matches_origin("https://any.domain.com"));
    }

    #[test]
    fn test_origin_matching_subdomain_wildcard() {
        let rule = CorsRule {
            allowed_origins: vec!["https://*.example.com".to_string()],
            allowed_methods: vec![CorsMethod::GET],
            ..Default::default()
        };

        assert!(rule.matches_origin("https://www.example.com"));
        assert!(rule.matches_origin("https://api.example.com"));
        assert!(rule.matches_origin("https://sub.domain.example.com"));
        assert!(!rule.matches_origin("https://example.com")); // Not a subdomain
        assert!(!rule.matches_origin("http://www.example.com")); // Wrong scheme
    }

    #[test]
    fn test_rule_validation() {
        let valid_rule = CorsRule {
            allowed_origins: vec!["https://example.com".to_string()],
            allowed_methods: vec![CorsMethod::GET, CorsMethod::POST],
            allowed_headers: vec!["Content-Type".to_string()],
            max_age_seconds: Some(3600),
            ..Default::default()
        };
        assert!(valid_rule.validate().is_ok());

        let no_origins = CorsRule {
            allowed_origins: vec![],
            allowed_methods: vec![CorsMethod::GET],
            ..Default::default()
        };
        assert!(matches!(
            no_origins.validate(),
            Err(CorsRuleError::NoAllowedOrigins)
        ));

        let no_methods = CorsRule {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![],
            ..Default::default()
        };
        assert!(matches!(
            no_methods.validate(),
            Err(CorsRuleError::NoAllowedMethods)
        ));
    }

    #[test]
    fn test_cors_method_parse() {
        assert_eq!("GET".parse::<CorsMethod>().unwrap(), CorsMethod::GET);
        assert_eq!("put".parse::<CorsMethod>().unwrap(), CorsMethod::PUT);
        assert!("PATCH".parse::<CorsMethod>().is_err());
    }

    #[test]
    fn test_header_matching() {
        let rule = CorsRule {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![CorsMethod::GET],
            allowed_headers: vec!["X-Custom-Header".to_string(), "Authorization".to_string()],
            ..Default::default()
        };

        // Simple headers always allowed
        assert!(rule.is_header_allowed("Content-Type"));
        assert!(rule.is_header_allowed("Accept"));

        // Custom headers
        assert!(rule.is_header_allowed("X-Custom-Header"));
        assert!(rule.is_header_allowed("Authorization"));
        assert!(!rule.is_header_allowed("X-Other-Header"));
    }

    #[test]
    fn test_wildcard_header_matching() {
        let rule = CorsRule {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![CorsMethod::GET],
            allowed_headers: vec!["*".to_string()],
            ..Default::default()
        };

        assert!(rule.is_header_allowed("X-Any-Header"));
        assert!(rule.is_header_allowed("Authorization"));
        assert!(rule.is_header_allowed("X-Custom-Whatever"));
    }

    #[test]
    fn test_configuration_validation() {
        let config = CorsConfiguration {
            cors_rules: vec![CorsRule {
                allowed_origins: vec!["https://example.com".to_string()],
                allowed_methods: vec![CorsMethod::GET],
                ..Default::default()
            }],
        };
        assert!(config.validate().is_ok());

        // Too many rules
        let too_many = CorsConfiguration {
            cors_rules: (0..101)
                .map(|i| CorsRule {
                    id: Some(format!("rule-{}", i)),
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec![CorsMethod::GET],
                    ..Default::default()
                })
                .collect(),
        };
        assert!(matches!(
            too_many.validate(),
            Err(CorsValidationError::TooManyRules { .. })
        ));
    }

    #[test]
    fn test_find_matching_rule() {
        let config = CorsConfiguration {
            cors_rules: vec![
                CorsRule {
                    id: Some("rule1".to_string()),
                    allowed_origins: vec!["https://app.example.com".to_string()],
                    allowed_methods: vec![CorsMethod::GET, CorsMethod::HEAD],
                    ..Default::default()
                },
                CorsRule {
                    id: Some("rule2".to_string()),
                    allowed_origins: vec!["https://*.example.org".to_string()],
                    allowed_methods: vec![CorsMethod::GET, CorsMethod::PUT, CorsMethod::POST],
                    ..Default::default()
                },
            ],
        };

        // Match first rule
        let rule = config.find_matching_rule("https://app.example.com", "GET");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().id, Some("rule1".to_string()));

        // Match second rule (subdomain wildcard)
        let rule = config.find_matching_rule("https://api.example.org", "PUT");
        assert!(rule.is_some());
        assert_eq!(rule.unwrap().id, Some("rule2".to_string()));

        // No match (wrong origin)
        let rule = config.find_matching_rule("https://other.com", "GET");
        assert!(rule.is_none());

        // No match (wrong method)
        let rule = config.find_matching_rule("https://app.example.com", "DELETE");
        assert!(rule.is_none());
    }
}
