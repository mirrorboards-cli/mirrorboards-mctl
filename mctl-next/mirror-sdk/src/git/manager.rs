use std::path::Path;

use git2::{
    build::RepoBuilder, Cred, FetchOptions, RemoteCallbacks, Repository as Git2Repository,
};

use crate::error::{MirrorError, Result};
use crate::models::{RefSpec, Repository};

/// Manager for git operations
pub struct GitManager {
    base_path: std::path::PathBuf,
}

impl GitManager {
    /// Create a new git manager with a base path for repositories
    pub fn new(base_path: impl AsRef<Path>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
        }
    }

    /// Get the full path for a repository
    pub fn repo_path(&self, repo: &Repository) -> std::path::PathBuf {
        self.base_path.join(&repo.path)
    }

    /// Check if a repository exists locally
    pub fn exists(&self, repo: &Repository) -> bool {
        let path = self.repo_path(repo);
        path.exists() && path.join(".git").exists()
    }

    /// Clone or update a repository
    pub fn sync(&self, repo: &Repository) -> Result<SyncResult> {
        if self.exists(repo) {
            self.update(repo).map(|_| SyncResult::Updated)
        } else {
            self.clone(repo).map(|_| SyncResult::Cloned)
        }
    }

    /// Clone a repository
    pub fn clone(&self, repo: &Repository) -> Result<Git2Repository> {
        let path = self.repo_path(repo);

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_opts);

        // Set branch for cloning
        if let RefSpec::Branch(branch) = &repo.ref_spec {
            builder.branch(branch);
        }

        let git_repo = builder.clone(&repo.git, &path)?;

        // For tag or rev, checkout after clone
        match &repo.ref_spec {
            RefSpec::Tag(tag) => {
                self.checkout_tag(&git_repo, tag)?;
            }
            RefSpec::Rev(rev) => {
                self.checkout_rev(&git_repo, rev)?;
            }
            RefSpec::Branch(_) => {
                // Already handled by builder
            }
        }

        Ok(git_repo)
    }

    /// Update an existing repository
    pub fn update(&self, repo: &Repository) -> Result<()> {
        let path = self.repo_path(repo);
        let git_repo = Git2Repository::open(&path)?;

        // Fetch from remote
        self.fetch(&git_repo)?;

        // Checkout the appropriate ref
        match &repo.ref_spec {
            RefSpec::Branch(branch) => {
                self.checkout_branch(&git_repo, branch)?;
            }
            RefSpec::Tag(tag) => {
                self.checkout_tag(&git_repo, tag)?;
            }
            RefSpec::Rev(rev) => {
                self.checkout_rev(&git_repo, rev)?;
            }
        }

        Ok(())
    }

    /// Fetch from origin
    fn fetch(&self, repo: &Git2Repository) -> Result<()> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut remote = repo.find_remote("origin")?;
        remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_opts), None)?;

        Ok(())
    }

    /// Checkout a branch
    fn checkout_branch(&self, repo: &Git2Repository, branch: &str) -> Result<()> {
        let refname = format!("refs/remotes/origin/{}", branch);
        let reference = repo.find_reference(&refname)?;
        let commit = reference.peel_to_commit()?;

        // Update or create local branch
        let local_refname = format!("refs/heads/{}", branch);
        repo.reference(&local_refname, commit.id(), true, "mctl: update branch")?;

        // Checkout
        let obj = commit.as_object();
        repo.checkout_tree(obj, None)?;
        repo.set_head(&local_refname)?;

        Ok(())
    }

    /// Checkout a tag
    fn checkout_tag(&self, repo: &Git2Repository, tag: &str) -> Result<()> {
        let refname = format!("refs/tags/{}", tag);
        let reference = repo.find_reference(&refname)?;
        let commit = reference.peel_to_commit()?;

        let obj = commit.as_object();
        repo.checkout_tree(obj, None)?;
        repo.set_head_detached(commit.id())?;

        Ok(())
    }

    /// Checkout a specific revision
    fn checkout_rev(&self, repo: &Git2Repository, rev: &str) -> Result<()> {
        let oid = git2::Oid::from_str(rev).map_err(|e| {
            MirrorError::Config(format!("Invalid revision '{}': {}", rev, e))
        })?;

        let commit = repo.find_commit(oid)?;
        let obj = commit.as_object();
        repo.checkout_tree(obj, None)?;
        repo.set_head_detached(oid)?;

        Ok(())
    }

    /// Get the current HEAD commit SHA
    pub fn get_head_sha(&self, repo: &Repository) -> Result<String> {
        let path = self.repo_path(repo);
        let git_repo = Git2Repository::open(&path)?;
        let head = git_repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id().to_string())
    }

    /// Check if repository has uncommitted changes
    pub fn has_changes(&self, repo: &Repository) -> Result<bool> {
        let path = self.repo_path(repo);
        let git_repo = Git2Repository::open(&path)?;

        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true);

        let statuses = git_repo.statuses(Some(&mut opts))?;
        Ok(!statuses.is_empty())
    }

    /// Get list of changed files
    pub fn get_changed_files(&self, repo: &Repository) -> Result<Vec<String>> {
        let path = self.repo_path(repo);
        let git_repo = Git2Repository::open(&path)?;

        let mut opts = git2::StatusOptions::new();
        opts.include_untracked(true);

        let statuses = git_repo.statuses(Some(&mut opts))?;
        let files: Vec<String> = statuses
            .iter()
            .filter_map(|entry| entry.path().map(String::from))
            .collect();

        Ok(files)
    }

    /// Stage all changes, commit and push
    pub fn save(&self, repo: &Repository, message: &str) -> Result<()> {
        let path = self.repo_path(repo);
        let git_repo = Git2Repository::open(&path)?;

        // Stage all changes
        let mut index = git_repo.index()?;
        index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        // Create commit
        let tree_id = index.write_tree()?;
        let tree = git_repo.find_tree(tree_id)?;
        let signature = git_repo.signature()?;
        let head = git_repo.head()?;
        let parent_commit = head.peel_to_commit()?;

        git_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        // Push to remote
        self.push(&git_repo, repo)?;

        Ok(())
    }

    /// Push to remote
    fn push(&self, git_repo: &Git2Repository, repo: &Repository) -> Result<()> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        let mut remote = git_repo.find_remote("origin")?;

        let branch = match &repo.ref_spec {
            RefSpec::Branch(b) => b.clone(),
            _ => "main".to_string(),
        };

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);
        remote.push(&[&refspec], Some(&mut push_opts))?;

        Ok(())
    }

    /// Open an existing repository
    pub fn open(&self, repo: &Repository) -> Result<Git2Repository> {
        let path = self.repo_path(repo);
        Ok(Git2Repository::open(&path)?)
    }
}

/// Result of a sync operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncResult {
    Cloned,
    Updated,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RefSpec;
    use tempfile::TempDir;

    fn create_test_repo(path: &str) -> Repository {
        Repository {
            git: "git@github.com:test/repo.git".to_string(),
            path: path.to_string(),
            ref_spec: RefSpec::default(),
            workspaces: vec![],
        }
    }

    #[test]
    fn test_repo_path() {
        let dir = TempDir::new().unwrap();
        let manager = GitManager::new(dir.path());
        let repo = create_test_repo("test/repo");

        let path = manager.repo_path(&repo);
        assert!(path.ends_with("test/repo"));
    }

    #[test]
    fn test_exists_false_for_missing() {
        let dir = TempDir::new().unwrap();
        let manager = GitManager::new(dir.path());
        let repo = create_test_repo("test/repo");

        assert!(!manager.exists(&repo));
    }
}
