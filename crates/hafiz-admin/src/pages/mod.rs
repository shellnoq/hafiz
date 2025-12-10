//! Page components

mod buckets;
mod cluster;
mod dashboard;
mod ldap;
mod not_found;
mod objects;
mod settings;
mod users;

pub use buckets::{BucketDetailPage, BucketsPage};
pub use cluster::ClusterPage;
pub use dashboard::DashboardPage;
pub use ldap::LdapSettingsPage;
pub use not_found::NotFoundPage;
pub use objects::ObjectsPage;
pub use settings::SettingsPage;
pub use users::UsersPage;
