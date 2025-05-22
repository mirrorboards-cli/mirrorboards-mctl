//! Repository configuration model.

use serde::{Deserialize, Serialize};

use crate::error::ValidationError;

/// Represents a single repository configuration in mirror.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    /// Optional unique identifier for the repository.
    pub id: Option<String>,
    
    /// Git repository origin URL.
    pub origin: String,
    
    /// Git branch to use.
    pub branch: String,
    
    /// Whether the branch is locked (cannot be changed).
    #[serde(default)]
    #[serde(rename = "branch-lock")]
    pub branch_lock: bool,
    
    /// Local filesystem path where the repository should be cloned.
    pub path: String,
    
    /// Optional tags for categorizing repositories.
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Repository {
    /// Validates the repository configuration.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Check for required fields
        if self.origin.is_empty() {
            return Err(ValidationError::MissingField("origin".to_string()));
        }
        
        if self.branch.is_empty() {
            return Err(ValidationError::MissingField("branch".to_string()));
        }
        
        if self.path.is_empty() {
            return Err(ValidationError::MissingField("path".to_string()));
        }
        
        // Validate origin format (basic check)
        if !self.origin.contains(':') {
            return Err(ValidationError::InvalidOrigin(self.origin.clone()));
        }
        
        // Validate path (basic check)
        if self.path.contains("..") {
            return Err(ValidationError::InvalidPath(self.path.clone()));
        }
        
        Ok(())
    }
}

/// Builder for Repository.
pub struct RepositoryBuilder {
    origin: Option<String>,
    branch: Option<String>,
    path: Option<String>,
    id: Option<String>,
    branch_lock: bool,
    tags: Vec<String>,
}

impl RepositoryBuilder {
    /// Create a new repository builder.
    pub fn new() -> Self {
        Self {
            origin: None,
            branch: None,
            path: None,
            id: None,
            branch_lock: false,
            tags: Vec::new(),
        }
    }
    
    /// Set the repository origin.
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }
    
    /// Set the repository branch.
    pub fn branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }
    
    /// Set the repository path.
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    
    /// Set the repository ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Set branch lock.
    pub fn branch_lock(mut self, lock: bool) -> Self {
        self.branch_lock = lock;
        self
    }
    
    /// Add a tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Add multiple tags.
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.push(tag.into());
        }
        self
    }
    
    /// Build the repository.
    pub fn build(self) -> Result<Repository, ValidationError> {
        let origin = self.origin.ok_or_else(|| ValidationError::MissingField("origin".to_string()))?;
        let branch = self.branch.ok_or_else(|| ValidationError::MissingField("branch".to_string()))?;
        let path = self.path.ok_or_else(|| ValidationError::MissingField("path".to_string()))?;
        
        let repo = Repository {
            id: self.id,
            origin,
            branch,
            branch_lock: self.branch_lock,
            path,
            tags: self.tags,
        };
        
        // Validate the repository
        repo.validate()?;
        
        Ok(repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_builder() {
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .id("test-repo")
            .branch_lock(true)
            .tag("example")
            .build()
            .unwrap();
        
        assert_eq!(repo.origin, "git@github.com:example/repo.git");
        assert_eq!(repo.branch, "main");
        assert_eq!(repo.path, "example/repo");
        assert_eq!(repo.id, Some("test-repo".to_string()));
        assert_eq!(repo.branch_lock, true);
        assert_eq!(repo.tags, vec!["example".to_string()]);
    }
    
    #[test]
    fn test_repository_validation() {
        // Missing origin
        let result = RepositoryBuilder::new()
            .branch("main")
            .path("example/repo")
            .build();
        assert!(result.is_err());
        
        // Missing branch
        let result = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .path("example/repo")
            .build();
        assert!(result.is_err());
        
        // Missing path
        let result = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .build();
        assert!(result.is_err());
        
        // Invalid origin
        let result = RepositoryBuilder::new()
            .origin("invalid-origin")
            .branch("main")
            .path("example/repo")
            .build();
        assert!(result.is_err());
        
        // Invalid path
        let result = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("../example/repo")
            .build();
        assert!(result.is_err());
    }
}