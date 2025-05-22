//! Repository management module for MCTL
//!
//! This module handles repository management operations.

mod manager;
mod save;
mod status;
mod sync;
mod update;

pub use manager::RepositoryManager;
pub use save::save_changes;
pub use status::check_status;
pub use sync::{get_sync_summary, sync_repositories};
pub use update::update_repositories;
