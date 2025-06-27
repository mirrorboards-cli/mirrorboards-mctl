use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::{ConfigError, RepositoryError, RepositoryResult};
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
    /// Returns an error if a repository with the same hash already exists or if there's a path conflict
    pub fn add_repository(&mut self, repo: Repository) -> Result<(), crate::error::MirrorSdkError> {
        repo.validate()?;
        
        let new_hash = repo.compute_hash();
        
        // Check for duplicates by hash
        if let Some(existing_repo) = self.repositories.iter().find(|r| r.compute_hash() == new_hash) {
            return Err(ConfigError::DuplicateRepository {
                hash: new_hash,
                existing_git: existing_repo.git.clone()
            }.into());
        }
        
        // Check for path conflicts
        if let Some(existing_repo) = self.has_path_conflict(&repo.path) {
            return Err(ConfigError::PathConflict {
                path: repo.path.clone(),
                existing_git: existing_repo.git.clone(),
                new_git: repo.git.clone(),
            }.into());
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
    
    /// Check if a given path conflicts with any existing repository paths
    /// Returns the conflicting repository if found, None otherwise
    pub fn has_path_conflict(&self, path: &str) -> Option<&Repository> {
        let normalized_path = Self::normalize_path(path);
        
        self.repositories.iter().find(|repo| {
            let existing_normalized = Self::normalize_path(&repo.path);
            existing_normalized == normalized_path
        })
    }
    
    /// Check for path conflicts among all repositories in the configuration
    /// Returns an error if any path conflicts are found
    pub fn check_path_conflicts(&self) -> Result<(), crate::error::MirrorSdkError> {
        for (i, repo) in self.repositories.iter().enumerate() {
            let normalized_path = Self::normalize_path(&repo.path);
            
            // Check against all subsequent repositories to avoid duplicate checking
            for other_repo in self.repositories.iter().skip(i + 1) {
                let other_normalized = Self::normalize_path(&other_repo.path);
                
                if normalized_path == other_normalized {
                    return Err(ConfigError::PathConflict {
                        path: repo.path.clone(),
                        existing_git: repo.git.clone(),
                        new_git: other_repo.git.clone(),
                    }.into());
                }
            }
        }
        Ok(())
    }
    
    /// Normalize a path for comparison by resolving relative components and removing trailing slashes
    /// This helps detect conflicts between paths like "repo" and "repo/" or "./repo" and "repo"
    fn normalize_path(path: &str) -> String {
        let path = path.trim();
        
        // Handle empty path
        if path.is_empty() {
            return String::new();
        }
        
        // Convert to Path for normalization
        let path_buf = Path::new(path);
        
        // Get components and rebuild the path, resolving . and removing trailing separators
        let mut components = Vec::new();
        for component in path_buf.components() {
            match component {
                std::path::Component::Normal(part) => {
                    components.push(part.to_string_lossy().to_string());
                }
                std::path::Component::CurDir => {
                    // Skip "." components
                    continue;
                }
                std::path::Component::ParentDir => {
                    // Keep ".." components as they're already validated as invalid in Repository::validate()
                    components.push("..".to_string());
                }
                std::path::Component::RootDir => {
                    // Preserve root for absolute paths
                    components.insert(0, String::new()); // This will create a leading slash when joined
                }
                std::path::Component::Prefix(_) => {
                    // Windows-specific, preserve as-is
                    components.push(component.as_os_str().to_string_lossy().to_string());
                }
            }
        }
        
        // Join components back together
        let result = if components.first().map_or(false, |c| c.is_empty()) {
            // This was an absolute path, join with leading slash
            format!("/{}", components[1..].join("/"))
        } else {
            components.join("/")
        };
        
        // Remove trailing slash unless it's the root path
        if result.len() > 1 && result.ends_with('/') {
            result.trim_end_matches('/').to_string()
        } else {
            result
        }
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
    
    #[test]
    fn test_path_normalization() {
        assert_eq!(MirrorConfig::normalize_path("repo"), "repo");
        assert_eq!(MirrorConfig::normalize_path("repo/"), "repo");
        assert_eq!(MirrorConfig::normalize_path("./repo"), "repo");
        assert_eq!(MirrorConfig::normalize_path("./repo/"), "repo");
        assert_eq!(MirrorConfig::normalize_path("/absolute/path"), "/absolute/path");
        assert_eq!(MirrorConfig::normalize_path("/absolute/path/"), "/absolute/path");
        assert_eq!(MirrorConfig::normalize_path("org/repo"), "org/repo");
        assert_eq!(MirrorConfig::normalize_path("org/repo/"), "org/repo");
        assert_eq!(MirrorConfig::normalize_path("./org/repo"), "org/repo");
        assert_eq!(MirrorConfig::normalize_path(""), "");
        assert_eq!(MirrorConfig::normalize_path("   "), "");
        assert_eq!(MirrorConfig::normalize_path("/"), "/");
    }
    
    #[test]
    fn test_path_conflict_detection() {
        let mut config = MirrorConfig::new();
        
        // Add first repository
        let repo1 = Repository::new(
            "git@github.com:org/repo1.git".to_string(),
            "shared/path".to_string(),
            None,
            None,
        );
        config.add_repository(repo1).unwrap();
        
        // Try to add second repository with same path - should fail
        let repo2 = Repository::new(
            "git@github.com:org/repo2.git".to_string(),
            "shared/path".to_string(),
            None,
            None,
        );
        let result = config.add_repository(repo2);
        assert!(result.is_err());
        
        // Try to add repository with normalized equivalent path - should also fail
        let repo3 = Repository::new(
            "git@github.com:org/repo3.git".to_string(),
            "shared/path/".to_string(),
            None,
            None,
        );
        let result = config.add_repository(repo3);
        assert!(result.is_err());
        
        // Try to add repository with different path - should succeed
        let repo4 = Repository::new(
            "git@github.com:org/repo4.git".to_string(),
            "different/path".to_string(),
            None,
            None,
        );
        assert!(config.add_repository(repo4).is_ok());
        assert_eq!(config.len(), 2);
    }
    
    #[test]
    fn test_has_path_conflict() {
        let mut config = MirrorConfig::new();
        
        let repo = Repository::new(
            "git@github.com:org/repo.git".to_string(),
            "existing/path".to_string(),
            None,
            None,
        );
        config.add_repository(repo.clone()).unwrap();
        
        // Test exact match
        assert!(config.has_path_conflict("existing/path").is_some());
        
        // Test normalized equivalent
        assert!(config.has_path_conflict("existing/path/").is_some());
        assert!(config.has_path_conflict("./existing/path").is_some());
        
        // Test no conflict
        assert!(config.has_path_conflict("different/path").is_none());
        assert!(config.has_path_conflict("existing/different").is_none());
    }
    
    #[test]
    fn test_check_path_conflicts() {
        // Test configuration with no conflicts
        let config = MirrorConfig::with_repositories(vec![
            Repository::new(
                "git@github.com:org/repo1.git".to_string(),
                "path1".to_string(),
                None,
                None,
            ),
            Repository::new(
                "git@github.com:org/repo2.git".to_string(),
                "path2".to_string(),
                None,
                None,
            ),
        ]);
        assert!(config.check_path_conflicts().is_ok());
        
        // Test configuration with conflicts
        let config_with_conflicts = MirrorConfig::with_repositories(vec![
            Repository::new(
                "git@github.com:org/repo1.git".to_string(),
                "shared".to_string(),
                None,
                None,
            ),
            Repository::new(
                "git@github.com:org/repo2.git".to_string(),
                "shared/".to_string(),
                None,
                None,
            ),
        ]);
        assert!(config_with_conflicts.check_path_conflicts().is_err());
    }
    
    #[test]
    fn test_absolute_vs_relative_paths() {
        let mut config = MirrorConfig::new();
        
        // Add repository with relative path
        let repo1 = Repository::new(
            "git@github.com:org/repo1.git".to_string(),
            "repo".to_string(),
            None,
            None,
        );
        config.add_repository(repo1).unwrap();
        
        // Try to add repository with absolute path (different from relative)
        let repo2 = Repository::new(
            "git@github.com:org/repo2.git".to_string(),
            "/repo".to_string(),
            None,
            None,
        );
        assert!(config.add_repository(repo2).is_ok());
        
        // Both should coexist as they're different paths
        assert_eq!(config.len(), 2);
    }
}