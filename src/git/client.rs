//! Git client implementation using external git CLI.

use crate::core::error::{GitError, GitResult};
use crate::core::repository::VersionSpec;
use crate::git::command::GitCommand;
use crate::git::output::{
    is_auth_error, is_network_error, is_repo_not_found, parse_branch_name, parse_rev,
    parse_status_porcelain_v2,
};
use crate::git::status::RepositoryStatus;
use std::path::Path;
use std::process::Output;
use std::time::Duration;

/// Default timeout for git operations (5 minutes).
const DEFAULT_TIMEOUT_SECS: u64 = 300;

/// Number of retry attempts for network operations.
const DEFAULT_RETRIES: u32 = 3;

/// Git client configuration.
#[derive(Debug, Clone)]
pub struct GitClientConfig {
    /// Path to git executable (default: "git")
    pub git_path: String,
    /// Custom SSH command (GIT_SSH_COMMAND)
    pub ssh_command: Option<String>,
    /// Timeout for operations in seconds
    pub timeout_secs: u64,
    /// Number of retries for network operations
    pub retries: u32,
    /// Enable verbose output
    pub verbose: bool,
}

impl Default for GitClientConfig {
    fn default() -> Self {
        Self {
            git_path: "git".to_string(),
            ssh_command: None,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            retries: DEFAULT_RETRIES,
            verbose: false,
        }
    }
}

/// Git client for repository operations.
#[derive(Debug, Clone)]
pub struct GitClient {
    config: GitClientConfig,
}

impl Default for GitClient {
    fn default() -> Self {
        Self::new()
    }
}

impl GitClient {
    /// Create a new git client with default configuration.
    pub fn new() -> Self {
        Self {
            config: GitClientConfig::default(),
        }
    }

    /// Create a git client with custom SSH command.
    pub fn with_ssh_command(ssh_cmd: impl Into<String>) -> Self {
        Self {
            config: GitClientConfig {
                ssh_command: Some(ssh_cmd.into()),
                ..Default::default()
            },
        }
    }

    /// Create a git client with custom configuration.
    pub fn with_config(config: GitClientConfig) -> Self {
        Self { config }
    }

