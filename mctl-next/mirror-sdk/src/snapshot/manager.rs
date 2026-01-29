use std::path::{Path, PathBuf};

use crate::error::{MirrorError, Result};
use crate::git::GitManager;
use crate::models::{MirrorConfig, Snapshot};

/// Manager for snapshot operations
pub struct SnapshotManager {
    snapshots_dir: PathBuf,
}

impl SnapshotManager {
    /// Create a new snapshot manager with a snapshots directory
    pub fn new(snapshots_dir: impl AsRef<Path>) -> Self {
        Self {
            snapshots_dir: snapshots_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a snapshot of current repository states
    pub fn create(
        &self,
        name: &str,
        config: &MirrorConfig,
        git_manager: &GitManager,
        description: Option<&str>,
    ) -> Result<Snapshot> {
        let mut snapshot = Snapshot::new(name);

        if let Some(desc) = description {
            snapshot = snapshot.with_description(desc);
        }

        for repo in &config.repositories {
            if git_manager.exists(repo) {
                let sha = git_manager.get_head_sha(repo)?;
                snapshot.add_repository(repo.git.clone(), repo.path.clone(), sha);
            }
        }

        Ok(snapshot)
    }

    /// Save a snapshot to file
    pub fn save(&self, snapshot: &Snapshot) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.snapshots_dir)?;

        let filename = format!("{}.snapshot.toml", snapshot.name);
        let path = self.snapshots_dir.join(&filename);

        let content = toml::to_string_pretty(snapshot)?;
        std::fs::write(&path, content)?;

        Ok(path)
    }

    /// Load a snapshot from file
    pub fn load(&self, name: &str) -> Result<Snapshot> {
        let filename = format!("{}.snapshot.toml", name);
        let path = self.snapshots_dir.join(&filename);

        if !path.exists() {
            return Err(MirrorError::SnapshotNotFound(name.to_string()));
        }

        let content = std::fs::read_to_string(&path)?;
        let snapshot: Snapshot = toml::from_str(&content)?;

        Ok(snapshot)
    }

    /// List all available snapshots
    pub fn list(&self) -> Result<Vec<SnapshotInfo>> {
        if !self.snapshots_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();

        for entry in std::fs::read_dir(&self.snapshots_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                    if filename.ends_with(".snapshot.toml") {
                        match self.load_snapshot_info(&path) {
                            Ok(info) => snapshots.push(info),
                            Err(_) => continue,
                        }
                    }
                }
            }
        }

        // Sort by created_at descending
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(snapshots)
    }

    /// Load snapshot metadata without full repository list
    fn load_snapshot_info(&self, path: &Path) -> Result<SnapshotInfo> {
        let content = std::fs::read_to_string(path)?;
        let snapshot: Snapshot = toml::from_str(&content)?;

        Ok(SnapshotInfo {
            name: snapshot.name,
            created_at: snapshot.created_at,
            description: snapshot.description,
            repository_count: snapshot.repositories.len(),
        })
    }

    /// Delete a snapshot
    pub fn delete(&self, name: &str) -> Result<()> {
        let filename = format!("{}.snapshot.toml", name);
        let path = self.snapshots_dir.join(&filename);

        if !path.exists() {
            return Err(MirrorError::SnapshotNotFound(name.to_string()));
        }

        std::fs::remove_file(path)?;
        Ok(())
    }
}

/// Summary information about a snapshot
#[derive(Debug, Clone)]
pub struct SnapshotInfo {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    pub repository_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_snapshot(name: &str) -> Snapshot {
        let mut snapshot = Snapshot::new(name);
        snapshot.add_repository(
            "git@github.com:test/repo.git".to_string(),
            "repo".to_string(),
            "abc123".to_string(),
        );
        snapshot
    }

    #[test]
    fn test_save_and_load_snapshot() {
        let dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(dir.path());

        let snapshot = create_test_snapshot("test-snap");
        manager.save(&snapshot).unwrap();

        let loaded = manager.load("test-snap").unwrap();
        assert_eq!(loaded.name, "test-snap");
        assert_eq!(loaded.repositories.len(), 1);
    }

    #[test]
    fn test_list_snapshots() {
        let dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(dir.path());

        manager.save(&create_test_snapshot("snap1")).unwrap();
        manager.save(&create_test_snapshot("snap2")).unwrap();

        let list = manager.list().unwrap();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_delete_snapshot() {
        let dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(dir.path());

        manager.save(&create_test_snapshot("to-delete")).unwrap();
        assert!(manager.load("to-delete").is_ok());

        manager.delete("to-delete").unwrap();
        assert!(manager.load("to-delete").is_err());
    }

    #[test]
    fn test_snapshot_not_found() {
        let dir = TempDir::new().unwrap();
        let manager = SnapshotManager::new(dir.path());

        let result = manager.load("nonexistent");
        assert!(matches!(result, Err(MirrorError::SnapshotNotFound(_))));
    }
}
