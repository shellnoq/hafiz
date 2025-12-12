//! Metadata storage for Hafiz
//!
//! Currently supports SQLite backend.
//! PostgreSQL support planned for future releases.

pub mod repository;
pub mod traits;

// PostgreSQL disabled for now - needs implementation fixes
// pub mod postgres;

pub use repository::MetadataStore;
pub use traits::*;
