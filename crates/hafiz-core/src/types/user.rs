//! User types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub access_key: String,
    pub secret_key: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(access_key: String, secret_key: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            access_key,
            secret_key,
            display_name: None,
            email: None,
            is_admin: false,
            created_at: Utc::now(),
        }
    }

    pub fn root(access_key: String, secret_key: String) -> Self {
        Self {
            id: "root".to_string(),
            access_key,
            secret_key,
            display_name: Some("Root User".to_string()),
            email: None,
            is_admin: true,
            created_at: Utc::now(),
        }
    }
}

/// Credentials for Admin API (extended from User)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub access_key: String,
    pub secret_key: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub policies: Vec<String>,
}

impl Credentials {
    pub fn new(access_key: String, secret_key: String) -> Self {
        Self {
            access_key,
            secret_key,
            name: None,
            email: None,
            enabled: true,
            created_at: Utc::now(),
            last_used: None,
            policies: Vec::new(),
        }
    }
    
    pub fn from_user(user: &User) -> Self {
        Self {
            access_key: user.access_key.clone(),
            secret_key: user.secret_key.clone(),
            name: user.display_name.clone(),
            email: user.email.clone(),
            enabled: true,
            created_at: user.created_at,
            last_used: None,
            policies: if user.is_admin {
                vec!["admin".to_string()]
            } else {
                Vec::new()
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub id: String,
    pub display_name: Option<String>,
}

impl From<User> for Owner {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            display_name: u.display_name,
        }
    }
}
