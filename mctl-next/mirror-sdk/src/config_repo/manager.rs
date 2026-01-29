use std::path::{Path, PathBuf};

use git2::{Cred, FetchOptions, RemoteCallbacks, Repository as Git2Repository};

use crate::error::{MirrorError, Result};
use crate::models::ConfigRepo;

/// Manager for config repository operations
pub struct ConfigRepoManager {
    config_repo: ConfigRepo,
    local_path: PathBuf,
}

impl ConfigRepoManager {
    /// Create a new config repo manager
    pub fn new(config_repo: ConfigRepo, local_path: impl AsRef<Path>) -> Self {
        Self {
            config_repo,
            local_path: local_path.as_ref().to_path_buf(),
        }
    }

    /// Get the local path for the config repo
    pub fn local_path(&self) -> &Path {
        &self.local_path
    }

    /// Get the path to the config file within the repo
    pub fn config_file_path(&self) -> PathBuf {
        self.local_path.join(&self.config_repo.config_path)
    }

    /// Get the path to the snapshots directory within the repo
    pub fn snapshots_dir(&self) -> PathBuf {
        self.local_path.join(&self.config_repo.snapshots_dir)
    }

    /// Check if the config repo is cloned locally
    pub fn is_cloned(&self) -> bool {
        self.local_path.join(".git").exists()
    }

    /// Clone or update the config repo
    pub fn sync(&self) -> Result<()> {
        if self.is_cloned() {
            self.pull()
        } else {
            self.clone()?;
            Ok(())
        }
    }

    /// Clone the config repo
    fn clone(&self) -> Result<Git2Repository> {
        if let Some(parent) = self.local_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_opts);
        builder.branch(&self.config_repo.branch);

        let repo = builder.clone(&self.config_repo.git, &self.local_path)?;
        Ok(repo)
    }

    /// Pull latest changes
    fn pull(&self) -> Result<()> {
        let repo = Git2Repository::open(&self.local_path)?;

        // Fetch
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut remote = repo.find_remote("origin")?;
        remote.fetch(
            &[&format!("refs/heads/{}", self.config_repo.branch)],
            Some(&mut fetch_opts),
            None,
        )?;

        // Fast-forward merge
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        let fetch_commit = fetch_head.peel_to_commit()?;

        let refname = format!("refs/heads/{}", self.config_repo.branch);
        repo.reference(&refname, fetch_commit.id(), true, "mctl: pull")?;

        repo.checkout_tree(fetch_commit.as_object(), None)?;
        repo.set_head(&refname)?;

        Ok(())
    }

    /// Push changes to remote
    pub fn push(&self) -> Result<()> {
        let repo = Git2Repository::open(&self.local_path)?;

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        let mut remote = repo.find_remote("origin")?;
        let refspec = format!(
            "refs/heads/{}:refs/heads/{}",
            self.config_repo.branch, self.config_repo.branch
        );
        remote.push(&[&refspec], Some(&mut push_opts))?;

        Ok(())
    }

    /// Stage and commit changes
    pub fn commit(&self, message: &str) -> Result<()> {
        let repo = Git2Repository::open(&self.local_path)?;

        // Stage all changes
        let mut index = repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Check if there are changes to commit
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;

        let head = repo.head()?;
        let parent_commit = head.peel_to_commit()?;

        // Check if tree is the same as parent
        if tree.id() == parent_commit.tree()?.id() {
            return Ok(()); // Nothing to commit
        }

        let signature = repo.signature()?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(())
    }

    /// Save config and push
    pub fn save_config(&self, config_content: &str, message: &str) -> Result<()> {
        self.sync()?;

        // Write config file
        let config_path = self.config_file_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&config_path, config_content)?;

        // Commit and push
        self.commit(message)?;
        self.push()?;

        Ok(())
    }

    /// Pull config content
    pub fn pull_config(&self) -> Result<String> {
        self.sync()?;

        let config_path = self.config_file_path();
        let content = std::fs::read_to_string(&config_path).map_err(|e| {
            MirrorError::Config(format!(
                "Failed to read config from {}: {}",
                config_path.display(),
                e
            ))
        })?;

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_paths() {
        let dir = TempDir::new().unwrap();
        let config_repo = ConfigRepo {
            git: "git@github.com:test/config.git".to_string(),
            branch: "main".to_string(),
            config_path: "mirror.toml".to_string(),
            snapshots_dir: "snapshots".to_string(),
        };

        let manager = ConfigRepoManager::new(config_repo, dir.path());

        assert_eq!(
            manager.config_file_path(),
            dir.path().join("mirror.toml")
        );
        assert_eq!(
            manager.snapshots_dir(),
            dir.path().join("snapshots")
        );
    }
}
