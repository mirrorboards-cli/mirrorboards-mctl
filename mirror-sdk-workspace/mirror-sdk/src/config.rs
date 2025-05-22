//! Configuration management for mirror.toml.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::MirrorError;
use crate::repository::Repository;
use crate::utils::{create_parent_dirs, normalize_path, path_exists_and_is_file, resolve_config_path};
use crate::Result;

/// Represents the configuration in mirror.toml.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MirrorConfig {
    /// The list of repositories in the configuration.
    #[serde(default)]
    pub repositories: Vec<Repository>,

    /// The path to the configuration file (not serialized).
    #[serde(skip)]
    config_path: Option<PathBuf>,
}

impl MirrorConfig {
    /// Creates a new empty configuration.
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
            config_path: None,
        }
    }

    /// Loads a configuration from a file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    ///
    /// * `Result<MirrorConfig>` - The loaded configuration or an error
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if !path_exists_and_is_file(path) {
            return Err(MirrorError::ConfigFileNotFound(path.to_path_buf()));
        }

        let content = fs::read_to_string(path)?;
        let mut config: MirrorConfig = toml::from_str(&content)?;
        config.config_path = Some(path.to_path_buf());

        Ok(config)
    }

    /// Loads a configuration from the environment variable or default path.
    ///
    /// # Returns
    ///
    /// * `Result<MirrorConfig>` - The loaded configuration or an error
    pub fn load_from_env() -> Result<Self> {
        let path = resolve_config_path(None)?;
        Self::load_from_file(&path)
    }

    /// Initializes a new configuration file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    ///
    /// * `Result<MirrorConfig>` - The initialized configuration or an error
    pub fn init(path: Option<&Path>) -> Result<Self> {
        let path = resolve_config_path(path)?;
        
        if path_exists_and_is_file(&path) {
            return Err(MirrorError::Other(format!(
                "Configuration file already exists at '{}'",
                path.display()
            )));
        }

        let config = Self::new();
        let mut config_with_path = config.clone();
        config_with_path.config_path = Some(path);
        config_with_path.save()?;

        Ok(config_with_path)
    }

    /// Saves the configuration to a file.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the configuration was saved, error otherwise
    pub fn save(&self) -> Result<()> {
        let path = self.config_path.as_ref().ok_or_else(|| {
            MirrorError::Other("Configuration path not set. Use save_to() instead.".to_string())
        })?;

        self.save_to(path)
    }

    /// Saves the configuration to a specific file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to save the configuration to
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if the configuration was saved, error otherwise
    pub fn save_to(&self, path: &Path) -> Result<()> {
        create_parent_dirs(path)?;
        
        let content = toml::to_string_pretty(self)?;
        fs_err::write(path, content)?;
        
        Ok(())
    }

    /// Adds a repository to the configuration.
    ///
    /// # Arguments
    ///
    /// * `repository` - The repository to add
    ///
    /// # Returns
    ///
    /// * `Result<MirrorConfig>` - The updated configuration or an error
    pub fn add_repository(&mut self, repository: Repository) -> Result<&mut Self> {
        // Check for duplicate ID if an ID is provided
        if let Some(id) = &repository.id {
            if self.repositories.iter().any(|r| r.id.as_ref() == Some(id)) {
                return Err(MirrorError::DuplicateRepositoryId(id.clone()));
            }
        }

        // Check for duplicate path (unless explicitly allowed)
        if self.repositories.iter().any(|r| r.path == repository.path) {
            // Note: In a real implementation, you might want to add a flag to allow path collisions
            // For now, we'll just add the repository without checking for path collisions
        }

        self.repositories.push(repository);
        Ok(self)
    }

    /// Removes a repository from the configuration by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the repository to remove
    ///
    /// # Returns
    ///
    /// * `Result<MirrorConfig>` - The updated configuration or an error
    pub fn remove_repository_by_id(&mut self, id: &str) -> Result<&mut Self> {
        let initial_len = self.repositories.len();
        self.repositories.retain(|r| r.id.as_deref() != Some(id));
        
        if self.repositories.len() == initial_len {
            return Err(MirrorError::RepositoryNotFound(id.to_string()));
        }
        
        Ok(self)
    }

    /// Removes a repository from the configuration by path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the repository to remove
    ///
    /// # Returns
    ///
    /// * `Result<MirrorConfig>` - The updated configuration or an error
    pub fn remove_repository_by_path(&mut self, path: &str) -> Result<&mut Self> {
        let normalized_path = normalize_path(path)?;
        let initial_len = self.repositories.len();
        
        self.repositories.retain(|r| {
            if let Ok(repo_path) = normalize_path(&r.path) {
                repo_path != normalized_path
            } else {
                true // Keep repositories with invalid paths
            }
        });
        
        if self.repositories.len() == initial_len {
            return Err(MirrorError::RepositoryNotFound(path.to_string()));
        }
        
        Ok(self)
    }

    /// Gets a repository by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the repository to get
    ///
    /// # Returns
    ///
    /// * `Option<&Repository>` - The repository if found, None otherwise
    pub fn get_repository_by_id(&self, id: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.id.as_deref() == Some(id))
    }

    /// Gets a repository by path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the repository to get
    ///
    /// # Returns
    ///
    /// * `Result<Option<&Repository>>` - The repository if found, None otherwise, or an error
    pub fn get_repository_by_path(&self, path: &str) -> Result<Option<&Repository>> {
        let normalized_path = normalize_path(path)?;
        
        for repo in &self.repositories {
            if let Ok(repo_path) = normalize_path(&repo.path) {
                if repo_path == normalized_path {
                    return Ok(Some(repo));
                }
            }
        }
        
        Ok(None)
    }

    /// Updates a repository in the configuration.
    ///
    /// # Arguments
    ///
    /// * `id` - The ID of the repository to update
    /// * `updated_repo` - The updated repository
    ///
    /// # Returns
    ///
    /// * `Result<MirrorConfig>` - The updated configuration or an error
    pub fn update_repository(&mut self, id: &str, updated_repo: Repository) -> Result<&mut Self> {
        let index = self.repositories
            .iter()
            .position(|r| r.id.as_deref() == Some(id))
            .ok_or_else(|| MirrorError::RepositoryNotFound(id.to_string()))?;
        
        // Check for duplicate path if the path is changing
        let current_path = &self.repositories[index].path;
        if updated_repo.path != *current_path {
            if self.repositories.iter().any(|r| r.path == updated_repo.path) {
                // Note: In a real implementation, you might want to add a flag to allow path collisions
                // For now, we'll just update the repository without checking for path collisions
            }
        }
        
        self.repositories[index] = updated_repo;
        Ok(self)
    }

    /// Gets all repositories with a specific tag.
    ///
    /// # Arguments
    ///
    /// * `tag` - The tag to filter by
    ///
    /// # Returns
    ///
    /// * `Vec<&Repository>` - The repositories with the tag
    pub fn get_repositories_by_tag(&self, tag: &str) -> Vec<&Repository> {
        self.repositories
            .iter()
            .filter(|r| {
                if let Some(tags) = &r.tags {
                    tags.contains(tag)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Gets the path to the configuration file.
    ///
    /// # Returns
    ///
    /// * `Option<&Path>` - The path to the configuration file, or None if not set
    pub fn get_config_path(&self) -> Option<&Path> {
        self.config_path.as_deref()
    }

    /// Sets the path to the configuration file.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file
    pub fn set_config_path(&mut self, path: PathBuf) {
        self.config_path = Some(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::Repository;
    // No need for assert_fs::prelude::* as we're using tempdir directly
    use tempfile::tempdir;

    #[test]
    fn test_new_config() {
        let config = MirrorConfig::new();
        assert!(config.repositories.is_empty());
        assert!(config.config_path.is_none());
    }

    #[test]
    fn test_add_repository() {
        let mut config = MirrorConfig::new();
        let repo = Repository {
            id: Some("test-repo".to_string()),
            origin: "git@github.com:example/repo.git".to_string(),
            branch: Some("main".to_string()),
            path: "./example/repo".to_string(),
            lock: Some(false),
            tags: None,
        };

        config.add_repository(repo.clone()).unwrap();
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].id, Some("test-repo".to_string()));
    }

    #[test]
    fn test_add_duplicate_id() {
        let mut config = MirrorConfig::new();
        let repo1 = Repository {
            id: Some("test-repo".to_string()),
            origin: "git@github.com:example/repo1.git".to_string(),
            branch: None,
            path: "./example/repo1".to_string(),
            lock: None,
            tags: None,
        };

        let repo2 = Repository {
            id: Some("test-repo".to_string()),
            origin: "git@github.com:example/repo2.git".to_string(),
            branch: None,
            path: "./example/repo2".to_string(),
            lock: None,
            tags: None,
        };

        config.add_repository(repo1).unwrap();
        let result = config.add_repository(repo2);
        assert!(matches!(result, Err(MirrorError::DuplicateRepositoryId(_))));
    }

    #[test]
    fn test_remove_repository_by_id() {
        let mut config = MirrorConfig::new();
        let repo = Repository {
            id: Some("test-repo".to_string()),
            origin: "git@github.com:example/repo.git".to_string(),
            branch: None,
            path: "./example/repo".to_string(),
            lock: None,
            tags: None,
        };

        config.add_repository(repo).unwrap();
        config.remove_repository_by_id("test-repo").unwrap();
        assert!(config.repositories.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_repository() {
        let mut config = MirrorConfig::new();
        let result = config.remove_repository_by_id("nonexistent");
        assert!(matches!(result, Err(MirrorError::RepositoryNotFound(_))));
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("mirror.toml");
        
        let mut config = MirrorConfig::new();
        config.set_config_path(config_path.clone());
        
        let repo = Repository {
            id: Some("test-repo".to_string()),
            origin: "git@github.com:example/repo.git".to_string(),
            branch: Some("main".to_string()),
            path: "./example/repo".to_string(),
            lock: Some(false),
            tags: None,
        };
        
        config.add_repository(repo).unwrap();
        config.save().unwrap();
        
        let loaded_config = MirrorConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded_config.repositories.len(), 1);
        assert_eq!(loaded_config.repositories[0].id, Some("test-repo".to_string()));
    }

    #[test]
    fn test_init() {
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("mirror.toml");
        
        let config = MirrorConfig::init(Some(&config_path)).unwrap();
        assert!(config_path.exists());
        assert!(config.repositories.is_empty());
        
        // Trying to init again should fail
        let result = MirrorConfig::init(Some(&config_path));
        assert!(result.is_err());
    }
}