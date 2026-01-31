//! mctl - Mirror configuration management tool
//!
//! This crate provides both a library and a CLI tool for managing multiple git
//! repositories defined in a `mirror.toml` configuration file.
//!
//! # Features
//!
//! - **Workspaces**: Group repositories into logical workspaces for filtering
//! - **Version Pinning**: Pin repositories to a branch, tag, or specific commit
//! - **Includes**: Compose configurations from multiple files
//! - **Remote Config**: Sync your mirror.toml with a remote repository
//! - **Snapshots**: Create point-in-time snapshots of repository states
//!
//! # Library Usage
//!
//! ```no_run
//! use mctl::{MirrorConfig, GitClient, Repository, VersionSpec};
//!
//! // Load configuration
//! let config = MirrorConfig::load_default().unwrap();
//!
//! // Create git client
//! let git = GitClient::new();
//!
//! // Clone a repository
//! let repo = &config.repositories[0];
//! git.clone(&repo.git, repo.path.as_ref(), &repo.version_spec()).unwrap();
//! ```
//!
//! # CLI Usage
//!
//! ```bash
//! # Initialize a new configuration
//! mctl init
//!
//! # Add a repository
//! mctl add git@github.com:owner/repo.git --workspace api
//!
//! # Sync all repositories
//! mctl sync
//!
//! # Check status
//! mctl status
//!
//! # Create a snapshot
//! mctl snapshot
//! ```

pub mod cli;
pub mod core;
pub mod git;

// Re-export commonly used types
pub use core::{
    ConfigError, ConfigManager, ConfigResult, GitError, GitResult, MirrorConfig, RawMirrorConfig,
    RemoteConfig, Repository, VersionSpec,
};

pub use git::{GitClient, GitClientConfig, RepositoryStatus};
