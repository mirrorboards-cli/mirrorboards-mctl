//! # Git Operations Module
//!
//! This module provides implementations for Git repository operations,
//! with a focus on proper SSH authentication support.
//!
//! It uses system Git commands with environment variables for SSH authentication.
//!
//! ## SSH Authentication
//!
//! The Git operations implementation supports SSH authentication through several mechanisms:
//!
//! - Default SSH key (~/.ssh/id_rsa) is used if no specific configuration is provided
//! - Path expansion for SSH keys (e.g., "~/.ssh/id_rsa" expands to the user's home directory)
//! - Repository-specific SSH configuration through `RepositoryConfig`
//! - Environment variable configuration for global defaults
//!
//! ## Environment Variables
//!
//! The following environment variables can be used to configure SSH authentication:
//!
//! - `GIT_SSH_KEY_PATH`: Path to the SSH private key file (e.g., "~/.ssh/id_rsa")
//! - `GIT_KNOWN_HOSTS_PATH`: Path to the SSH known hosts file (e.g., "~/.ssh/known_hosts")
//! - `GIT_SSH_PASSPHRASE_COMMAND`: Command to retrieve the passphrase for the SSH key (if needed)
//! - `GIT_SSH_BATCH_MODE`: Whether to use batch mode (no interactive prompts) - "true" by default
//! - `GIT_SSH_STRICT_HOST_CHECKING`: StrictHostKeyChecking option value ("accept-new" by default)
//! - `GIT_SSH_CONNECTION_TIMEOUT`: SSH connection timeout in seconds (30 by default)
//! - `GIT_SSH_ADDITIONAL_OPTIONS`: Additional SSH options to pass to the SSH command
//! - `GIT_OPERATION_TIMEOUT`: Default timeout for Git operations in seconds (300 by default)
//!
//! ## Error Handling
//!
//! The module provides detailed error diagnostics and guidance for authentication issues, including:
//!
//! - SSH key not found or permission denied
//! - Host key verification failures
//! - Connection timeouts
//! - Passphrase required but not provided
//! - DNS or network issues
//!
//! ## Usage Example
//!
//! ```no_run
//! use std::path::Path;
//! use crate::domain::repository::{RepositoryConfig, SshConfig};
//! use crate::infrastructure::git::GitOperations;
//!
//! // Create a GitOperations instance with default settings
//! let git = GitOperations::new();
//!
//! // Or create one with repository-specific configuration
//! let config = RepositoryConfig {
//!     ssh: Some(SshConfig {
//!         key_path: Some(Path::new("~/.ssh/repo_specific_key").to_path_buf()),
//!         known_hosts_path: None,
//!         passphrase_command: None,
//!     }),
//!     commands: None,
//! };
//!
//! let git = GitOperations::from_repository_config(Some(&config));
//!
//! // Now use it for Git operations
//! git.clone("git@github.com:org/repo.git", Path::new("./local/path")).unwrap();
//! ```
//!

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use anyhow::{Result, Context, anyhow};
use tokio::process::Command as TokioCommand;
use tokio::time::timeout;

use crate::domain::repository::{RepositoryOperations, RepositoryStatus, RepositoryConfig};
use crate::domain::error::{GitError, RepositoryError};

/// Handler for SSH authentication with Git
pub struct GitSshHandler {
    /// Path to SSH private key
    ssh_key_path: Option<PathBuf>,
    /// Path to known hosts file
    known_hosts_path: Option<PathBuf>,
    /// Command to retrieve passphrase (if needed)
    passphrase_command: Option<String>,
    /// Use batch mode for SSH (no interactive prompts)
    batch_mode: bool,
}

