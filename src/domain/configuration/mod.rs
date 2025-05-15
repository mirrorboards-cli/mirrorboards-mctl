//! # Configuration Domain Module
//!
//! This module defines the core configuration entities and validation rules.
//! It represents the structure of the TOML configuration file.

use std::path::PathBuf;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::domain::repository::{Repository, SshConfig, CommandConfig};

/// Complete configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Global settings
    #[serde(default)]
    pub global: GlobalConfig,
    
    /// Authentication settings
    #[serde(default)]
    pub auth: AuthConfig,
    
    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
    
    /// Command-specific settings
    #[serde(default)]
    pub commands: CommandsConfig,
    
    /// Repository definitions
    #[serde(default)]
    pub repositories: Vec<Repository>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            auth: AuthConfig::default(),
            logging: LoggingConfig::default(),
            commands: CommandsConfig::default(),
            repositories: Vec::new(),
        }
    }
}

/// Global configuration settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalConfig {
    /// Enable parallel repository operations
    #[serde(default = "default_parallel")]
    pub parallel: bool,
    
    /// Maximum number of parallel operations
    #[serde(default = "default_max_threads")]
    pub max_threads: usize,
}

fn default_parallel() -> bool {
    true
}

fn default_max_threads() -> usize {
    8
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            parallel: default_parallel(),
            max_threads: default_max_threads(),
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    /// SSH authentication configuration
    #[serde(default)]
    pub ssh: SshAuthConfig,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            ssh: SshAuthConfig::default(),
        }
    }
}

/// SSH authentication configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SshAuthConfig {
    /// Path to SSH private key
    pub key_path: Option<String>,
    
    /// Command to retrieve passphrase
    pub passphrase_command: Option<String>,
    
    /// Path to known hosts file
    pub known_hosts_path: Option<String>,
}

impl Default for SshAuthConfig {
    fn default() -> Self {
        Self {
            key_path: Some("~/.ssh/id_rsa".to_string()),
            passphrase_command: None,
            known_hosts_path: Some("~/.ssh/known_hosts".to_string()),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,
    
    /// Log file path
    pub file: Option<String>,
    
    /// Log format
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            file: None,
            format: default_log_format(),
        }
    }
}

/// Command-specific configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandsConfig {
    /// Sync command configuration
    #[serde(default)]
    pub sync: SyncCommandConfig,
    
    /// Status command configuration
    #[serde(default)]
    pub status: StatusCommandConfig,
    
    /// Save command configuration
    #[serde(default)]
    pub save: SaveCommandConfig,
}

impl Default for CommandsConfig {
    fn default() -> Self {
        Self {
            sync: SyncCommandConfig::default(),
            status: StatusCommandConfig::default(),
            save: SaveCommandConfig::default(),
        }
    }
}

/// Sync command configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyncCommandConfig {
    /// Clone submodules recursively
    #[serde(default = "default_recursive")]
    pub recursive: bool,
    
    /// Git clone depth
    #[serde(default = "default_depth")]
    pub depth: u32,
    
    /// Operation timeout in seconds
    #[serde(default = "default_sync_timeout")]
    pub timeout_seconds: u64,
}

fn default_recursive() -> bool {
    true
}

fn default_depth() -> u32 {
    1
}

fn default_sync_timeout() -> u64 {
    300
}

impl Default for SyncCommandConfig {
    fn default() -> Self {
        Self {
            recursive: default_recursive(),
            depth: default_depth(),
            timeout_seconds: default_sync_timeout(),
        }
    }
}

/// Status command configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusCommandConfig {
    /// Include untracked files
    #[serde(default)]
    pub include_untracked: bool,
    
    /// Operation timeout in seconds
    #[serde(default = "default_status_timeout")]
    pub timeout_seconds: u64,
}

fn default_status_timeout() -> u64 {
    60
}

impl Default for StatusCommandConfig {
    fn default() -> Self {
        Self {
            include_untracked: false,
            timeout_seconds: default_status_timeout(),
        }
    }
}

/// Save command configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SaveCommandConfig {
    /// Push after commit
    #[serde(default = "default_push")]
    pub push: bool,
    
    /// Sign commits
    #[serde(default)]
    pub sign_commits: bool,
}

fn default_push() -> bool {
    true
}

impl Default for SaveCommandConfig {
    fn default() -> Self {
        Self {
            push: default_push(),
            sign_commits: false,
        }
    }
}

/// Configuration validation
pub trait ConfigValidator {
    /// Validate the configuration
    fn validate(&self, config: &Config) -> Result<(), String>;
}

/// Path expansion utilities
pub trait PathExpander {
    /// Expand all paths in the configuration
    fn expand_paths(&self, config: &mut Config);
    
    /// Expand a single path with environment variables and ~ substitution
    fn expand_path(&self, path: &str) -> PathBuf;
    
    /// Expand environment variables in a string using ${VAR} or ${VAR:-default} syntax
    fn expand_env_vars(&self, input: &str) -> String;
}