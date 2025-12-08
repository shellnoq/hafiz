//! Page components

mod cluster;
mod dashboard;
mod buckets;
mod ldap;
mod objects;
mod users;
mod settings;
mod not_found;

pub use cluster::ClusterPage;
pub use dashboard::DashboardPage;
pub use buckets::{BucketsPage, BucketDetailPage};
pub use ldap::LdapSettingsPage;
pub use objects::ObjectsPage;
pub use users::UsersPage;
pub use settings::SettingsPage;
pub use not_found::NotFoundPage;