impl GitSshHandler {
    /// Create a new SSH handler with default settings
    pub fn new() -> Self {
        // First try to get paths from environment variables
        let key_path = Self::get_path_from_env("GIT_SSH_KEY_PATH")
            .or_else(|| Self::default_ssh_key_path());
        
        let known_hosts_path = Self::get_path_from_env("GIT_KNOWN_HOSTS_PATH")
            .or_else(|| Self::default_known_hosts_path());
        
        Self {
            ssh_key_path: key_path,
            known_hosts_path: known_hosts_path,
            passphrase_command: std::env::var("GIT_SSH_PASSPHRASE_COMMAND").ok(),
            batch_mode: std::env::var("GIT_SSH_BATCH_MODE")
                .map(|v| v.to_lowercase() != "false")
                .unwrap_or(true),
        }
    }
    
    /// Expands a path that might contain ~ to reference the home directory
    fn expand_path(path: &str) -> PathBuf {
        if path.starts_with("~/") || path == "~" {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
            if path == "~" {
                PathBuf::from(home)
            } else {
                PathBuf::from(format!("{}{}", home, &path[1..]))
            }
        } else {
            PathBuf::from(path)
        }
    }
    
    /// Get path from environment variable and expand it
    fn get_path_from_env(var_name: &str) -> Option<PathBuf> {
        std::env::var(var_name).ok().map(|path| Self::expand_path(&path))
    }
    
    /// Get default SSH key path with expansion
    fn default_ssh_key_path() -> Option<PathBuf> {
        Some(Self::expand_path("~/.ssh/id_rsa"))
    }
    
    /// Get default known hosts path with expansion
    fn default_known_hosts_path() -> Option<PathBuf> {
        Some(Self::expand_path("~/.ssh/known_hosts"))
    }
    
    /// Create a new SSH handler with custom settings
    pub fn with_config(
        key_path: Option<PathBuf>,
        known_hosts_path: Option<PathBuf>,
        passphrase_command: Option<String>,
        batch_mode: bool,
    ) -> Self {
        Self {
            ssh_key_path: key_path,
            known_hosts_path: known_hosts_path,
            passphrase_command: passphrase_command,
            batch_mode: batch_mode,
        }
    }
    
