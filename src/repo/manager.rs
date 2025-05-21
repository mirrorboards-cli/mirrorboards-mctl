//! Repository manager module for MCTL
//!
//! This module provides a manager for handling multiple repositories.

use crate::config::mirror_config::{MirrorConfig, Repository};
use crate::error::types::{ErrorCode, MctlError};
use crate::git::credentials::GitCredentials;
use crate::git::operations;
use crate::git::repository::GitRepository;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Repository manager for handling multiple repositories
pub struct RepositoryManager {
    /// Mirror configuration
    config: MirrorConfig,

    /// Path to the configuration file
    config_path: PathBuf,

    /// Map of repository instances
    repositories: HashMap<String, GitRepository>,

    /// Git credentials for authentication
    credentials: Option<GitCredentials>,
}

impl RepositoryManager {
    /// Create a new repository manager from a configuration file
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self, MctlError> {
        let config_path = config_path.as_ref().to_path_buf();

        // Load the configuration
        let config = MirrorConfig::load(&config_path)?;

        debug!(
            "Creating repository manager with {} repositories",
            config.repositories.len()
        );

        let mut manager = Self {
            config,
            config_path,
            repositories: HashMap::new(),
            credentials: None,
        };

        // Initialize repositories
        manager.initialize_repositories()?;

        Ok(manager)
    }

    /// Set credentials for authentication
    pub fn set_credentials(&mut self, credentials: GitCredentials) {
        self.credentials = Some(credentials);
    }

    /// Initialize repositories from configuration
    fn initialize_repositories(&mut self) -> Result<(), MctlError> {
        for (name, repo_config) in &self.config.repositories {
            debug!("Initializing repository: {}", name);

            let mut repo = GitRepository::new(
                &repo_config.path,
                repo_config.url.clone(),
                repo_config.branch.clone(),
                repo_config.auth_method.clone(),
            );

            // Set credentials if available
            if let Some(creds) = &self.credentials {
                repo.set_credentials(creds.clone());
            }

            self.repositories.insert(name.clone(), repo);
        }

        info!("Initialized {} repositories", self.repositories.len());
        Ok(())
    }

    /// Get a repository by name
    pub fn get_repository(&self, name: &str) -> Result<&GitRepository, MctlError> {
        self.repositories.get(name).ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::RepositoryNotFound,
                format!("Repository '{}' not found", name),
                "".to_string(),
            )
            .into()
        })
    }

    /// Get a mutable repository by name
    pub fn get_repository_mut(&mut self, name: &str) -> Result<&mut GitRepository, MctlError> {
        self.repositories.get_mut(name).ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::RepositoryNotFound,
                format!("Repository '{}' not found", name),
                "".to_string(),
            )
            .into()
        })
    }

    /// Get all repository names
    pub fn get_repository_names(&self) -> Vec<String> {
        self.config.repositories.keys().cloned().collect()
    }

    /// Add a new repository
    pub fn add_repository(
        &mut self,
        name: String,
        url: String,
        path: PathBuf,
        branch: Option<String>,
        auth_method: Option<String>,
    ) -> Result<(), MctlError> {
        // Check if repository already exists
        if self.config.repositories.contains_key(&name) {
            return Err(crate::error::types::ConfigError::new(
                ErrorCode::InvalidConfigFormat,
                format!("Repository '{}' already exists", name),
                "".to_string(),
            )
            .into());
        }

        // Create repository configuration
        let mut repo_config = Repository::new(url.clone(), path.clone(), branch.clone());

        // Set authentication method
        if let Some(auth) = auth_method.clone() {
            repo_config.auth_method = Some(auth);
        } else {
            repo_config.detect_auth_method();
        }

        // Add to configuration
        self.config.add_repository(name.clone(), repo_config)?;

        // Create repository instance
        let mut repo = GitRepository::new(&path, url, branch, auth_method);

        // Set credentials if available
        if let Some(creds) = &self.credentials {
            repo.set_credentials(creds.clone());
        }

        // Add to repositories map
        self.repositories.insert(name, repo);

        // Save configuration
        self.save_config()?;

        Ok(())
    }

    /// Remove a repository
    pub fn remove_repository(&mut self, name: &str) -> Result<(), MctlError> {
        // Remove from configuration
        self.config.remove_repository(name)?;

        // Remove from repositories map
        self.repositories.remove(name);

        // Save configuration
        self.save_config()?;

        Ok(())
    }

    /// Save the configuration
    pub fn save_config(&self) -> Result<(), MctlError> {
        self.config.save(&self.config_path)
    }

    /// Clone all repositories
    pub fn clone_all(&mut self) -> Result<(), MctlError> {
        for (name, repo) in &mut self.repositories {
            debug!("Cloning repository: {}", name);

            match repo.clone() {
                Ok(_) => info!("Successfully cloned repository: {}", name),
                Err(e) => {
                    warn!("Failed to clone repository {}: {}", name, e);
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    /// Clone a specific repository
    pub fn clone_repository(&mut self, name: &str) -> Result<(), MctlError> {
        let repo = self.get_repository_mut(name)?;
        repo.clone()?;
        Ok(())
    }

    /// Get the configuration
    pub fn get_config(&self) -> &MirrorConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn get_config_mut(&mut self) -> &mut MirrorConfig {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new_repository_manager() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("mirror.toml");

        // Create a configuration
        let mut config = MirrorConfig::new();

        let repo = Repository::new(
            "git@github.com:example/repo.git".to_string(),
            PathBuf::from("example-repo"),
            Some("main".to_string()),
        );

        config.add_repository("example".to_string(), repo).unwrap();
        config.save(&config_path).unwrap();

        // Create a repository manager
        let manager = RepositoryManager::new(&config_path).unwrap();

        assert_eq!(manager.get_repository_names().len(), 1);
        assert!(manager
            .get_repository_names()
            .contains(&"example".to_string()));
    }
}
