//! Mirror SDK - Library for managing multi-repository Git workspaces
//!
//! This crate provides the core functionality for the mctl CLI tool,
//! including configuration loading, git operations, workspace management,
//! and snapshot functionality.

pub mod config;
pub mod config_repo;
pub mod error;
pub mod git;
pub mod models;
pub mod snapshot;
pub mod workspace;

// Re-export commonly used types
pub use config::{ConfigLoader, ConfigValidator};
pub use config_repo::ConfigRepoManager;
pub use error::{MirrorError, Result};
pub use git::{GitManager, SyncResult};
pub use models::{ConfigRepo, MirrorConfig, RefSpec, Repository, Snapshot, SnapshotRepository};
pub use snapshot::{SnapshotInfo, SnapshotManager};
pub use workspace::WorkspaceManager;