    /// Check if git is available.
    pub fn check_git_available(&self) -> GitResult<String> {
        let output = GitCommand::new("--version")
            .git_path(&self.config.git_path)
            .build()
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(GitError::GitNotFound {
                path: self.config.git_path.clone(),
            })
        }
    }

    /// Check if a path is a git repository.
    pub fn is_git_repository(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        let result = self.run_command(GitCommand::is_git_repo(path));
        result.is_ok()
    }

    /// Clone a repository.
    pub fn clone(&self, url: &str, target: &Path, version: &VersionSpec) -> GitResult<()> {
        // Create parent directory if needed
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let cmd = match version {
            VersionSpec::DefaultBranch => {
                // Let git use repository's default branch
                GitCommand::clone_repo(url, target)
            }
            VersionSpec::Branch(branch) => {
                GitCommand::clone_branch(url, target, branch)
            }
            VersionSpec::Tag(tag) => {
                GitCommand::clone_repo(url, target)
                    .option("--branch", tag)
                    .flag("--single-branch")
            }
            VersionSpec::Rev(_) => {
                // For specific revision, clone first then checkout
                GitCommand::clone_repo(url, target)
            }
        };

        let cmd = self.apply_config(cmd);
        self.run_command_with_retry(cmd)?;

        // For rev, checkout the specific commit
        if let VersionSpec::Rev(rev) = version {
            self.checkout_rev(target, rev)?;
        }

        Ok(())
    }

    /// Fetch from remote.
    pub fn fetch(&self, repo_path: &Path) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::fetch(repo_path));
        self.run_command_with_retry(cmd)?;
        Ok(())
    }

    /// Pull from remote (fast-forward only).
    pub fn pull(&self, repo_path: &Path) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::pull(repo_path));
        self.run_command_with_retry(cmd)?;
        Ok(())
    }

    /// Push to remote.
    pub fn push(&self, repo_path: &Path) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::push(repo_path));
        let result = self.run_command_with_retry(cmd);

        match result {
            Ok(_) => Ok(()),
            Err(GitError::CommandFailed { stderr, .. }) if stderr.contains("rejected") => {
                Err(GitError::PushRejected)
            }
            Err(e) => Err(e),
        }
    }

    /// Get repository status.
    pub fn status(&self, repo_path: &Path) -> GitResult<RepositoryStatus> {
        self.ensure_git_repo(repo_path)?;

        // Get status
        let cmd = self.apply_config(GitCommand::status(repo_path));
        let output = self.run_command(cmd)?;
        let (branch_info, files) = parse_status_porcelain_v2(&output);

        // Get HEAD rev
        let head_full = self.get_head_rev(repo_path)?;
        let head_short = self.get_head_rev_short(repo_path)?;

        let branch = branch_info.unwrap_or_else(|| crate::git::status::BranchInfo {
            name: "HEAD".to_string(),
            upstream: None,
            ahead: 0,
            behind: 0,
        });

        Ok(RepositoryStatus {
            branch,
            files,
            head_short,
            head_full,
        })
    }

    /// Get repository status (fast version - single git call, no HEAD rev).
    /// Use this for table display where HEAD hash is not needed.
    pub fn status_fast(&self, repo_path: &Path) -> GitResult<RepositoryStatus> {
        // Skip ensure_git_repo - caller should check this beforehand
        let cmd = self.apply_config(GitCommand::status(repo_path));
        let output = self.run_command(cmd)?;
        let (branch_info, files) = parse_status_porcelain_v2(&output);

        let branch = branch_info.unwrap_or_else(|| crate::git::status::BranchInfo {
            name: "HEAD".to_string(),
            upstream: None,
            ahead: 0,
            behind: 0,
        });

        Ok(RepositoryStatus {
            branch,
            files,
            head_short: String::new(),
            head_full: String::new(),
        })
    }

    /// Get current branch name.
    pub fn get_current_branch(&self, repo_path: &Path) -> GitResult<String> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::current_branch(repo_path));
        let output = self.run_command(cmd)?;
        parse_branch_name(&output).ok_or_else(|| GitError::ParseError {
            message: "Failed to parse branch name".to_string(),
        })
    }

    /// Get HEAD commit hash (full).
    pub fn get_head_rev(&self, repo_path: &Path) -> GitResult<String> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::rev_parse_head(repo_path));
        let output = self.run_command(cmd)?;
        parse_rev(&output).ok_or_else(|| GitError::ParseError {
            message: "Failed to parse HEAD revision".to_string(),
        })
    }

    /// Get HEAD commit hash (short).
    pub fn get_head_rev_short(&self, repo_path: &Path) -> GitResult<String> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::rev_parse_head_short(repo_path));
        let output = self.run_command(cmd)?;
        Ok(output.trim().to_string())
    }

    /// Checkout a branch.
    pub fn checkout(&self, repo_path: &Path, ref_name: &str) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::checkout(repo_path, ref_name));
        self.run_command(cmd)?;
        Ok(())
    }

    /// Checkout a specific revision.
    pub fn checkout_rev(&self, repo_path: &Path, rev: &str) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::checkout(repo_path, rev));
        let result = self.run_command(cmd);

        match result {
            Ok(_) => Ok(()),
            Err(GitError::CommandFailed { stderr, .. })
                if stderr.contains("did not match any") =>
            {
                Err(GitError::RevisionNotFound {
                    rev: rev.to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }

    /// Checkout a tag.
    pub fn checkout_tag(&self, repo_path: &Path, tag: &str) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        // First try to checkout the tag directly
        let cmd = self.apply_config(GitCommand::checkout(repo_path, &format!("tags/{}", tag)));
        let result = self.run_command(cmd);

        match result {
            Ok(_) => Ok(()),
            Err(GitError::CommandFailed { .. }) => {
                // Try without tags/ prefix
                let cmd = self.apply_config(GitCommand::checkout(repo_path, tag));
                let result = self.run_command(cmd);
                match result {
                    Ok(_) => Ok(()),
                    Err(GitError::CommandFailed { stderr, .. })
                        if stderr.contains("did not match any") =>
                    {
                        Err(GitError::TagNotFound {
                            tag: tag.to_string(),
                        })
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Create and checkout a new branch.
    pub fn checkout_new_branch(&self, repo_path: &Path, branch: &str) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::checkout_new_branch(repo_path, branch));
        self.run_command(cmd)?;
        Ok(())
    }

    /// Get diff (unstaged changes).
    pub fn diff(&self, repo_path: &Path) -> GitResult<String> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::diff(repo_path));
        self.run_command(cmd)
    }

    /// Get diff of staged changes.
    pub fn diff_staged(&self, repo_path: &Path) -> GitResult<String> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::diff_staged(repo_path));
        self.run_command(cmd)
    }

    /// Stage all changes.
    pub fn add_all(&self, repo_path: &Path) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::add_all(repo_path));
        self.run_command(cmd)?;
        Ok(())
    }

    /// Commit staged changes.
    pub fn commit(&self, repo_path: &Path, message: &str) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::commit(repo_path, message));
        let result = self.run_command(cmd);

        match result {
            Ok(_) => Ok(()),
            Err(GitError::CommandFailed { stderr, .. })
                if stderr.contains("nothing to commit") =>
            {
                Err(GitError::NoChangesToCommit)
            }
            Err(e) => Err(e),
        }
    }

    /// Get remote URL.
    pub fn get_remote_url(&self, repo_path: &Path, remote: &str) -> GitResult<String> {
        self.ensure_git_repo(repo_path)?;
        let cmd = self.apply_config(GitCommand::remote_url(repo_path, remote));
        let output = self.run_command(cmd)?;
        Ok(output.trim().to_string())
    }

    /// Sync a repository to match a version spec.
    pub fn sync(&self, repo_path: &Path, version: &VersionSpec) -> GitResult<()> {
        self.ensure_git_repo(repo_path)?;

        // Fetch first
        self.fetch(repo_path)?;

        match version {
            VersionSpec::DefaultBranch => {
                // Just pull on current branch
                self.pull(repo_path)?;
            }
            VersionSpec::Branch(branch) => {
                // Checkout branch and pull
                self.checkout(repo_path, branch)?;
                self.pull(repo_path)?;
            }
            VersionSpec::Rev(rev) => {
                self.checkout_rev(repo_path, rev)?;
            }
            VersionSpec::Tag(tag) => {
                self.checkout_tag(repo_path, tag)?;
            }
        }

        Ok(())
    }

    // Internal helpers

    fn apply_config(&self, cmd: GitCommand) -> GitCommand {
        let mut cmd = cmd.git_path(&self.config.git_path);

        if let Some(ssh_cmd) = &self.config.ssh_command {
            cmd = cmd.ssh_command(ssh_cmd);
        }

        cmd
    }

    fn ensure_git_repo(&self, path: &Path) -> GitResult<()> {
        if !path.exists() {
            return Err(GitError::RepositoryNotFound {
                path: path.to_path_buf(),
            });
        }

        if !self.is_git_repository(path) {
            return Err(GitError::NotGitRepository {
                path: path.to_path_buf(),
            });
        }

        Ok(())
    }

    fn run_command(&self, cmd: GitCommand) -> GitResult<String> {
        let command_str = cmd.to_string();
        let mut process = cmd.build();

        let output = process.output()?;

        self.handle_output(output, &command_str)
    }

    fn run_command_with_retry(&self, cmd: GitCommand) -> GitResult<String> {
        let mut last_error = None;

        for attempt in 0..self.config.retries {
            if attempt > 0 {
                // Exponential backoff
                let wait_secs = 2u64.pow(attempt);
                std::thread::sleep(Duration::from_secs(wait_secs));
            }

            match self.run_command(cmd.clone()) {
                Ok(output) => return Ok(output),
                Err(e) => {
                    // Only retry network/auth errors
                    match &e {
                        GitError::CommandFailed { stderr, .. } => {
                            if is_network_error(stderr) || is_auth_error(stderr) {
                                last_error = Some(e);
                                continue;
                            }
                        }
                        GitError::AuthenticationFailed { .. } => {
                            last_error = Some(e);
                            continue;
                        }
                        _ => {}
                    }
                    return Err(e);
                }
            }
        }

        Err(last_error.unwrap_or(GitError::Timeout {
            seconds: self.config.timeout_secs,
        }))
    }

    fn handle_output(&self, output: Output, command: &str) -> GitResult<String> {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(stdout)
        } else {
            let exit_code = output.status.code().unwrap_or(-1);

            // Check for specific error types
            if is_auth_error(&stderr) {
                return Err(GitError::AuthenticationFailed {
                    message: stderr.clone(),
                });
            }

            if is_repo_not_found(&stderr) {
                // Extract URL from command if possible
                return Err(GitError::CloneFailed {
                    url: "unknown".to_string(),
                    message: stderr.clone(),
                });
            }

            Err(GitError::CommandFailed {
                exit_code,
                stderr,
                command: command.to_string(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_git_available() {
        let client = GitClient::new();
        let result = client.check_git_available();
        assert!(result.is_ok());
        assert!(result.unwrap().contains("git version"));
    }

    #[test]
    fn test_is_not_git_repository() {
        let client = GitClient::new();
        // A temporary directory is not a git repo
        let temp_dir = std::env::temp_dir();
        assert!(!client.is_git_repository(&temp_dir.join("nonexistent_repo_12345")));
    }
}
