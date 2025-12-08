//! Page components

mod dashboard;
mod buckets;
mod objects;
mod users;
mod settings;
mod not_found;

pub use dashboard::DashboardPage;
pub use buckets::{BucketsPage, BucketDetailPage};
pub use objects::ObjectsPage;
pub use users::UsersPage;
pub use settings::SettingsPage;
pub use not_found::NotFoundPage;
