use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Repository state within a snapshot
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SnapshotRepository {
    /// Git URL of the repository
    pub git: String,

    /// Local path of the repository
    pub path: String,

    /// Commit SHA at snapshot time
    pub rev: String,
}

/// Snapshot of all repository states
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Snapshot {
    /// Snapshot name/identifier
    pub name: String,

    /// When the snapshot was created
    pub created_at: DateTime<Utc>,

    /// Optional description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// State of each repository
    pub repositories: Vec<SnapshotRepository>,
}

impl Snapshot {
    /// Create a new snapshot with the current timestamp
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            created_at: Utc::now(),
            description: None,
            repositories: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a repository to the snapshot
    pub fn add_repository(&mut self, git: String, path: String, rev: String) {
        self.repositories.push(SnapshotRepository { git, path, rev });
    }

    /// Find a repository by path
    pub fn find_by_path(&self, path: &str) -> Option<&SnapshotRepository> {
        self.repositories.iter().find(|r| r.path == path)
    }

    /// Find a repository by git URL
    pub fn find_by_git(&self, git: &str) -> Option<&SnapshotRepository> {
        self.repositories.iter().find(|r| r.git == git)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let mut snapshot = Snapshot::new("test-snapshot");
        snapshot.add_repository(
            "git@github.com:test/repo.git".to_string(),
            "test/repo".to_string(),
            "abc123".to_string(),
        );

        assert_eq!(snapshot.name, "test-snapshot");
        assert_eq!(snapshot.repositories.len(), 1);
        assert!(snapshot.find_by_path("test/repo").is_some());
    }

    #[test]
    fn test_snapshot_with_description() {
        let snapshot = Snapshot::new("test").with_description("Test snapshot");
        assert_eq!(snapshot.description, Some("Test snapshot".to_string()));
    }
}
