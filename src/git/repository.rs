//! Git repository module for MCTL
//!
//! This module provides a wrapper around git2 for repository operations.

use crate::error::types::{ErrorCode, MctlError};
use crate::git::credentials::GitCredentials;
use git2::{
    Cred, FetchOptions, PushOptions, Remote, RemoteCallbacks, Repository, Status, StatusOptions,
};
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

/// Git repository wrapper
pub struct GitRepository {
    /// Path to the repository
    pub path: PathBuf,

    /// Git URL of the repository
    pub url: String,

    /// Branch to track
    pub branch: Option<String>,

    /// Authentication method (ssh or https)
    pub auth_method: Option<String>,

    /// Git2 repository instance
    repo: Option<Repository>,

    /// Credentials for authentication
    credentials: Option<GitCredentials>,
}

impl GitRepository {
    /// Create a new GitRepository instance
    pub fn new<P: AsRef<Path>>(
        path: P,
        url: String,
        branch: Option<String>,
        auth_method: Option<String>,
    ) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            url,
            branch,
            auth_method,
            repo: None,
            credentials: None,
        }
    }

    /// Open an existing repository
    pub fn open(&mut self) -> Result<&Repository, MctlError> {
        if self.repo.is_none() {
            debug!("Opening repository at {}", self.path.display());
            let repo = Repository::open(&self.path).map_err(|e| {
                let error_code = if e.code() == git2::ErrorCode::NotFound {
                    ErrorCode::RepositoryNotFound
                } else {
                    ErrorCode::GitCommandFailed
                };

                let error = crate::error::types::ConfigError::new(
                    error_code,
                    format!("Failed to open repository: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e));

                error
            })?;

            self.repo = Some(repo);
        }

        Ok(self.repo.as_ref().unwrap())
    }

    /// Clone a repository
    pub fn clone(&mut self) -> Result<&Repository, MctlError> {
        if self.repo.is_some() {
            return Ok(self.repo.as_ref().unwrap());
        }

        debug!(
            "Cloning repository from {} to {}",
            self.url,
            self.path.display()
        );

        // Create parent directories if they don't exist
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to create parent directories: {}", e),
                    parent.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;
        }

        // Set up fetch options with authentication
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();

        // Set up credentials callback
        if let Some(credentials) = &self.credentials {
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                credentials.get_cred(url, username_from_url, allowed_types)
            });
        }

        fetch_options.remote_callbacks(callbacks);

        // Clone options
        let mut clone_options = git2::CloneOptions::new();
        clone_options.fetch_options(fetch_options);

        // Set branch if specified
        if let Some(branch) = &self.branch {
            clone_options.checkout_branch(branch);
        }

        // Clone the repository
        let repo = git2::Repository::clone(&self.url, &self.path).map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to clone repository: {}", e),
                format!("URL: {}, Path: {}", self.url, self.path.display()),
            )
            .with_source(Box::new(e))
        })?;

        self.repo = Some(repo);
        info!("Successfully cloned repository to {}", self.path.display());

        Ok(self.repo.as_ref().unwrap())
    }

    /// Set credentials for authentication
    pub fn set_credentials(&mut self, credentials: GitCredentials) {
        self.credentials = Some(credentials);
    }

    /// Get the repository status
    pub fn status(&self) -> Result<Vec<(String, Status)>, MctlError> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::RepositoryNotFound,
                "Repository not opened or cloned".to_string(),
                self.path.display().to_string(),
            )
        })?;

        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.recurse_untracked_dirs(true);
        status_options.include_ignored(false);

        let statuses = repo.statuses(Some(&mut status_options)).map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to get repository status: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        let mut result = Vec::new();
        for entry in statuses.iter() {
            let path = entry.path().unwrap_or("").to_string();
            let status = entry.status();
            result.push((path, status));
        }

        Ok(result)
    }

    /// Commit changes to the repository
    pub fn commit(&self, message: &str) -> Result<git2::Oid, MctlError> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::RepositoryNotFound,
                "Repository not opened or cloned".to_string(),
                self.path.display().to_string(),
            )
        })?;

        // Get repository status
        let statuses = self.status()?;
        if statuses.is_empty() {
            info!("No changes to commit in repository {}", self.path.display());
            return Err(crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                "No changes to commit".to_string(),
                self.path.display().to_string(),
            )
            .into());
        }

        // Get the index
        let mut index = repo.index().map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to get repository index: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Add all changes to the index
        for (path, status) in &statuses {
            if status.is_wt_new()
                || status.is_wt_modified()
                || status.is_wt_renamed()
                || status.is_wt_typechange()
            {
                index.add_path(Path::new(path)).map_err(|e| {
                    crate::error::types::ConfigError::new(
                        ErrorCode::GitCommandFailed,
                        format!("Failed to add file to index: {}", e),
                        path.to_string(),
                    )
                    .with_source(Box::new(e))
                })?;
            }
        }

        // Write the index
        let oid = index.write_tree().map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to write index: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Get the tree
        let tree = repo.find_tree(oid).map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to find tree: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Get the signature
        let signature = repo.signature().map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to get signature: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Get the parent commit
        let head = repo.head().ok();
        let parent_commit = head.as_ref().and_then(|h| h.peel_to_commit().ok());
        let parents = parent_commit
            .as_ref()
            .map(|c| vec![c])
            .unwrap_or_else(Vec::new);
        let parents = parents.iter().collect::<Vec<_>>();

        // Create the commit
        let commit_id = repo
            .commit(
                Some("HEAD"),
                &signature,
                &signature,
                message,
                &tree,
                &parents,
            )
            .map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to commit changes: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

        info!(
            "Committed changes to repository {}: {}",
            self.path.display(),
            commit_id
        );

        Ok(commit_id)
    }

    /// Push changes to the remote repository
    pub fn push(&self) -> Result<(), MctlError> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::RepositoryNotFound,
                "Repository not opened or cloned".to_string(),
                self.path.display().to_string(),
            )
        })?;

        // Get the remote
        let mut remote = repo.find_remote("origin").map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to find remote 'origin': {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Set up push options with authentication
        let mut push_options = PushOptions::new();
        let mut callbacks = RemoteCallbacks::new();

        // Set up credentials callback
        if let Some(credentials) = &self.credentials {
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                credentials.get_cred(url, username_from_url, allowed_types)
            });
        }

        push_options.remote_callbacks(callbacks);

        // Get the current branch
        let head = repo.head().map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to get HEAD: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        let branch_name = head.shorthand().ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                "Failed to get branch name".to_string(),
                self.path.display().to_string(),
            )
        })?;

        // Push the branch
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
        remote
            .push(&[&refspec], Some(&mut push_options))
            .map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to push changes: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

        info!(
            "Pushed changes to remote repository for {}",
            self.path.display()
        );

        Ok(())
    }

    /// Pull changes from the remote repository
    pub fn pull(&self) -> Result<(), MctlError> {
        let repo = self.repo.as_ref().ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::RepositoryNotFound,
                "Repository not opened or cloned".to_string(),
                self.path.display().to_string(),
            )
        })?;

        // Get the remote
        let mut remote = repo.find_remote("origin").map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to find remote 'origin': {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Set up fetch options with authentication
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();

        // Set up credentials callback
        if let Some(credentials) = &self.credentials {
            callbacks.credentials(move |url, username_from_url, allowed_types| {
                credentials.get_cred(url, username_from_url, allowed_types)
            });
        }

        fetch_options.remote_callbacks(callbacks);

        // Fetch from remote
        remote
            .fetch(&[], Some(&mut fetch_options), None)
            .map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to fetch from remote: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

        // Get the current branch
        let head = repo.head().map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to get HEAD: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        let branch_name = head.shorthand().ok_or_else(|| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                "Failed to get branch name".to_string(),
                self.path.display().to_string(),
            )
        })?;

        // Get the remote branch reference
        let remote_branch = format!("refs/remotes/origin/{}", branch_name);
        let remote_ref = repo.find_reference(&remote_branch).map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to find remote reference: {}", e),
                remote_branch,
            )
            .with_source(Box::new(e))
        })?;

        // Get the remote commit
        let remote_commit = remote_ref.peel_to_commit().map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to get remote commit: {}", e),
                remote_branch,
            )
            .with_source(Box::new(e))
        })?;

        // Merge the remote commit into the current branch
        let annotated_commit = repo
            .reference_to_annotated_commit(&remote_ref)
            .map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to get annotated commit: {}", e),
                    remote_branch,
                )
                .with_source(Box::new(e))
            })?;

        let (merge_analysis, _) = repo.merge_analysis(&[&annotated_commit]).map_err(|e| {
            crate::error::types::ConfigError::new(
                ErrorCode::GitCommandFailed,
                format!("Failed to analyze merge: {}", e),
                self.path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        if merge_analysis.is_up_to_date() {
            info!("Repository {} is already up to date", self.path.display());
            return Ok(());
        }

        if merge_analysis.is_fast_forward() {
            // Fast-forward merge
            let mut reference = repo
                .find_reference(&format!("refs/heads/{}", branch_name))
                .map_err(|e| {
                    crate::error::types::ConfigError::new(
                        ErrorCode::GitCommandFailed,
                        format!("Failed to find reference: {}", e),
                        branch_name.to_string(),
                    )
                    .with_source(Box::new(e))
                })?;

            reference
                .set_target(remote_commit.id(), "Fast-forward")
                .map_err(|e| {
                    crate::error::types::ConfigError::new(
                        ErrorCode::GitCommandFailed,
                        format!("Failed to fast-forward: {}", e),
                        branch_name.to_string(),
                    )
                    .with_source(Box::new(e))
                })?;

            repo.checkout_head(Some(git2::build::CheckoutBuilder::new().force()))
                .map_err(|e| {
                    crate::error::types::ConfigError::new(
                        ErrorCode::GitCommandFailed,
                        format!("Failed to checkout HEAD: {}", e),
                        self.path.display().to_string(),
                    )
                    .with_source(Box::new(e))
                })?;

            info!(
                "Fast-forwarded repository {} to {}",
                self.path.display(),
                remote_commit.id()
            );
        } else {
            // Normal merge
            let head_commit = head.peel_to_commit().map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to get head commit: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

            let mut merge_options = git2::MergeOptions::new();
            let mut checkout_options = git2::build::CheckoutBuilder::new();
            checkout_options.force();

            repo.merge(
                &[&annotated_commit],
                Some(&mut merge_options),
                Some(&mut checkout_options),
            )
            .map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to merge: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

            // Check if merge resulted in conflicts
            if repo.index().unwrap().has_conflicts() {
                warn!(
                    "Merge conflicts detected in repository {}",
                    self.path.display()
                );

                // Abort the merge
                repo.cleanup_state().map_err(|e| {
                    crate::error::types::ConfigError::new(
                        ErrorCode::GitMergeConflict,
                        format!("Failed to abort merge: {}", e),
                        self.path.display().to_string(),
                    )
                    .with_source(Box::new(e))
                })?;

                return Err(crate::error::types::ConfigError::new(
                    ErrorCode::GitMergeConflict,
                    "Merge conflicts detected".to_string(),
                    self.path.display().to_string(),
                )
                .into());
            }

            // Create the merge commit
            let tree_id = repo.index().unwrap().write_tree().map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to write tree: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

            let tree = repo.find_tree(tree_id).map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to find tree: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

            let signature = repo.signature().map_err(|e| {
                crate::error::types::ConfigError::new(
                    ErrorCode::GitCommandFailed,
                    format!("Failed to get signature: {}", e),
                    self.path.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;

            let message = format!("Merge branch '{}' of {}", branch_name, self.url);
            let commit_id = repo
                .commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &message,
                    &tree,
                    &[&head_commit, &remote_commit],
                )
                .map_err(|e| {
                    crate::error::types::ConfigError::new(
                        ErrorCode::GitCommandFailed,
                        format!("Failed to create merge commit: {}", e),
                        self.path.display().to_string(),
                    )
                    .with_source(Box::new(e))
                })?;

            info!(
                "Merged changes into repository {}: {}",
                self.path.display(),
                commit_id
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new_git_repository() {
        let repo = GitRepository::new(
            "test-repo",
            "git@github.com:example/repo.git".to_string(),
            Some("main".to_string()),
            Some("ssh".to_string()),
        );

        assert_eq!(repo.path, PathBuf::from("test-repo"));
        assert_eq!(repo.url, "git@github.com:example/repo.git");
        assert_eq!(repo.branch, Some("main".to_string()));
        assert_eq!(repo.auth_method, Some("ssh".to_string()));
        assert!(repo.repo.is_none());
        assert!(repo.credentials.is_none());
    }
}
