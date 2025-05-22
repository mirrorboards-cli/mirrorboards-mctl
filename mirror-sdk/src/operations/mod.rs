//! Repository management operations for the Mirror SDK.

pub mod add;
pub mod init;
pub mod remove;
pub mod update;

pub use add::add_repository;
pub use init::init_config;
pub use remove::{remove_repository_by_path, remove_repository_by_id};
pub use update::{update_repository, update_repository_by_id};