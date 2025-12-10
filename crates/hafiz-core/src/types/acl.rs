//! Access Control List (ACL) types
//!
//! S3-compatible ACL implementation supporting:
//! - Canned ACLs (private, public-read, etc.)
//! - Grant-based ACLs
//! - XML serialization for S3 compatibility

use serde::{Deserialize, Serialize};
use std::str::FromStr;

// ============================================================================
// Canned ACLs
// ============================================================================

/// Canned (predefined) ACL types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CannedAcl {
    /// Owner gets FULL_CONTROL. No one else has access rights.
    Private,
    /// Owner gets FULL_CONTROL. Everyone else gets READ access.
    PublicRead,
    /// Owner gets FULL_CONTROL. Everyone else gets READ and WRITE access.
    PublicReadWrite,
    /// Owner gets FULL_CONTROL. Authenticated users get READ access.
    AuthenticatedRead,
    /// Object owner gets FULL_CONTROL. Bucket owner gets READ access.
    BucketOwnerRead,
    /// Both object owner and bucket owner get FULL_CONTROL.
    BucketOwnerFullControl,
}

impl Default for CannedAcl {
    fn default() -> Self {
        CannedAcl::Private
    }
}

impl FromStr for CannedAcl {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "private" => Ok(CannedAcl::Private),
            "public-read" => Ok(CannedAcl::PublicRead),
            "public-read-write" => Ok(CannedAcl::PublicReadWrite),
            "authenticated-read" => Ok(CannedAcl::AuthenticatedRead),
            "bucket-owner-read" => Ok(CannedAcl::BucketOwnerRead),
            "bucket-owner-full-control" => Ok(CannedAcl::BucketOwnerFullControl),
            _ => Err(format!("Invalid canned ACL: {}", s)),
        }
    }
}

impl std::fmt::Display for CannedAcl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CannedAcl::Private => write!(f, "private"),
            CannedAcl::PublicRead => write!(f, "public-read"),
            CannedAcl::PublicReadWrite => write!(f, "public-read-write"),
            CannedAcl::AuthenticatedRead => write!(f, "authenticated-read"),
            CannedAcl::BucketOwnerRead => write!(f, "bucket-owner-read"),
            CannedAcl::BucketOwnerFullControl => write!(f, "bucket-owner-full-control"),
        }
    }
}

// ============================================================================
// ACL Permissions
// ============================================================================

/// ACL Permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    /// Allows grantee to list objects in bucket / read object data
    Read,
    /// Allows grantee to create/overwrite objects in bucket
    Write,
    /// Allows grantee to read the ACL
    ReadAcp,
    /// Allows grantee to write the ACL
    WriteAcp,
    /// Allows all of the above
    FullControl,
}

impl FromStr for Permission {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "READ" => Ok(Permission::Read),
            "WRITE" => Ok(Permission::Write),
            "READ_ACP" => Ok(Permission::ReadAcp),
            "WRITE_ACP" => Ok(Permission::WriteAcp),
            "FULL_CONTROL" => Ok(Permission::FullControl),
            _ => Err(format!("Invalid permission: {}", s)),
        }
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Permission::Read => write!(f, "READ"),
            Permission::Write => write!(f, "WRITE"),
            Permission::ReadAcp => write!(f, "READ_ACP"),
            Permission::WriteAcp => write!(f, "WRITE_ACP"),
            Permission::FullControl => write!(f, "FULL_CONTROL"),
        }
    }
}

impl Permission {
    /// Check if this permission includes another permission
    pub fn includes(&self, other: &Permission) -> bool {
        match self {
            Permission::FullControl => true,
            Permission::Read => matches!(other, Permission::Read),
            Permission::Write => matches!(other, Permission::Write),
            Permission::ReadAcp => matches!(other, Permission::ReadAcp),
            Permission::WriteAcp => matches!(other, Permission::WriteAcp),
        }
    }
}

// ============================================================================
// Grantee Types
// ============================================================================

/// Grantee type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "PascalCase")]
pub enum Grantee {
    /// Canonical user (by ID)
    CanonicalUser {
        id: String,
        display_name: Option<String>,
    },
    /// AWS Account (by email)
    AmazonCustomerByEmail { email_address: String },
    /// Predefined group
    Group { uri: String },
}

