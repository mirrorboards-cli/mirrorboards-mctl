//! Git operations using external git CLI.
//!
//! This module provides a wrapper around the git command-line interface,
//! avoiding the complexity of libgit2 and SSH agent issues.

pub mod client;
pub mod command;
pub mod output;
pub mod status;

// Re-exports for convenience
pub use client::{GitClient, GitClientConfig};
pub use command::GitCommand;
pub use status::{BranchInfo, FileStatus, FileStatusCode, RepositoryStatus, SyncStatus};
