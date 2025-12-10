//! LDAP/Active Directory authentication module
//!
//! Provides enterprise authentication via:
//! - LDAP (OpenLDAP, 389 Directory Server)
//! - Microsoft Active Directory
//!
//! Features:
//! - User authentication
//! - Group-based policy mapping
//! - User caching
//! - TLS/STARTTLS support

mod client;
mod types;

pub use client::{LdapAuthProvider, LdapClient};
pub use types::*;
