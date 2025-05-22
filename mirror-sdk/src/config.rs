//! Configuration module for mirror-sdk
//!
//! This module provides the data structures and functions for working with
//! the mirror.toml file format.

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::Error;
use crate::repository::Repository;

/// Default filename for mirror configuration
pub const DEFAULT_FILENAME: &str = "mirror.toml";

/// Environment variable name for custom mirror.toml path
pub const ENV_MIRROR_PATH: &str = "MIRROR_CONFIG_PATH";

/// Represents the mirror.toml configuration file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MirrorConfig {
    /// List of repositories
    pub repositories: Vec<Repository>,
    
    /// Path to the configuration file (not serialized)
    #[serde(skip)]
    path: Option<PathBuf>,
}

impl MirrorConfig {
    /// Creates a new empty mirror configuration
    ///
    /// # Returns
    ///
    /// A new `MirrorConfig` instance with no repositories
    ///
    /// # Example
    ///
    /// ```
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let config = MirrorConfig::new();
    /// ```
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
            path: None,
        }
    }
    
    /// Loads a mirror configuration from the default location or environment variable
    ///
    /// This method will try to load the configuration from:
    /// 1. The path specified in the MIRROR_CONFIG_PATH environment variable
    /// 2. The default location (./mirror.toml)
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let config = MirrorConfig::load().unwrap();
    /// ```
    pub fn load() -> Result<Self, Error> {
        // Try to get the path from the environment variable
        if let Ok(path) = std::env::var(ENV_MIRROR_PATH) {
            return Self::load_from(Path::new(&path));
        }
        
        // Fall back to the default location
        Self::load_from(Path::new(DEFAULT_FILENAME))
    }
    
    /// Loads a mirror configuration from the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the mirror.toml file
    ///
    /// # Returns
    ///
    /// A `Result` containing the loaded configuration or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    /// use std::path::Path;
    ///
    /// let config = MirrorConfig::load_from(Path::new("custom/path/mirror.toml")).unwrap();
    /// ```
    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        
        // Read the file
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse the TOML
        let mut config: MirrorConfig = toml::from_str(&contents)?;
        
        // Set the path
        config.path = Some(path.to_path_buf());
        
        Ok(config)
    }
    
    /// Saves the mirror configuration to the default location or the path it was loaded from
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let mut config = MirrorConfig::new();
    /// config.save().unwrap();
    /// ```
    pub fn save(&self) -> Result<(), Error> {
        if let Some(path) = &self.path {
            self.save_to(path)
        } else {
            self.save_to(Path::new(DEFAULT_FILENAME))
        }
    }
    
    /// Saves the mirror configuration to the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to save the mirror.toml file
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    /// use std::path::Path;
    ///
    /// let mut config = MirrorConfig::new();
    /// config.save_to(Path::new("custom/path/mirror.toml")).unwrap();
    /// ```
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let path = path.as_ref();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Serialize to TOML
        let toml = toml::to_string_pretty(self)?;
        
        // Write to file
        let mut file = File::create(path)?;
        file.write_all(toml.as_bytes())?;
        
        Ok(())
    }
    
    /// Adds a repository to the configuration
    ///
    /// # Arguments
    ///
    /// * `repository` - Repository to add
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    /// use mirror_sdk::repository::Repository;
    ///
    /// let mut config = MirrorConfig::new();
    /// let repo = Repository::new(
    ///     "git@github.com:mirrorboards/example-repo.git",
    ///     "example/path",
    /// ).unwrap();
    /// config.add_repository(repo).unwrap();
    /// ```
    pub fn add_repository(&mut self, mut repository: Repository) -> Result<(), Error> {
        // Generate an ID if one doesn't exist
        let id = repository.get_id();
        
        // Check for duplicate ID
        if self.repositories.iter().any(|r| r.id == Some(id.clone())) {
            return Err(Error::DuplicateId(id));
        }
        
        // Allow path collision as per requirements
        
        // Add the repository
        self.repositories.push(repository);
        
        Ok(())
    }
    
    /// Removes a repository from the configuration by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the repository to remove
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let mut config = MirrorConfig::load().unwrap();
    /// config.remove_repository("repo-id").unwrap();
    /// ```
    pub fn remove_repository<S: AsRef<str>>(&mut self, id: S) -> Result<(), Error> {
        let id = id.as_ref();
        let initial_len = self.repositories.len();
        
        self.repositories.retain(|r| {
            if let Some(repo_id) = &r.id {
                repo_id != id
            } else {
                true
            }
        });
        
        if self.repositories.len() == initial_len {
            return Err(Error::RepositoryNotFound(id.to_string()));
        }
        
        Ok(())
    }
    
    /// Gets a repository by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the repository to get
    ///
    /// # Returns
    ///
    /// A `Result` containing the repository or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let config = MirrorConfig::load().unwrap();
    /// let repo = config.get_repository("repo-id").unwrap();
    /// ```
    pub fn get_repository<S: AsRef<str>>(&self, id: S) -> Result<&Repository, Error> {
        let id = id.as_ref();
        
        self.repositories
            .iter()
            .find(|r| r.id.as_ref().map_or(false, |repo_id| repo_id == id))
            .ok_or_else(|| Error::RepositoryNotFound(id.to_string()))
    }
    
    /// Gets a mutable reference to a repository by ID
    ///
    /// # Arguments
    ///
    /// * `id` - ID of the repository to get
    ///
    /// # Returns
    ///
    /// A `Result` containing a mutable reference to the repository or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let mut config = MirrorConfig::load().unwrap();
    /// let repo = config.get_repository_mut("repo-id").unwrap();
    /// repo.path = "new/path".to_string();
    /// ```
    pub fn get_repository_mut<S: AsRef<str>>(&mut self, id: S) -> Result<&mut Repository, Error> {
        let id = id.as_ref();
        
        self.repositories
            .iter_mut()
            .find(|r| r.id.as_ref().map_or(false, |repo_id| repo_id == id))
            .ok_or_else(|| Error::RepositoryNotFound(id.to_string()))
    }
    
    /// Gets all repositories
    ///
    /// # Returns
    ///
    /// A slice of all repositories
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let config = MirrorConfig::load().unwrap();
    /// for repo in config.get_repositories() {
    ///     println!("Repository: {:?}", repo);
    /// }
    /// ```
    pub fn get_repositories(&self) -> &[Repository] {
        &self.repositories
    }
    
    /// Gets all repositories with a specific tag
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag to filter by
    ///
    /// # Returns
    ///
    /// A vector of repositories with the specified tag
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let config = MirrorConfig::load().unwrap();
    /// let monorepo_repos = config.get_repositories_by_tag("monorepo");
    /// ```
    pub fn get_repositories_by_tag<S: AsRef<str>>(&self, tag: S) -> Vec<&Repository> {
        let tag = tag.as_ref();
        
        self.repositories
            .iter()
            .filter(|r| {
                r.tags.as_ref().map_or(false, |tags| tags.iter().any(|t| t == tag))
            })
            .collect()
    }
    
    /// Initializes a new mirror.toml file at the default location
    ///
    /// # Returns
    ///
    /// A `Result` containing the new configuration or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    ///
    /// let config = MirrorConfig::init().unwrap();
    /// ```
    pub fn init() -> Result<Self, Error> {
        Self::init_at(Path::new(DEFAULT_FILENAME))
    }
    
    /// Initializes a new mirror.toml file at the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to create the mirror.toml file
    ///
    /// # Returns
    ///
    /// A `Result` containing the new configuration or an error
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use mirror_sdk::config::MirrorConfig;
    /// use std::path::Path;
    ///
    /// let config = MirrorConfig::init_at(Path::new("custom/path/mirror.toml")).unwrap();
    /// ```
    pub fn init_at<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        
        // Check if the file already exists
        if path.exists() {
            return Err(Error::Other(format!("File already exists: {}", path.display())));
        }
        
        // Create a new configuration
        let config = Self::new();
        
        // Save it to the specified path
        config.save_to(path)?;
        
        // Return the configuration with the path set
        Ok(Self {
            repositories: Vec::new(),
            path: Some(path.to_path_buf()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_mirror_config_add_repository() {
        let mut config = MirrorConfig::new();
        let repo = Repository::new("git@github.com:mirrorboards/example-repo.git", "example/path")
            .unwrap()
            .with_id("test-id");
        
        config.add_repository(repo).unwrap();
        
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].id, Some("test-id".to_string()));
    }
    
    #[test]
    fn test_mirror_config_remove_repository() {
        let mut config = MirrorConfig::new();
        let repo = Repository::new("git@github.com:mirrorboards/example-repo.git", "example/path")
            .unwrap()
            .with_id("test-id");
        
        config.add_repository(repo).unwrap();
        config.remove_repository("test-id").unwrap();
        
        assert_eq!(config.repositories.len(), 0);
    }
    
    #[test]
    fn test_mirror_config_save_and_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("mirror.toml");
        
        // Create a config with a repository
        let mut config = MirrorConfig::new();
        let repo = Repository::new("git@github.com:mirrorboards/example-repo.git", "example/path")
            .unwrap()
            .with_id("test-id");
        
        config.add_repository(repo).unwrap();
        
        // Save it
        config.save_to(&file_path).unwrap();
        
        // Load it back
        let loaded_config = MirrorConfig::load_from(&file_path).unwrap();
        
        // Check that it's the same
        assert_eq!(loaded_config.repositories.len(), 1);
        assert_eq!(loaded_config.repositories[0].id, Some("test-id".to_string()));
        assert_eq!(loaded_config.repositories[0].origin, "git@github.com:mirrorboards/example-repo.git");
        assert_eq!(loaded_config.repositories[0].path, "example/path");
    }
}