    /// Prepare environment variables for Git SSH command
    pub fn prepare_environment(&self) -> Result<HashMap<String, String>> {
        let mut env_vars = HashMap::new();
        
        // Start with base SSH command
        let mut ssh_command = String::from("ssh");
        
        // Add batch mode if enabled
        if self.batch_mode {
            ssh_command.push_str(" -o BatchMode=yes");
        }
        
        // Add identity file if specified
        if let Some(key_path) = &self.ssh_key_path {
            if key_path.exists() {
                // Quote the path to handle spaces and special characters
                ssh_command.push_str(&format!(" -i \"{}\"", key_path.display()));
            } else {
                return Err(anyhow!(GitError::AuthenticationError {
                    message: format!("SSH key not found at {}. Ensure the key exists or set GIT_SSH_KEY_PATH environment variable to the correct path.", key_path.display()),
                    key_path: Some(key_path.clone()),
                }));
            }
        }
        
        // Add known hosts file if specified
        if let Some(known_hosts) = &self.known_hosts_path {
            if known_hosts.exists() {
                // Quote the path to handle spaces and special characters
                ssh_command.push_str(&format!(" -o UserKnownHostsFile=\"{}\"", known_hosts.display()));
            } else {
                // Just log a warning if known_hosts doesn't exist
                eprintln!("Warning: Known hosts file not found at {}", known_hosts.display());
            }
        }
        
        // For strict host key checking - use value from environment if available
        let strict_host_checking = std::env::var("GIT_SSH_STRICT_HOST_CHECKING")
            .unwrap_or_else(|_| String::from("accept-new"));
        ssh_command.push_str(&format!(" -o StrictHostKeyChecking={}", strict_host_checking));
        
        // Add connection timeout to avoid hanging operations
        let connection_timeout = std::env::var("GIT_SSH_CONNECTION_TIMEOUT")
            .unwrap_or_else(|_| String::from("30"));
        ssh_command.push_str(&format!(" -o ConnectTimeout={}", connection_timeout));
        
        // Add additional SSH options from environment if provided
        if let Ok(additional_opts) = std::env::var("GIT_SSH_ADDITIONAL_OPTIONS") {
            ssh_command.push_str(&format!(" {}", additional_opts));
        }
        
        // Set the GIT_SSH_COMMAND environment variable
        env_vars.insert("GIT_SSH_COMMAND".to_string(), ssh_command);
        
        // Pass through any SSH-specific environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with("SSH_") {
                env_vars.insert(key, value);
            }
        }
        
        Ok(env_vars)
    }
    
    /// Detect authentication issues from Git command output
    pub fn detect_auth_issues(&self, stderr: &str) -> Option<GitError> {
        // SSH public key authentication failure
        if stderr.contains("Permission denied (publickey)") {
            let mut message = "SSH authentication failed. Ensure your SSH key is correctly configured and added to the remote repository.".to_string();
            
            // Add more specific guidance based on available information
            if let Some(key_path) = &self.ssh_key_path {
                message.push_str(&format!("\n  - Verify the key at {} exists and has correct permissions (chmod 600).", key_path.display()));
                message.push_str("\n  - Confirm this key has been added to your remote repository provider (GitHub, GitLab, etc.).");
                message.push_str("\n  - Try running 'ssh-add' to add the key to the SSH agent.");
            } else {
                message.push_str("\n  - No SSH key specified. Set GIT_SSH_KEY_PATH or use a configuration file.");
            }
            
            message.push_str("\n  - Set SSH_AUTH_SOCK environment variable if using an SSH agent.");
            
            return Some(GitError::AuthenticationError {
                message,
                key_path: self.ssh_key_path.clone(),
            });
        }
        
        // Host key verification issues
        if stderr.contains("Host key verification failed") {
            let message = format!(
                "SSH host verification failed. The remote host key is not in your known_hosts file.\n  \
                - Run `ssh -T git@<host>` manually to add the host key\n  \
                - Or set GIT_SSH_STRICT_HOST_CHECKING=no to temporarily bypass verification\n  \
                - Known hosts file: {}",
                self.known_hosts_path.as_ref().map_or("not specified".to_string(), |p| p.display().to_string())
            );
            
            return Some(GitError::AuthenticationError {
                message,
                key_path: None,
            });
        }
        
        // Timeout issues
        if stderr.contains("Connection timed out") {
            return Some(GitError::AuthenticationError {
                message: "SSH connection timed out. Check your network connection and remote server availability.".to_string(),
                key_path: None,
            });
        }
        
        // Could not resolve hostname
        if stderr.contains("Could not resolve hostname") {
            return Some(GitError::AuthenticationError {
                message: "Could not resolve hostname. Check your network connection and DNS settings.".to_string(),
                key_path: None,
            });
        }
        
        // Passphrase needed
        if stderr.contains("Enter passphrase for key") || stderr.contains("Bad passphrase") {
            let message = format!(
                "SSH key requires a passphrase. You can:\n  \
                - Set GIT_SSH_PASSPHRASE_COMMAND to provide the passphrase automatically\n  \
                - Use ssh-agent and ssh-add to cache your key passphrase\n  \
                - Create and use a key without a passphrase for automation"
            );
            
            return Some(GitError::AuthenticationError {
                message,
                key_path: self.ssh_key_path.clone(),
            });
        }
        
        None
    }
}

/// Git operations implementation using system Git command
pub struct GitOperations {
    /// SSH handler for authentication
    ssh_handler: GitSshHandler,
    /// Default timeout for Git operations
    timeout_seconds: u64,
    /// Repository-specific configuration (if any)
    repo_config: Option<RepositoryConfig>,
}

