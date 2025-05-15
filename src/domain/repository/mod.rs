//! # Repository Domain Module
//!
//! This module defines the core entities and interfaces related to repository operations.
//! It establishes the contract that implementation layers must fulfill.

pub mod orchestrator;
#[cfg(test)]
pub mod orchestrator_tests;
use std::path::{Path, PathBuf};
use anyhow::Result;
use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// Represents the status of a repository
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Whether the repository has uncommitted changes
    pub has_changes: bool,
    /// Whether the repository has unpushed commits
    pub has_unpushed_commits: bool,
    /// Current branch name
    pub current_branch: String,
    /// List of changed files
    pub changed_files: HashSet<String>,
    /// Additional status information
    pub message: Option<String>,
}

/// Represents a repository in the configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Local path to the repository
    pub path: PathBuf,
    /// Remote origin URL
    pub origin: String,
    /// Branch to use (if specified)
    pub branch: Option<String>,
    /// Whether this is a Git repository
    #[serde(default = "default_is_git")]
    pub is_git: bool,
    /// Whether the repository is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Custom tags for grouping
    #[serde(default)]
    pub tags: Vec<String>,
    /// Repository-specific configuration overrides
    #[serde(default)]
    pub config_overrides: Option<RepositoryConfig>,
}

fn default_is_git() -> bool {
    true
}

fn default_enabled() -> bool {
    true
}

/// Repository-specific configuration overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// SSH authentication settings
    #[serde(default)]
    pub ssh: Option<SshConfig>,
    /// Command-specific settings
    #[serde(default)]
    pub commands: Option<CommandConfig>,
}

/// SSH configuration for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    /// Path to SSH key
    pub key_path: Option<PathBuf>,
    /// Path to known hosts file
    pub known_hosts_path: Option<PathBuf>,
    /// Command to retrieve passphrase
    pub passphrase_command: Option<String>,
}

/// Command-specific configuration for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Sync command configuration
    pub sync: Option<SyncCommandConfig>,
    /// Status command configuration
    pub status: Option<StatusCommandConfig>,
    /// Save command configuration
    pub save: Option<SaveCommandConfig>,
}

/// Configuration for the sync command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCommandConfig {
    /// Whether to clone recursively
    pub recursive: Option<bool>,
    /// Git clone depth
    pub depth: Option<u32>,
    /// Operation timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Configuration for the status command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusCommandConfig {
    /// Whether to include untracked files
    pub include_untracked: Option<bool>,
    /// Operation timeout in seconds
    pub timeout_seconds: Option<u64>,
}

/// Configuration for the save command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveCommandConfig {
    /// Whether to push after commit
    pub push: Option<bool>,
    /// Whether to sign commits
    pub sign_commits: Option<bool>,
}

/// Core interface for repository operations
pub trait RepositoryOperations {
    /// Clone a repository from the given URL to the specified path
    fn clone(&self, url: &str, path: &Path) -> Result<()>;
    
    /// Update submodules in a repository
    fn update_submodules(&self, path: &Path) -> Result<()>;
    
    /// Check if a repository has changes
    fn has_changes(&self, path: &Path) -> Result<bool>;
    
    /// Commit changes in a repository with the given message
    fn commit_changes(&self, path: &Path, message: &str) -> Result<()>;
    
    /// Push changes to the remote repository
    fn push_changes(&self, path: &Path) -> Result<()>;
    
    /// Get the status of a repository
    fn get_status(&self, path: &Path) -> Result<RepositoryStatus>;
    
    /// Get the remote URL of a repository
    fn get_remote_url(&self, path: &Path) -> Result<String>;
}