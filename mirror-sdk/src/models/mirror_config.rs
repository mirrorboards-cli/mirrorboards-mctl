//! Mirror configuration model.

use serde::{Deserialize, Serialize};

use crate::error::ValidationError;
use crate::models::repository::Repository;

/// Represents the entire mirror.toml configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MirrorConfig {
    /// List of repository configurations.
    pub repositories: Vec<Repository>,
}

impl MirrorConfig {
    /// Creates a new empty mirror configuration.
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
        }
    }

    /// Validates the entire configuration.
    pub fn validate(&self) -> Result<(), ValidationError> {
        // Validate each repository
        for repo in &self.repositories {
            repo.validate()?;
        }

        // Check for duplicate IDs
        let mut ids = Vec::new();
        for repo in &self.repositories {
            if let Some(id) = &repo.id {
                if ids.contains(id) {
                    return Err(ValidationError::DuplicateId(id.clone()));
                }
                ids.push(id.clone());
            }
        }

        // Check for path conflicts
        for (i, repo1) in self.repositories.iter().enumerate() {
            for (j, repo2) in self.repositories.iter().enumerate() {
                if i != j && repo1.path == repo2.path {
                    return Err(ValidationError::PathConflict(
                        repo1.path.clone(),
                        repo2.path.clone(),
                    ));
                }
                // Check for path prefix conflicts (one path is a prefix of another)
                if i != j && (repo1.path.starts_with(&repo2.path) || repo2.path.starts_with(&repo1.path)) {
                    // Only report if one path is a direct parent of another (with a trailing slash)
                    let path1 = if !repo1.path.ends_with('/') {
                        format!("{}/", repo1.path)
                    } else {
                        repo1.path.clone()
                    };
                    
                    let path2 = if !repo2.path.ends_with('/') {
                        format!("{}/", repo2.path)
                    } else {
                        repo2.path.clone()
                    };
                    
                    if path1.starts_with(&path2) || path2.starts_with(&path1) {
                        return Err(ValidationError::PathConflict(
                            repo1.path.clone(),
                            repo2.path.clone(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Finds a repository by path.
    pub fn find_by_path(&self, path: &str) -> Option<&Repository> {
        self.repositories.iter().find(|repo| repo.path == path)
    }

    /// Finds a repository by ID.
    pub fn find_by_id(&self, id: &str) -> Option<&Repository> {
        self.repositories
            .iter()
            .find(|repo| repo.id.as_ref().map_or(false, |repo_id| repo_id == id))
    }

    /// Finds repositories by tag.
    pub fn find_by_tag(&self, tag: &str) -> Vec<&Repository> {
        self.repositories
            .iter()
            .filter(|repo| repo.tags.contains(&tag.to_string()))
            .collect()
    }
}

impl Default for MirrorConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::repository::RepositoryBuilder;

    #[test]
    fn test_new_config() {
        let config = MirrorConfig::new();
        assert!(config.repositories.is_empty());
    }

    #[test]
    fn test_find_by_path() {
        let mut config = MirrorConfig::new();
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        config.repositories.push(repo);

        let found = config.find_by_path("example/repo");
        assert!(found.is_some());
        assert_eq!(found.unwrap().origin, "git@github.com:example/repo.git");

        let not_found = config.find_by_path("nonexistent/path");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_id() {
        let mut config = MirrorConfig::new();
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .id("test-repo")
            .build()
            .unwrap();
        config.repositories.push(repo);

        let found = config.find_by_id("test-repo");
        assert!(found.is_some());
        assert_eq!(found.unwrap().origin, "git@github.com:example/repo.git");

        let not_found = config.find_by_id("nonexistent-id");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_tag() {
        let mut config = MirrorConfig::new();
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo1")
            .tag("example")
            .build()
            .unwrap();
        let repo2 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo2.git")
            .branch("main")
            .path("example/repo2")
            .tag("test")
            .build()
            .unwrap();
        config.repositories.push(repo1);
        config.repositories.push(repo2);

        let found = config.find_by_tag("example");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].origin, "git@github.com:example/repo1.git");

        let not_found = config.find_by_tag("nonexistent-tag");
        assert!(not_found.is_empty());
    }

    #[test]
    fn test_validate_duplicate_id() {
        let mut config = MirrorConfig::new();
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo1")
            .id("duplicate-id")
            .build()
            .unwrap();
        let repo2 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo2.git")
            .branch("main")
            .path("example/repo2")
            .id("duplicate-id")
            .build()
            .unwrap();
        config.repositories.push(repo1);
        config.repositories.push(repo2);

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(ValidationError::DuplicateId(id)) => assert_eq!(id, "duplicate-id"),
            _ => panic!("Expected DuplicateId error"),
        }
    }

    #[test]
    fn test_validate_path_conflict() {
        let mut config = MirrorConfig::new();
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        let repo2 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo2.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        config.repositories.push(repo1);
        config.repositories.push(repo2);

        let result = config.validate();
        assert!(result.is_err());
        match result {
            Err(ValidationError::PathConflict(path1, path2)) => {
                assert_eq!(path1, "example/repo");
                assert_eq!(path2, "example/repo");
            }
            _ => panic!("Expected PathConflict error"),
        }
    }
}