impl Grantee {
    /// Create a canonical user grantee
    pub fn canonical_user(id: impl Into<String>) -> Self {
        Grantee::CanonicalUser {
            id: id.into(),
            display_name: None,
        }
    }

    /// Create a canonical user with display name
    pub fn canonical_user_with_name(id: impl Into<String>, name: impl Into<String>) -> Self {
        Grantee::CanonicalUser {
            id: id.into(),
            display_name: Some(name.into()),
        }
    }

    /// Create a group grantee
    pub fn group(uri: impl Into<String>) -> Self {
        Grantee::Group { uri: uri.into() }
    }

    /// All users (anonymous access)
    pub fn all_users() -> Self {
        Grantee::Group {
            uri: "http://acs.amazonaws.com/groups/global/AllUsers".to_string(),
        }
    }

    /// Authenticated users
    pub fn authenticated_users() -> Self {
        Grantee::Group {
            uri: "http://acs.amazonaws.com/groups/global/AuthenticatedUsers".to_string(),
        }
    }

    /// Log delivery group
    pub fn log_delivery() -> Self {
        Grantee::Group {
            uri: "http://acs.amazonaws.com/groups/s3/LogDelivery".to_string(),
        }
    }

    /// Check if grantee matches a principal
    pub fn matches(&self, principal: &str, is_authenticated: bool) -> bool {
        match self {
            Grantee::CanonicalUser { id, .. } => id == principal,
            Grantee::AmazonCustomerByEmail { email_address } => email_address == principal,
            Grantee::Group { uri } => {
                if uri.contains("AllUsers") {
                    true
                } else if uri.contains("AuthenticatedUsers") {
                    is_authenticated
                } else {
                    false
                }
            }
        }
    }
}

// ============================================================================
// Grant
// ============================================================================

/// A single ACL grant
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Grant {
    /// Who receives the permission
    pub grantee: Grantee,
    /// The permission granted
    pub permission: Permission,
}

impl Grant {
    /// Create a new grant
    pub fn new(grantee: Grantee, permission: Permission) -> Self {
        Self {
            grantee,
            permission,
        }
    }
}

// ============================================================================
// Owner
// ============================================================================

/// ACL Owner
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Owner {
    /// Owner's canonical ID
    pub id: String,
    /// Owner's display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl Owner {
    /// Create a new owner
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: None,
        }
    }

    /// Create owner with display name
    pub fn with_name(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            display_name: Some(name.into()),
        }
    }
}

// ============================================================================
// Access Control Policy (Full ACL)
// ============================================================================

/// Full Access Control Policy
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AccessControlPolicy {
    /// Owner of the resource
    pub owner: Owner,
    /// List of grants
    pub access_control_list: AccessControlList,
}

/// Access Control List (grants container)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct AccessControlList {
    /// Grants
    #[serde(default)]
    pub grant: Vec<Grant>,
}

impl AccessControlPolicy {
    /// Create a new ACL with owner
    pub fn new(owner: Owner) -> Self {
        Self {
            owner,
            access_control_list: AccessControlList::default(),
        }
    }

    /// Add a grant
    pub fn add_grant(mut self, grant: Grant) -> Self {
        self.access_control_list.grant.push(grant);
        self
    }

    /// Create ACL from canned ACL
    pub fn from_canned(owner: Owner, canned: CannedAcl) -> Self {
        let mut acl = Self::new(owner.clone());

        // Owner always has full control
        acl = acl.add_grant(Grant::new(
            Grantee::canonical_user_with_name(
                &owner.id,
                owner.display_name.clone().unwrap_or_default(),
            ),
            Permission::FullControl,
        ));

        match canned {
            CannedAcl::Private => {
                // Only owner has access (already added)
            }
            CannedAcl::PublicRead => {
                acl = acl.add_grant(Grant::new(Grantee::all_users(), Permission::Read));
            }
            CannedAcl::PublicReadWrite => {
                acl = acl.add_grant(Grant::new(Grantee::all_users(), Permission::Read));
                acl = acl.add_grant(Grant::new(Grantee::all_users(), Permission::Write));
            }
            CannedAcl::AuthenticatedRead => {
                acl = acl.add_grant(Grant::new(Grantee::authenticated_users(), Permission::Read));
            }
            CannedAcl::BucketOwnerRead | CannedAcl::BucketOwnerFullControl => {
                // These are handled at a higher level with bucket owner info
            }
        }

        acl
    }