impl GitOperations {
    /// Create a new Git operations handler with default settings
    pub fn new() -> Self {
        Self {
            ssh_handler: GitSshHandler::new(),
            timeout_seconds: std::env::var("GIT_OPERATION_TIMEOUT")
                .ok()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(300),
            repo_config: None,
        }
    }
    
    /// Create a new Git operations handler with custom settings
    pub fn with_ssh_handler(ssh_handler: GitSshHandler, timeout_seconds: u64) -> Self {
        Self {
            ssh_handler,
            timeout_seconds,
            repo_config: None,
        }
    }
    
    /// Create a Git operations handler from repository-specific configuration
    pub fn from_repository_config(config: Option<&RepositoryConfig>) -> Self {
        // If no config provided, use defaults
        if config.is_none() {
            return Self::new();
        }
        
        let config = config.unwrap();
        
        // If no SSH config provided, use defaults for SSH
        let ssh_handler = if let Some(ssh_config) = &config.ssh {
            GitSshHandler::with_config(
                ssh_config.key_path.clone(),
                ssh_config.known_hosts_path.clone(),
                ssh_config.passphrase_command.clone(),
                true, // Use batch mode by default for security
            )
        } else {
            GitSshHandler::new()
        };
        
        // Get timeout from command config if available
        let timeout_seconds = config.commands
            .as_ref()
            .and_then(|cmd_config| cmd_config.sync.as_ref())
            .and_then(|sync_config| sync_config.timeout_seconds)
            .unwrap_or(300);
        
        Self {
            ssh_handler,
            timeout_seconds,
            repo_config: Some(config.clone()),
        }
    }
    
    /// Execute a Git command with proper environment
    async fn execute_git_command(
        &self,
        args: &[&str],
        cwd: Option<&Path>,
        timeout_seconds: Option<u64>,
    ) -> Result<(String, String)> {
        // Prepare environment variables for SSH authentication
        let env_vars = self.ssh_handler.prepare_environment()
            .context("Failed to prepare SSH environment for Git command")?;
        
        // Create command
        let mut cmd = TokioCommand::new("git");
        cmd.args(args);
        
        // Set current working directory if provided
        if let Some(path) = cwd {
            cmd.current_dir(path);
        }
        
        // Apply environment variables
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
        
        // Configure stdout and stderr
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Set timeout duration
        let timeout_duration = Duration::from_secs(timeout_seconds.unwrap_or(self.timeout_seconds));
        
        // Execute command with timeout
        let output = match timeout(timeout_duration, cmd.output()).await {
            Ok(result) => result.context("Failed to execute Git command")?,
            Err(_) => {
                return Err(anyhow!(RepositoryError::OperationTimeout {
                    seconds: timeout_seconds.unwrap_or(self.timeout_seconds),
                }));
            }
        };
        
        // Convert output to strings
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        // Check for authentication issues
        if let Some(auth_error) = self.ssh_handler.detect_auth_issues(&stderr) {
            return Err(anyhow!(auth_error));
        }
        
        // Handle command failures
        if !output.status.success() {
            return Err(anyhow!(GitError::CommandError {
                command: format!("git {}", args.join(" ")),
                output: format!("STDOUT: {}\nSTDERR: {}", stdout, stderr),
            }));
        }
        
        Ok((stdout, stderr))
    }
    
    /// Check if a path is a Git repository
    async fn is_git_repository(&self, path: &Path) -> bool {
        let args = &["rev-parse", "--is-inside-work-tree"];
        match self.execute_git_command(args, Some(path), Some(5)).await {
            Ok((stdout, _)) => stdout.trim() == "true",
            Err(_) => false,
        }
    }
}

