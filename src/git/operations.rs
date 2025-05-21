//! Git operations module for MCTL
//!
//! This module provides high-level git operations for repository management.

use crate::error::types::MctlError;
use crate::git::credentials::GitCredentials;
use crate::git::repository::GitRepository;
use log::{debug, info, warn};
use std::path::{Path, PathBuf};

/// Clone a git repository
pub fn clone<P: AsRef<Path>>(
    url: &str,
    path: P,
    branch: Option<&str>,
    auth_method: Option<&str>,
    credentials: Option<GitCredentials>,
) -> Result<GitRepository, MctlError> {
    debug!(
        "Cloning repository from {} to {}",
        url,
        path.as_ref().display()
    );

    let mut repo = GitRepository::new(
        path.as_ref(),
        url.to_string(),
        branch.map(|s| s.to_string()),
        auth_method.map(|s| s.to_string()),
    );

    // Set credentials if provided
    if let Some(creds) = credentials {
        repo.set_credentials(creds);
    }

    // Clone the repository
    repo.clone()?;

    info!(
        "Successfully cloned repository from {} to {}",
        url,
        path.as_ref().display()
    );

    Ok(repo)
}

/// Pull changes from a git repository
pub fn pull<P: AsRef<Path>>(path: P, credentials: Option<GitCredentials>) -> Result<(), MctlError> {
    debug!(
        "Pulling changes for repository at {}",
        path.as_ref().display()
    );

    // Open the repository
    let mut repo = GitRepository::new(
        path.as_ref(),
        "".to_string(), // URL not needed for pull
        None,
        None,
    );

    // Set credentials if provided
    if let Some(creds) = credentials {
        repo.set_credentials(creds);
    }

    // Open the repository
    repo.open()?;

    // Pull changes
    repo.pull()?;

    info!(
        "Successfully pulled changes for repository at {}",
        path.as_ref().display()
    );

    Ok(())
}

/// Push changes to a git repository
pub fn push<P: AsRef<Path>>(path: P, credentials: Option<GitCredentials>) -> Result<(), MctlError> {
    debug!(
        "Pushing changes for repository at {}",
        path.as_ref().display()
    );

    // Open the repository
    let mut repo = GitRepository::new(
        path.as_ref(),
        "".to_string(), // URL not needed for push
        None,
        None,
    );

    // Set credentials if provided
    if let Some(creds) = credentials {
        repo.set_credentials(creds);
    }

    // Open the repository
    repo.open()?;

    // Push changes
    repo.push()?;

    info!(
        "Successfully pushed changes for repository at {}",
        path.as_ref().display()
    );

    Ok(())
}

/// Commit changes to a git repository
pub fn commit<P: AsRef<Path>>(path: P, message: &str) -> Result<git2::Oid, MctlError> {
    debug!(
        "Committing changes for repository at {}",
        path.as_ref().display()
    );

    // Open the repository
    let mut repo = GitRepository::new(
        path.as_ref(),
        "".to_string(), // URL not needed for commit
        None,
        None,
    );

    // Open the repository
    repo.open()?;

    // Commit changes
    let commit_id = repo.commit(message)?;

    info!(
        "Successfully committed changes for repository at {}: {}",
        path.as_ref().display(),
        commit_id
    );

    Ok(commit_id)
}

/// Get the status of a git repository
pub fn status<P: AsRef<Path>>(path: P) -> Result<Vec<(String, git2::Status)>, MctlError> {
    debug!(
        "Getting status for repository at {}",
        path.as_ref().display()
    );

    // Open the repository
    let mut repo = GitRepository::new(
        path.as_ref(),
        "".to_string(), // URL not needed for status
        None,
        None,
    );

    // Open the repository
    repo.open()?;

    // Get status
    let status = repo.status()?;

    debug!("Repository status: {:?}", status);

    Ok(status)
}

/// Format git status for display
pub fn format_status(status: &[(String, git2::Status)]) -> String {
    let mut result = String::new();

    if status.is_empty() {
        result.push_str("No changes\n");
        return result;
    }

    for (path, status_flags) in status {
        let status_str = if status_flags.is_index_new() {
            "A"
        } else if status_flags.is_index_modified() {
            "M"
        } else if status_flags.is_index_deleted() {
            "D"
        } else if status_flags.is_index_renamed() {
            "R"
        } else if status_flags.is_index_typechange() {
            "T"
        } else if status_flags.is_wt_new() {
            "??"
        } else if status_flags.is_wt_modified() {
            " M"
        } else if status_flags.is_wt_deleted() {
            " D"
        } else if status_flags.is_wt_renamed() {
            " R"
        } else if status_flags.is_wt_typechange() {
            " T"
        } else if status_flags.is_ignored() {
            "!!"
        } else {
            "  "
        };

        result.push_str(&format!("{} {}\n", status_str, path));
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_format_status() {
        let status = vec![
            ("file1.txt".to_string(), git2::Status::WT_NEW),
            ("file2.txt".to_string(), git2::Status::WT_MODIFIED),
            ("file3.txt".to_string(), git2::Status::INDEX_NEW),
            ("file4.txt".to_string(), git2::Status::INDEX_MODIFIED),
        ];

        let formatted = format_status(&status);
        assert!(formatted.contains("?? file1.txt"));
        assert!(formatted.contains(" M file2.txt"));
        assert!(formatted.contains("A file3.txt"));
        assert!(formatted.contains("M file4.txt"));
    }
}
