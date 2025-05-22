//! Repository configuration for mirror.toml.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

use crate::error::MirrorError;
use crate::Result;

/// Represents a repository configuration in mirror.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Repository {
    /// The unique identifier for the repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The Git repository origin URL.
    pub origin: String,

    /// The branch to use (defaults to "main" if not specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// The local path where the repository should be cloned.
    pub path: String,

    /// Whether the repository is locked (defaults to false if not specified).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock: Option<bool>,

    /// Tags associated with the repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashSet<String>>,
}

impl Repository {
    /// Creates a new repository builder.
    pub fn new() -> RepositoryBuilder {
        RepositoryBuilder::default()
    }

    /// Generates a new unique ID for the repository.
    pub fn generate_id() -> String {
        Uuid::new_v4().to_string().split('-').next().unwrap().to_string()
    }

    /// Returns the ID of the repository, or generates a new one if not set.
    pub fn get_or_generate_id(&self) -> String {
        self.id.clone().unwrap_or_else(Self::generate_id)
    }

    /// Returns true if the repository is locked.
    pub fn is_locked(&self) -> bool {
        self.lock.unwrap_or(false)
    }

    /// Returns the branch of the repository, or "main" if not set.
    pub fn get_branch(&self) -> &str {
        self.branch.as_deref().unwrap_or("main")
    }

    /// Returns the tags of the repository, or an empty set if not set.
    pub fn get_tags(&self) -> HashSet<String> {
        self.tags.clone().unwrap_or_default()
    }

    /// Returns true if the repository has the given tag.
    pub fn has_tag(&self, tag: &str) -> bool {
        self.get_tags().contains(tag)
    }
}

/// Builder for creating Repository instances.
#[derive(Debug, Default)]
pub struct RepositoryBuilder {
    id: Option<String>,
    origin: Option<String>,
    branch: Option<String>,
    path: Option<String>,
    lock: Option<bool>,
    tags: Option<HashSet<String>>,
}

impl RepositoryBuilder {
    /// Sets the ID for the repository.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the origin for the repository.
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    /// Sets the branch for the repository.
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// Sets the path for the repository.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Sets whether the repository is locked.
    pub fn with_lock(mut self, lock: bool) -> Self {
        self.lock = Some(lock);
        self
    }

    /// Adds a tag to the repository.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let tag = tag.into();
        let tags = self.tags.get_or_insert_with(HashSet::new);
        tags.insert(tag);
        self
    }

    /// Adds multiple tags to the repository.
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let tags_set = self.tags.get_or_insert_with(HashSet::new);
        for tag in tags {
            tags_set.insert(tag.into());
        }
        self
    }

    /// Builds the repository.
    pub fn build(self) -> Result<Repository> {
        let origin = self.origin.ok_or(MirrorError::MissingOrigin)?;
        let path = self.path.ok_or(MirrorError::MissingPath)?;

        Ok(Repository {
            id: self.id,
            origin,
            branch: self.branch,
            path,
            lock: self.lock,
            tags: self.tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repository_builder() {
        let repo = Repository::new()
            .with_id("test-repo")
            .with_origin("git@github.com:example/repo.git")
            .with_path("./example/repo")
            .with_branch("main")
            .with_lock(true)
            .with_tag("test")
            .with_tags(["tag1", "tag2"])
            .build()
            .unwrap();

        assert_eq!(repo.id, Some("test-repo".to_string()));
        assert_eq!(repo.origin, "git@github.com:example/repo.git");
        assert_eq!(repo.path, "./example/repo");
        assert_eq!(repo.branch, Some("main".to_string()));
        assert_eq!(repo.lock, Some(true));
        
        let expected_tags: HashSet<String> = ["test", "tag1", "tag2"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(repo.tags, Some(expected_tags));
    }

    #[test]
    fn test_missing_origin() {
        let result = Repository::new()
            .with_path("./example/repo")
            .build();

        assert!(matches!(result, Err(MirrorError::MissingOrigin)));
    }

    #[test]
    fn test_missing_path() {
        let result = Repository::new()
            .with_origin("git@github.com:example/repo.git")
            .build();

        assert!(matches!(result, Err(MirrorError::MissingPath)));
    }

    #[test]
    fn test_get_or_generate_id() {
        let repo_with_id = Repository {
            id: Some("test-id".to_string()),
            origin: "git@github.com:example/repo.git".to_string(),
            branch: None,
            path: "./example/repo".to_string(),
            lock: None,
            tags: None,
        };

        let repo_without_id = Repository {
            id: None,
            origin: "git@github.com:example/repo.git".to_string(),
            branch: None,
            path: "./example/repo".to_string(),
            lock: None,
            tags: None,
        };

        assert_eq!(repo_with_id.get_or_generate_id(), "test-id");
        assert!(!repo_without_id.get_or_generate_id().is_empty());
    }
}