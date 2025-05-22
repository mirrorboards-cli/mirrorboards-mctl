//! Repository module for mirror-sdk
//!
//! This module provides the data structure and functions for working with
//! repository configurations in mirror.toml files.

use serde::{Deserialize, Serialize};
use crate::error::Error;
use crate::utils::generate_id;

/// Represents a repository configuration in mirror.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    /// Repository ID (optional, auto-generated if not provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    
    /// Git repository URL (required)
    pub origin: String,
    
    /// Git branch (optional, defaults to "main")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    
    /// Local path where the repository should be cloned (required)
    pub path: String,
    
    /// Whether the repository is locked (optional, defaults to false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock: Option<bool>,
    
    /// Tags for the repository (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl Repository {
    /// Creates a new repository configuration with the given origin and path
    ///
    /// # Arguments
    ///
    /// * `origin` - Git repository URL
    /// * `path` - Local path where the repository should be cloned
    ///
    /// # Returns
    ///
    /// A new `Repository` instance with default values for optional fields
    ///
    /// # Example
    ///
    /// ```
    /// use mirror_sdk::repository::Repository;
    ///
    /// let repo = Repository::new(
    ///     "git@github.com:mirrorboards/example-repo.git",
    ///     "example/path",
    /// ).unwrap();
    /// ```
    pub fn new<S: Into<String>>(origin: S, path: S) -> Result<Self, Error> {
        let origin = origin.into();
        let path = path.into();
        
        if origin.is_empty() {
            return Err(Error::MissingField("origin".to_string()));
        }
        
        if path.is_empty() {
            return Err(Error::MissingField("path".to_string()));
        }
        
        Ok(Self {
            id: None,
            origin,
            branch: None,
            path,
            lock: None,
            tags: None,
        })
    }
    
    /// Sets the repository ID
    ///
    /// # Arguments
    ///
    /// * `id` - Repository ID
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Sets the repository branch
    ///
    /// # Arguments
    ///
    /// * `branch` - Git branch
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_branch<S: Into<String>>(mut self, branch: S) -> Self {
        self.branch = Some(branch.into());
        self
    }
    
    /// Sets whether the repository is locked
    ///
    /// # Arguments
    ///
    /// * `lock` - Whether the repository is locked
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_lock(mut self, lock: bool) -> Self {
        self.lock = Some(lock);
        self
    }
    
    /// Sets the repository tags
    ///
    /// # Arguments
    ///
    /// * `tags` - Repository tags
    ///
    /// # Returns
    ///
    /// Self for method chaining
    pub fn with_tags<S: Into<String>>(mut self, tags: Vec<S>) -> Self {
        self.tags = Some(tags.into_iter().map(|s| s.into()).collect());
        self
    }
    
    /// Gets the repository ID, generating one if it doesn't exist
    ///
    /// # Returns
    ///
    /// The repository ID
    pub fn get_id(&mut self) -> String {
        if self.id.is_none() {
            self.id = Some(generate_id());
        }
        
        self.id.clone().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_repository_new() {
        let repo = Repository::new("git@github.com:mirrorboards/example-repo.git", "example/path").unwrap();
        
        assert_eq!(repo.origin, "git@github.com:mirrorboards/example-repo.git");
        assert_eq!(repo.path, "example/path");
        assert_eq!(repo.id, None);
        assert_eq!(repo.branch, None);
        assert_eq!(repo.lock, None);
        assert_eq!(repo.tags, None);
    }
    
    #[test]
    fn test_repository_with_id() {
        let repo = Repository::new("git@github.com:mirrorboards/example-repo.git", "example/path")
            .unwrap()
            .with_id("test-id");
        
        assert_eq!(repo.id, Some("test-id".to_string()));
    }
    
    #[test]
    fn test_repository_get_id() {
        let mut repo = Repository::new("git@github.com:mirrorboards/example-repo.git", "example/path").unwrap();
        
        let id = repo.get_id();
        assert!(!id.is_empty());
        assert_eq!(repo.id, Some(id));
    }
}