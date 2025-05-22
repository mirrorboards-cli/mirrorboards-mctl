//! Mirror SDK - A Rust library for managing mirror.toml configuration files.
//!
//! This library provides functionality for parsing, manipulating, and serializing
//! mirror.toml configuration files, which are used to manage multiple Git repositories.
//!
//! # Features
//!
//! - Parse and serialize mirror.toml configuration files
//! - Manage repository configurations (add, remove, update)
//! - Support file system operations for working with repositories
//! - Handle configuration through default paths and environment variables
//! - Provide a comprehensive error handling strategy
//! - Expose a clean, well-documented public API
//!
//! # Examples
//!
//! ```no_run
//! use mirror_sdk::{MirrorSdk, Repository, RepositoryBuilder, MirrorError};
//!
//! fn main() -> Result<(), MirrorError> {
//!     // Create a new SDK instance
//!     let sdk = MirrorSdk::new();
//!     
//!     // Load an existing configuration
//!     let mut config = sdk.load_config("mirror.toml")?;
//!     
//!     // Create a new repository
//!     let repo = RepositoryBuilder::new()
//!         .origin("git@github.com:example/repo.git")
//!         .branch("main")
//!         .path("example/repo")
//!         .tag("example")
//!         .build()?;
//!     
//!     // Add the repository to the configuration
//!     sdk.add_repository(&mut config, repo)?;
//!     
//!     // Save the updated configuration
//!     sdk.save_config(&config, "mirror.toml")?;
//!     
//!     Ok(())
//! }
//! ```

// Public modules
pub mod config;
pub mod error;
pub mod fs;
pub mod models;
pub mod operations;
pub mod utils;

// Public types
pub use crate::models::{Repository, RepositoryBuilder, MirrorConfig};
pub use crate::error::{MirrorError, ValidationError};
pub use crate::config::ConfigSettings;

use std::path::{Path, PathBuf};

/// Main SDK struct for interacting with mirror.toml configurations.
pub struct MirrorSdk {
    settings: ConfigSettings,
}

impl MirrorSdk {
    /// Create a new SDK instance with default settings.
    pub fn new() -> Self {
        Self {
            settings: ConfigSettings::default(),
        }
    }
    
    /// Create a new SDK instance with custom settings.
    pub fn with_settings(settings: ConfigSettings) -> Self {
        Self { settings }
    }
    
    /// Load a mirror.toml configuration from a file.
    pub fn load_config<P: AsRef<Path>>(&self, path: P) -> Result<MirrorConfig, MirrorError> {
        fs::read_config(path)
    }
    
    /// Save a mirror.toml configuration to a file.
    pub fn save_config<P: AsRef<Path>>(&self, config: &MirrorConfig, path: P) -> Result<(), MirrorError> {
        fs::write_config(config, path)
    }
    
    /// Create a new empty mirror.toml configuration.
    pub fn new_config(&self) -> MirrorConfig {
        MirrorConfig::new()
    }
    
    /// Initialize a new mirror.toml configuration file.
    pub fn init_config<P: AsRef<Path>>(&self, path: P, force: bool) -> Result<MirrorConfig, MirrorError> {
        operations::init_config(path, force)
    }
    
    /// Add a repository to a configuration.
    pub fn add_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
        operations::add_repository(config, repo)
    }
    
    /// Remove a repository from a configuration by path.
    pub fn remove_repository_by_path(&self, config: &mut MirrorConfig, path: &str) -> Result<(), MirrorError> {
        operations::remove_repository_by_path(config, path)
    }
    
    /// Remove a repository from a configuration by ID.
    pub fn remove_repository_by_id(&self, config: &mut MirrorConfig, id: &str) -> Result<(), MirrorError> {
        operations::remove_repository_by_id(config, id)
    }
    
    /// Update a repository in a configuration.
    pub fn update_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
        operations::update_repository(config, repo)
    }
    
    /// Update a repository in a configuration by ID.
    pub fn update_repository_by_id(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
        operations::update_repository_by_id(config, repo)
    }
    
    /// Find a repository by path.
    pub fn find_repository_by_path<'a>(&self, config: &'a MirrorConfig, path: &str) -> Option<&'a Repository> {
        config.find_by_path(path)
    }
    
    /// Find a repository by ID.
    pub fn find_repository_by_id<'a>(&self, config: &'a MirrorConfig, id: &str) -> Option<&'a Repository> {
        config.find_by_id(id)
    }
    
    /// Find repositories by tag.
    pub fn find_repositories_by_tag<'a>(&self, config: &'a MirrorConfig, tag: &str) -> Vec<&'a Repository> {
        config.find_by_tag(tag)
    }
    
    /// Validate a configuration.
    pub fn validate_config(&self, config: &MirrorConfig) -> Result<(), error::ValidationError> {
        utils::validate_config(config)
    }
    
    /// Get the path to the mirror.toml file.
    pub fn get_config_path(&self) -> Result<PathBuf, MirrorError> {
        config::get_config_path(self.settings.default_config_path.as_deref())
    }
}

impl Default for MirrorSdk {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_sdk_new() {
        let sdk = MirrorSdk::new();
        assert_eq!(sdk.settings.validate_paths, true);
        assert_eq!(sdk.settings.validate_origins, true);
    }
    
    #[test]
    fn test_sdk_with_settings() {
        let settings = ConfigSettings::default()
            .with_validate_paths(false)
            .with_validate_origins(false);
        
        let sdk = MirrorSdk::with_settings(settings);
        assert_eq!(sdk.settings.validate_paths, false);
        assert_eq!(sdk.settings.validate_origins, false);
    }
    
    #[test]
    fn test_sdk_new_config() {
        let sdk = MirrorSdk::new();
        let config = sdk.new_config();
        assert!(config.repositories.is_empty());
    }
    
    #[test]
    fn test_sdk_init_config() {
        let sdk = MirrorSdk::new();
        let dir = tempdir().unwrap();
        let path = dir.path().join("mirror.toml");
        
        let config = sdk.init_config(&path, false).unwrap();
        assert!(config.repositories.is_empty());
        assert!(path.exists());
    }
    
    #[test]
    fn test_sdk_load_save_config() {
        let sdk = MirrorSdk::new();
        let dir = tempdir().unwrap();
        let path = dir.path().join("mirror.toml");
        
        // Create a new config
        let mut config = sdk.new_config();
        
        // Add a repository
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        sdk.add_repository(&mut config, repo).unwrap();
        
        // Save the config
        sdk.save_config(&config, &path).unwrap();
        
        // Load the config
        let loaded_config = sdk.load_config(&path).unwrap();
        
        assert_eq!(loaded_config.repositories.len(), 1);
        assert_eq!(loaded_config.repositories[0].origin, "git@github.com:example/repo.git");
        assert_eq!(loaded_config.repositories[0].branch, "main");
        assert_eq!(loaded_config.repositories[0].path, "example/repo");
    }
}