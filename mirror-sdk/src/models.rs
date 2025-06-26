use serde::{Deserialize, Serialize};
use crate::error::{RepositoryError, RepositoryResult};
use crate::hash::generate_hash;
use crate::url_parser::extract_path_from_url;

/// Represents a git repository configuration in mirror.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    /// Git URL (SSH or HTTPS format)
    pub git: String,
    /// Local path for the repository
    pub path: String,
    /// Branch to track (defaults to "main")
    #[serde(default = "default_branch")]
    pub branch: String,
    /// Whether to skip pushing to this repository (defaults to false)
    #[serde(default = "default_skip_push", rename = "skip-push")]
    pub skip_push: bool,
}

/// Root configuration structure for mirror.toml
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct MirrorConfig {
    /// List of repository configurations
    pub repositories: Vec<Repository>,
}

impl Repository {
    /// Create a new Repository from a git URL with default values
    pub fn from_url(git_url: String) -> RepositoryResult<Self> {
        let path = extract_path_from_url(&git_url)?;
        Ok(Repository {
            git: git_url,
            path,
            branch: default_branch(),
            skip_push: default_skip_push(),
        })
    }
    
    /// Create a new Repository with all fields specified
    pub fn new(git: String, path: String, branch: Option<String>, skip_push: Option<bool>) -> Self {
        Repository {
            git,
            path,
            branch: branch.unwrap_or_else(default_branch),
            skip_push: skip_push.unwrap_or_else(default_skip_push),
        }
    }
    
    /// Compute a unique hash for this repository based on all metadata
    pub fn compute_hash(&self) -> String {
        generate_hash(self)
    }
    
    /// Validate the repository configuration
    pub fn validate(&self) -> RepositoryResult<()> {
        // Validate git URL is not empty
        if self.git.trim().is_empty() {
            return Err(RepositoryError::InvalidUrl { 
                url: self.git.clone() 
            });
        }
        
        // Validate path is not empty and doesn't contain invalid characters
        if self.path.trim().is_empty() {
            return Err(RepositoryError::InvalidPath { 
                path: self.path.clone() 
            });
        }
        
        // Basic path validation - dangerous characters only (absolute paths are allowed but warned about)
        if self.path.contains("..") {
            return Err(RepositoryError::InvalidPath {
                path: self.path.clone()
            });
        }
        
        // Validate branch name doesn't contain invalid characters
        if self.branch.trim().is_empty() || 
           self.branch.contains(' ') || 
           self.branch.contains('\n') || 
           self.branch.contains('\t') {
            return Err(RepositoryError::InvalidBranch { 
                branch: self.branch.clone() 
            });
        }
        
        Ok(())
    }
}

impl MirrorConfig {
    /// Create a new empty mirror configuration
    pub fn new() -> Self {
        MirrorConfig {
            repositories: Vec::new(),
        }
    }
    
    /// Create a mirror configuration with the given repositories
    pub fn with_repositories(repositories: Vec<Repository>) -> Self {
        MirrorConfig { repositories }
    }
    
    /// Add a repository to the configuration
    /// Returns an error if a repository with the same hash already exists
    pub fn add_repository(&mut self, repo: Repository) -> RepositoryResult<()> {
        repo.validate()?;
        
        let new_hash = repo.compute_hash();
        
        // Check for duplicates by hash
        if self.repositories.iter().any(|r| r.compute_hash() == new_hash) {
            return Err(RepositoryError::InvalidUrl { 
                url: format!("Repository with hash '{}' already exists", new_hash)
            });
        }
        
        self.repositories.push(repo);
        Ok(())
    }
    
    /// Remove a repository by its hash
    /// Returns the removed repository if found
    pub fn remove_repository(&mut self, hash: &str) -> Option<Repository> {
        if let Some(pos) = self.repositories.iter().position(|r| r.compute_hash().starts_with(hash)) {
            Some(self.repositories.remove(pos))
        } else {
            None
        }
    }
    
    /// Find a repository by its hash (supports partial matching)
    pub fn find_by_hash(&self, hash: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.compute_hash().starts_with(hash))
    }
    
    /// Find a repository by its git URL
    pub fn find_by_git_url(&self, git_url: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.git == git_url)
    }
    
    /// Get all repositories as a slice
    pub fn repositories(&self) -> &[Repository] {
        &self.repositories
    }
    
    /// Get the number of repositories
    pub fn len(&self) -> usize {
        self.repositories.len()
    }
    
    /// Check if the configuration is empty
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty()
    }
    
    /// Validate all repositories in the configuration
    pub fn validate(&self) -> RepositoryResult<()> {
        for repo in &self.repositories {
            repo.validate()?;
        }
        Ok(())
    }
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Default branch name for repositories
fn default_branch() -> String {
    "main".to_string()
}

/// Default skip-push value for repositories
fn default_skip_push() -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_from_url() {
        let repo = Repository::from_url("git@github.com:org/repo.git".to_string()).unwrap();
        assert_eq!(repo.git, "git@github.com:org/repo.git");
        assert_eq!(repo.path, "org/repo");
        assert_eq!(repo.branch, "main");
        assert_eq!(repo.skip_push, false);
    }
    
    #[test]
    fn test_repository_validation() {
        let mut repo = Repository::new(
            "git@github.com:org/repo.git".to_string(),
            "org/repo".to_string(),
            None,
            None,
        );
        assert!(repo.validate().is_ok());
        
        // Test empty git URL
        repo.git = "".to_string();
        assert!(repo.validate().is_err());
        
        // Test invalid path with dangerous characters
        repo.git = "git@github.com:org/repo.git".to_string();
        repo.path = "valid/../invalid".to_string();
        assert!(repo.validate().is_err());
        
        // Absolute paths are now allowed (but may generate warnings in CLI)
        repo.path = "/absolute/path".to_string();
        assert!(repo.validate().is_ok());
    }
    
    #[test]
    fn test_mirror_config_operations() {
        let mut config = MirrorConfig::new();
        assert!(config.is_empty());
        
        let repo = Repository::from_url("git@github.com:org/repo.git".to_string()).unwrap();
        config.add_repository(repo.clone()).unwrap();
        
        assert_eq!(config.len(), 1);
        assert!(!config.is_empty());
        
        let hash = repo.compute_hash();
        let found = config.find_by_hash(&hash[..4]);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &repo);
        
        let removed = config.remove_repository(&hash[..4]);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap(), repo);
        assert!(config.is_empty());
    }
}