    /// Check if a principal has a specific permission
    pub fn has_permission(
        &self,
        principal: &str,
        permission: Permission,
        is_authenticated: bool,
    ) -> bool {
        // Owner always has full control
        if self.owner.id == principal {
            return true;
        }

        // Check grants
        for grant in &self.access_control_list.grant {
            if grant.grantee.matches(principal, is_authenticated) {
                if grant.permission.includes(&permission) {
                    return true;
                }
            }
        }

        false
    }

    /// Check if anonymous access is allowed for a permission
    pub fn allows_anonymous(&self, permission: Permission) -> bool {
        for grant in &self.access_control_list.grant {
            if let Grantee::Group { uri } = &grant.grantee {
                if uri.contains("AllUsers") && grant.permission.includes(&permission) {
                    return true;
                }
            }
        }
        false
    }

    /// Convert to XML string
    pub fn to_xml(&self) -> String {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str(r#"<AccessControlPolicy xmlns="http://s3.amazonaws.com/doc/2006-03-01/">"#);

        // Owner
        xml.push_str("<Owner>");
        xml.push_str(&format!("<ID>{}</ID>", xml_escape(&self.owner.id)));
        if let Some(ref name) = self.owner.display_name {
            xml.push_str(&format!("<DisplayName>{}</DisplayName>", xml_escape(name)));
        }
        xml.push_str("</Owner>");

        // Access Control List
        xml.push_str("<AccessControlList>");
        for grant in &self.access_control_list.grant {
            xml.push_str("<Grant>");

            // Grantee
            match &grant.grantee {
                Grantee::CanonicalUser { id, display_name } => {
                    xml.push_str(r#"<Grantee xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:type="CanonicalUser">"#);
                    xml.push_str(&format!("<ID>{}</ID>", xml_escape(id)));
                    if let Some(ref name) = display_name {
                        xml.push_str(&format!("<DisplayName>{}</DisplayName>", xml_escape(name)));
                    }
                    xml.push_str("</Grantee>");
                }
                Grantee::AmazonCustomerByEmail { email_address } => {
                    xml.push_str(r#"<Grantee xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:type="AmazonCustomerByEmail">"#);
                    xml.push_str(&format!(
                        "<EmailAddress>{}</EmailAddress>",
                        xml_escape(email_address)
                    ));
                    xml.push_str("</Grantee>");
                }
                Grantee::Group { uri } => {
                    xml.push_str(r#"<Grantee xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:type="Group">"#);
                    xml.push_str(&format!("<URI>{}</URI>", xml_escape(uri)));
                    xml.push_str("</Grantee>");
                }
            }

            // Permission
            xml.push_str(&format!("<Permission>{}</Permission>", grant.permission));
            xml.push_str("</Grant>");
        }
        xml.push_str("</AccessControlList>");

        xml.push_str("</AccessControlPolicy>");
        xml
    }
}

/// XML escape helper
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// ============================================================================
// ACL Request Headers
// ============================================================================

/// ACL headers from request
#[derive(Debug, Clone, Default)]
pub struct AclHeaders {
    /// x-amz-acl header (canned ACL)
    pub canned_acl: Option<CannedAcl>,
    /// x-amz-grant-read
    pub grant_read: Option<String>,
    /// x-amz-grant-write
    pub grant_write: Option<String>,
    /// x-amz-grant-read-acp
    pub grant_read_acp: Option<String>,
    /// x-amz-grant-write-acp
    pub grant_write_acp: Option<String>,
    /// x-amz-grant-full-control
    pub grant_full_control: Option<String>,
}

impl AclHeaders {
    /// Parse grant header value into grantees
    pub fn parse_grant_header(value: &str) -> Vec<Grantee> {
        let mut grantees = Vec::new();

        for part in value.split(',') {
            let part = part.trim();
            if let Some((key, val)) = part.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches('"');

                match key {
                    "id" => {
                        grantees.push(Grantee::canonical_user(val));
                    }
                    "emailAddress" => {
                        grantees.push(Grantee::AmazonCustomerByEmail {
                            email_address: val.to_string(),
                        });
                    }
                    "uri" => {
                        grantees.push(Grantee::group(val));
                    }
                    _ => {}
                }
            }
        }

        grantees
    }

    /// Build ACL from headers
    pub fn build_acl(&self, owner: Owner) -> AccessControlPolicy {
        // If canned ACL is specified, use it
        if let Some(canned) = self.canned_acl {
            return AccessControlPolicy::from_canned(owner, canned);
        }

        // Build from grant headers
        let mut acl = AccessControlPolicy::new(owner.clone());

        // Owner always has full control
        acl = acl.add_grant(Grant::new(
            Grantee::canonical_user_with_name(&owner.id, owner.display_name.unwrap_or_default()),
            Permission::FullControl,
        ));

        // Add grants from headers
        if let Some(ref read) = self.grant_read {
            for grantee in Self::parse_grant_header(read) {
                acl = acl.add_grant(Grant::new(grantee, Permission::Read));
            }
        }

        if let Some(ref write) = self.grant_write {
            for grantee in Self::parse_grant_header(write) {
                acl = acl.add_grant(Grant::new(grantee, Permission::Write));
            }
        }

        if let Some(ref read_acp) = self.grant_read_acp {
            for grantee in Self::parse_grant_header(read_acp) {
                acl = acl.add_grant(Grant::new(grantee, Permission::ReadAcp));
            }
        }

        if let Some(ref write_acp) = self.grant_write_acp {
            for grantee in Self::parse_grant_header(write_acp) {
                acl = acl.add_grant(Grant::new(grantee, Permission::WriteAcp));
            }
        }

        if let Some(ref full) = self.grant_full_control {
            for grantee in Self::parse_grant_header(full) {
                acl = acl.add_grant(Grant::new(grantee, Permission::FullControl));
            }
        }

        acl
    }

    /// Check if any ACL headers are present
    pub fn has_acl_headers(&self) -> bool {
        self.canned_acl.is_some()
            || self.grant_read.is_some()
            || self.grant_write.is_some()
            || self.grant_read_acp.is_some()
            || self.grant_write_acp.is_some()
            || self.grant_full_control.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canned_acl_parsing() {
        assert_eq!(CannedAcl::from_str("private").unwrap(), CannedAcl::Private);
        assert_eq!(
            CannedAcl::from_str("public-read").unwrap(),
            CannedAcl::PublicRead
        );
    }

    #[test]
    fn test_permission_includes() {
        assert!(Permission::FullControl.includes(&Permission::Read));
        assert!(Permission::FullControl.includes(&Permission::Write));
        assert!(!Permission::Read.includes(&Permission::Write));
    }

    #[test]
    fn test_acl_from_canned() {
        let owner = Owner::new("user123");
        let acl = AccessControlPolicy::from_canned(owner, CannedAcl::PublicRead);

        assert!(acl.has_permission("user123", Permission::FullControl, true));
        assert!(acl.allows_anonymous(Permission::Read));
        assert!(!acl.allows_anonymous(Permission::Write));
    }

    #[test]
    fn test_grant_header_parsing() {
        let header = r#"id="user123", uri="http://acs.amazonaws.com/groups/global/AllUsers""#;
        let grantees = AclHeaders::parse_grant_header(header);

        assert_eq!(grantees.len(), 2);
    }

    #[test]
    fn test_acl_to_xml() {
        let owner = Owner::with_name("user123", "Test User");
        let acl = AccessControlPolicy::from_canned(owner, CannedAcl::Private);
        let xml = acl.to_xml();

        assert!(xml.contains("<ID>user123</ID>"));
        assert!(xml.contains("FULL_CONTROL"));
    }
}