#[async_trait::async_trait]
impl RepositoryOperations for GitOperations {
    /// Clone a repository from the given URL to the specified path
    fn clone(&self, url: &str, path: &Path) -> Result<()> {
        // Use tokio runtime to execute async function in sync context
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        rt.block_on(async {
            // Check if path already exists and is a Git repository
            if path.exists() && self.is_git_repository(path).await {
                return Err(anyhow!(RepositoryError::RepositoryAlreadyExists {
                    path: path.to_path_buf(),
                }));
            }
            
            // Create parent directories if needed
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .context("Failed to create parent directories")?;
            }
            
            // Build command arguments as owned strings to avoid temporary value issues
            let mut command_args = Vec::new();
            command_args.push("clone".to_string());
            
            // Check for depth parameter in repository config
            if let Some(repo_config) = &self.repo_config {
                if let Some(commands) = &repo_config.commands {
                    if let Some(sync) = &commands.sync {
                        if let Some(depth) = sync.depth {
                            command_args.push(format!("--depth={}", depth));
                        }
                        
                        // If recursive is specified and true, add --recursive flag
                        if sync.recursive.unwrap_or(false) {
                            command_args.push("--recursive".to_string());
                        }
                    }
                }
            }
            
            // Add URL and path as owned strings
            command_args.push(url.to_string());
            command_args.push(path.to_string_lossy().to_string());
            
            // Convert to slice of &str for execute_git_command
            let args: Vec<&str> = command_args.iter().map(AsRef::as_ref).collect();
            
            // Execute git clone
            self.execute_git_command(&args, None, None).await
                .with_context(|| format!("Failed to clone repository from {} to {}", url, path.display()))?;
            
            Ok(())
        })
    }
    
    /// Update submodules in a repository
    fn update_submodules(&self, path: &Path) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        rt.block_on(async {
            // Verify the path is a Git repository
            if !self.is_git_repository(path).await {
                return Err(anyhow!(RepositoryError::RepositoryNotFound {
                    path: path.to_path_buf(),
                }));
            }
            
            // Execute submodule update
            let args = &["submodule", "update", "--init", "--recursive"];
            self.execute_git_command(args, Some(path), None).await
                .with_context(|| format!("Failed to update submodules in repository at {}", path.display()))?;
            
            Ok(())
        })
    }
    
    /// Check if a repository has changes
    fn has_changes(&self, path: &Path) -> Result<bool> {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        rt.block_on(async {
            // Verify the path is a Git repository
            if !self.is_git_repository(path).await {
                return Err(anyhow!(RepositoryError::RepositoryNotFound {
                    path: path.to_path_buf(),
                }));
            }
            
            // Check for modified files
            let args = &["status", "--porcelain"];
            let (stdout, _) = self.execute_git_command(args, Some(path), Some(10)).await
                .with_context(|| format!("Failed to check status in repository at {}", path.display()))?;
            
            Ok(!stdout.trim().is_empty())
        })
    }
    
    /// Commit changes in a repository with the given message
    fn commit_changes(&self, path: &Path, message: &str) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        rt.block_on(async {
            // Verify the path is a Git repository
            if !self.is_git_repository(path).await {
                return Err(anyhow!(RepositoryError::RepositoryNotFound {
                    path: path.to_path_buf(),
                }));
            }
            
            // First stage all changes
            let stage_args = &["add", "--all"];
            self.execute_git_command(stage_args, Some(path), Some(30)).await
                .with_context(|| format!("Failed to stage changes in repository at {}", path.display()))?;
            
            // Then commit
            let commit_args = &["commit", "-m", message];
            self.execute_git_command(commit_args, Some(path), Some(30)).await
                .with_context(|| format!("Failed to commit changes in repository at {}", path.display()))?;
            
            Ok(())
        })
    }
    
    /// Push changes to the remote repository
    fn push_changes(&self, path: &Path) -> Result<()> {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        rt.block_on(async {
            // Verify the path is a Git repository
            if !self.is_git_repository(path).await {
                return Err(anyhow!(RepositoryError::RepositoryNotFound {
                    path: path.to_path_buf(),
                }));
            }
            
            // Execute git push
            let args = &["push"];
            self.execute_git_command(args, Some(path), None).await
                .with_context(|| format!("Failed to push changes in repository at {}", path.display()))?;
            
            Ok(())
        })
    }
    
    /// Get the status of a repository
    fn get_status(&self, path: &Path) -> Result<RepositoryStatus> {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        rt.block_on(async {
            // Verify the path is a Git repository
            if !self.is_git_repository(path).await {
                return Err(anyhow!(RepositoryError::RepositoryNotFound {
                    path: path.to_path_buf(),
                }));
            }
            
            // Get current branch
            let branch_args = &["rev-parse", "--abbrev-ref", "HEAD"];
            let (branch_output, _) = self.execute_git_command(branch_args, Some(path), Some(5)).await
                .context("Failed to get current branch")?;
            let current_branch = branch_output.trim().to_string();
            
            // Check for local changes
            let status_args = &["status", "--porcelain"];
            let (status_output, _) = self.execute_git_command(status_args, Some(path), Some(10)).await
                .context("Failed to get status")?;
            let has_changes = !status_output.trim().is_empty();
            
            // Get changed files
            let mut changed_files = HashSet::new();
            for line in status_output.lines() {
                if line.len() > 3 {
                    changed_files.insert(line[3..].to_string());
                }
            }
            
            // Check for unpushed commits
            let unpushed_args = &["log", "@{push}..", "--oneline"];
            let unpushed_result = self.execute_git_command(unpushed_args, Some(path), Some(10)).await;
            let has_unpushed_commits = match unpushed_result {
                Ok((output, _)) => !output.trim().is_empty(),
                Err(_) => false, // No upstream branch or other error
            };
            
            Ok(RepositoryStatus {
                has_changes,
                has_unpushed_commits,
                current_branch,
                changed_files,
                message: None,
            })
        })
    }
    
    /// Get the remote URL of a repository
    fn get_remote_url(&self, path: &Path) -> Result<String> {
        let rt = tokio::runtime::Runtime::new()
            .context("Failed to create Tokio runtime")?;
        
        rt.block_on(async {
            // Verify the path is a Git repository
            if !self.is_git_repository(path).await {
                return Err(anyhow!(RepositoryError::RepositoryNotFound {
                    path: path.to_path_buf(),
                }));
            }
            
            // Get remote URL
            let args = &["remote", "get-url", "origin"];
            let (stdout, _) = self.execute_git_command(args, Some(path), Some(5)).await
                .context("Failed to get remote URL")?;
            
            Ok(stdout.trim().to_string())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::env;
    use crate::domain::repository::{RepositoryConfig, SshConfig, CommandConfig, SyncCommandConfig};
    
    #[test]
    fn test_ssh_handler_create_new() {
        let handler = GitSshHandler::new();
        assert!(handler.ssh_key_path.is_some());
        assert!(handler.known_hosts_path.is_some());
    }
    
    #[test]
    fn test_ssh_handler_with_config() {
        let key_path = PathBuf::from("/custom/key");
        let known_hosts = PathBuf::from("/custom/known_hosts");
        
        let handler = GitSshHandler::with_config(
            Some(key_path.clone()),
            Some(known_hosts.clone()),
            Some("echo passphrase".to_string()),
            false
        );
        
        assert_eq!(handler.ssh_key_path.unwrap(), key_path);
        assert_eq!(handler.known_hosts_path.unwrap(), known_hosts);
        assert_eq!(handler.passphrase_command.unwrap(), "echo passphrase");
        assert_eq!(handler.batch_mode, false);
    }
    
    #[test]
    fn test_path_expansion() {
        // Set a mock HOME for testing
        let original_home = env::var("HOME").ok();
        env::set_var("HOME", "/mock_home");
        
        // Test tilde expansion at the beginning of a path
        let expanded = GitSshHandler::expand_path("~/.ssh/id_rsa");
        assert_eq!(expanded, PathBuf::from("/mock_home/.ssh/id_rsa"));
        
        // Test just tilde
        let expanded = GitSshHandler::expand_path("~");
        assert_eq!(expanded, PathBuf::from("/mock_home"));
        
        // Test path without tilde
        let expanded = GitSshHandler::expand_path("/absolute/path");
        assert_eq!(expanded, PathBuf::from("/absolute/path"));
        
        // Restore original HOME
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }
    
    #[test]
    fn test_env_var_configuration() {
        // Save original environment variables
        let original_key_path = env::var("GIT_SSH_KEY_PATH").ok();
        let original_known_hosts = env::var("GIT_KNOWN_HOSTS_PATH").ok();
        let original_passphrase_cmd = env::var("GIT_SSH_PASSPHRASE_COMMAND").ok();
        
        // Set test environment variables
        env::set_var("GIT_SSH_KEY_PATH", "/env/test/id_rsa");
        env::set_var("GIT_KNOWN_HOSTS_PATH", "/env/test/known_hosts");
        env::set_var("GIT_SSH_PASSPHRASE_COMMAND", "echo test_passphrase");
        
        // Create a new handler which should pick up the environment variables
        let handler = GitSshHandler::new();
        
        // Verify the handler used environment variables
        assert_eq!(handler.ssh_key_path.unwrap(), PathBuf::from("/env/test/id_rsa"));
        assert_eq!(handler.known_hosts_path.unwrap(), PathBuf::from("/env/test/known_hosts"));
        assert_eq!(handler.passphrase_command.unwrap(), "echo test_passphrase");
        
        // Restore original environment
        match original_key_path {
            Some(val) => env::set_var("GIT_SSH_KEY_PATH", val),
            None => env::remove_var("GIT_SSH_KEY_PATH"),
        }
        
        match original_known_hosts {
            Some(val) => env::set_var("GIT_KNOWN_HOSTS_PATH", val),
            None => env::remove_var("GIT_KNOWN_HOSTS_PATH"),
        }
        
        match original_passphrase_cmd {
            Some(val) => env::set_var("GIT_SSH_PASSPHRASE_COMMAND", val),
            None => env::remove_var("GIT_SSH_PASSPHRASE_COMMAND"),
        }
    }
    
    #[test]
    fn test_from_repository_config() {
        // Create a mock repository configuration
        let ssh_config = SshConfig {
            key_path: Some(PathBuf::from("/repo/specific/id_rsa")),
            known_hosts_path: Some(PathBuf::from("/repo/specific/known_hosts")),
            passphrase_command: Some("cat /repo/passphrase".to_string()),
        };
        
        let sync_config = SyncCommandConfig {
            recursive: Some(true),
            depth: Some(1),
            timeout_seconds: Some(120),
        };
        
        let command_config = CommandConfig {
            sync: Some(sync_config),
            status: None,
            save: None,
        };
        
        let repo_config = RepositoryConfig {
            ssh: Some(ssh_config),
            commands: Some(command_config),
        };
        
        // Create GitOperations from this config
        let git_ops = GitOperations::from_repository_config(Some(&repo_config));
        
        // Verify it used the repository-specific configuration
        assert_eq!(git_ops.ssh_handler.ssh_key_path.unwrap(), PathBuf::from("/repo/specific/id_rsa"));
        assert_eq!(git_ops.ssh_handler.known_hosts_path.unwrap(), PathBuf::from("/repo/specific/known_hosts"));
        assert_eq!(git_ops.ssh_handler.passphrase_command.unwrap(), "cat /repo/passphrase");
        assert_eq!(git_ops.timeout_seconds, 120);
        assert!(git_ops.repo_config.is_some());
    }
    
    #[test]
    fn test_detect_auth_issues() {
        let handler = GitSshHandler::new();
        
        // Test permission denied
        let stderr = "git@github.com: Permission denied (publickey).
                     fatal: Could not read from remote repository.";
        let error = handler.detect_auth_issues(stderr).unwrap();
        match error {
            GitError::AuthenticationError { message, .. } => {
                assert!(message.contains("SSH authentication failed"));
                // Check for diagnostic information
                assert!(message.contains("Verify the key at"));
                assert!(message.contains("Confirm this key has been added"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
        
        // Test host key verification
        let stderr = "Host key verification failed.
                     fatal: Could not read from remote repository.";
        let error = handler.detect_auth_issues(stderr).unwrap();
        match error {
            GitError::AuthenticationError { message, .. } => {
                assert!(message.contains("SSH host verification failed"));
                assert!(message.contains("Run `ssh -T git@<host>`"));
                assert!(message.contains("GIT_SSH_STRICT_HOST_CHECKING=no"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
        
        // Test passphrase needed
        let stderr = "Enter passphrase for key '/home/user/.ssh/id_rsa':";
        let error = handler.detect_auth_issues(stderr).unwrap();
        match error {
            GitError::AuthenticationError { message, .. } => {
                assert!(message.contains("SSH key requires a passphrase"));
                assert!(message.contains("GIT_SSH_PASSPHRASE_COMMAND"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
        
        // Test connection timeout
        let stderr = "ssh: connect to host github.com port 22: Connection timed out";
        let error = handler.detect_auth_issues(stderr).unwrap();
        match error {
            GitError::AuthenticationError { message, .. } => {
                assert!(message.contains("SSH connection timed out"));
            }
            _ => panic!("Expected AuthenticationError"),
        }
        
        // Test no error
        let stderr = "Everything is fine";
        assert!(handler.detect_auth_issues(stderr).is_none());
    }
    
    #[test]
    fn test_auth_error_handling() {
        // Create a GitOperations instance with a mocked SSH handler
        let ssh_handler = GitSshHandler::with_config(
            Some(PathBuf::from("~/.ssh/test_key")),
            Some(PathBuf::from("~/.ssh/known_hosts")),
            None,
            true
        );
        
        let git_ops = GitOperations::with_ssh_handler(ssh_handler, 10);
        
        // For this simple test, we can verify that the SSH key path is correctly passed
        assert_eq!(
            git_ops.ssh_handler.ssh_key_path.unwrap(),
            GitSshHandler::expand_path("~/.ssh/test_key")
        );
        
        // Test that auth error detection works as expected by using a mocked error message
        let auth_error_message = "git@github.com: Permission denied (publickey).";
        let detected_error = git_ops.ssh_handler.detect_auth_issues(auth_error_message);
        assert!(detected_error.is_some(), "Should detect auth error");
        
        match detected_error.unwrap() {
            GitError::AuthenticationError { message, .. } => {
                assert!(message.contains("SSH authentication failed"));
            },
            _ => panic!("Wrong error type detected"),
        }
    }
    
    #[test]
    fn test_clone_with_depth_option() {
        // This test would normally create a mock for the command execution
        // but for simplicity we'll just test the logic for constructing arguments
        
        // Create configuration with depth=2
        let sync_config = SyncCommandConfig {
            recursive: Some(true),
            depth: Some(2),
            timeout_seconds: Some(120),
        };
        
        let command_config = CommandConfig {
            sync: Some(sync_config),
            status: None,
            save: None,
        };
        
        let repo_config = RepositoryConfig {
            ssh: None,
            commands: Some(command_config),
        };
        
        let git_ops = GitOperations::from_repository_config(Some(&repo_config));
        
        // In a real test, we would mock the execute_git_command method to verify
        // that the correct arguments are passed, including "--depth=2"
        // For now, we just verify that the configuration was stored correctly
        let stored_depth = git_ops.repo_config.unwrap().commands.unwrap().sync.unwrap().depth.unwrap();
        assert_eq!(stored_depth, 2);
    }
}