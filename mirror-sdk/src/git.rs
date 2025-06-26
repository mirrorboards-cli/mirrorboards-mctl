//! Git Operations Module
//! 
//! Provides Git repository cloning and update operations with robust SSH authentication
//! integration and retry limits to prevent infinite loops.

use git2::{Repository, RemoteCallbacks, FetchOptions, Cred, Diff, DiffOptions};
use crate::ssh::SshManager;
use crate::models::Repository as RepoConfig;
use crate::error::{GitError, GitResult};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Git manager that handles repository operations with SSH authentication
pub struct GitManager {
    ssh_manager: SshManager,
}

/// Repository status information
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryStatus {
    /// Repository directory doesn't exist
    Missing,
    /// Directory exists but is not a git repository
    NotGitRepository,
    /// Repository is up to date with remote
    UpToDate,
    /// Repository needs to be updated (behind remote)
    NeedsUpdate,
    /// Repository has local changes that conflict with remote
    HasConflicts,
}

/// Detailed file status information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStatus {
    /// File path relative to repository root
    pub path: String,
    /// Status of the file in the working directory
    pub working_dir_status: FileChangeType,
    /// Status of the file in the index (staging area)
    pub index_status: FileChangeType,
}

/// Type of change for a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChangeType {
    /// File is unmodified
    Unmodified,
    /// File is new/untracked
    New,
    /// File is modified
    Modified,
    /// File is deleted
    Deleted,
    /// File is renamed
    Renamed,
    /// File is copied
    Copied,
    /// File is ignored
    Ignored,
}

/// Detailed repository status including file-level changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetailedRepositoryStatus {
    /// Overall repository status
    pub status: RepositoryStatus,
    /// List of files with changes (only populated if repository has git status)
    pub files: Vec<FileStatus>,
}

/// Repository diff information
#[derive(Debug, Clone)]
pub struct RepositoryDiff {
    /// Working directory diff (unstaged changes)
    pub working_diff: Option<String>,
    /// Staged diff (index changes)
    pub staged_diff: Option<String>,
}

impl GitManager {
    /// Create a new GitManager with SSH authentication support
    pub fn new() -> GitResult<Self> {
        Self::new_with_verbose(false)
    }
    
    /// Create a new GitManager with optional verbose output
    pub fn new_with_verbose(verbose: bool) -> GitResult<Self> {
        let ssh_manager = SshManager::new_with_verbose(verbose)
            .map_err(|e| GitError::SshError { source: e })?;
        
        Ok(GitManager { ssh_manager })
    }
    
    /// Clone a repository to the specified target path
    pub fn clone_repository(&self, repo: &RepoConfig, target_path: &Path) -> GitResult<()> {
        println!("Cloning repository: {} -> {}", repo.git, target_path.display());
        
        // Create parent directories if they don't exist
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Setup progress bar
        let progress_bar = ProgressBar::new(100);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        progress_bar.set_message(format!("Cloning {}", repo.git));
        
        // Try cloning with specific branch first (if not main/master)
        if repo.branch != "main" && repo.branch != "master" {
            match self.clone_with_branch(repo, target_path, &progress_bar) {
                Ok(()) => {
                    progress_bar.finish_with_message(format!("Successfully cloned {} on branch {}", repo.git, repo.branch));
                    println!("Repository cloned successfully to {} on branch {}", target_path.display(), repo.branch);
                    return Ok(());
                }
                Err(e) => {
                    // Check if this is a branch-specific error
                    if self.is_branch_error(&e) {
                        println!("Branch '{}' not found remotely, falling back to default branch clone", repo.branch);
                        progress_bar.set_message(format!("Fallback: cloning default branch for {}", repo.git));
                    } else {
                        // Not a branch error, propagate it
                        progress_bar.finish_with_message("Clone failed");
                        return Err(e);
                    }
                }
            }
        }
        
        // Fallback: Clone without specifying branch (gets default branch)
        match self.clone_without_branch(repo, target_path, &progress_bar) {
            Ok(repository) => {
                // If we need a specific branch that's not main/master, create it locally
                if repo.branch != "main" && repo.branch != "master" {
                    if let Err(e) = self.create_local_branch(&repository, &repo.branch) {
                        println!("Warning: Failed to create local branch '{}': {}", repo.branch, e);
                        // Don't fail the entire operation, just log the warning
                    } else {
                        println!("Created and switched to local branch '{}'", repo.branch);
                    }
                }
                
                progress_bar.finish_with_message(format!("Successfully cloned {}", repo.git));
                println!("Repository cloned successfully to {}", target_path.display());
                Ok(())
            }
            Err(e) => {
                progress_bar.finish_with_message("Clone failed");
                Err(e)
            }
        }
    }
    
    /// Update a repository by pulling the latest changes
    pub fn update_repository(&self, repo_path: &Path) -> GitResult<()> {
        println!("Updating repository: {}", repo_path.display());
        
        // Open the repository
        let repo = Repository::open(repo_path)
            .map_err(|_e| GitError::RepositoryNotFound {
                path: repo_path.to_path_buf()
            })?;
        
        // Get the current branch
        let head = repo.head()
            .map_err(|e| GitError::InvalidState { 
                message: format!("Failed to get HEAD: {}", e) 
            })?;
        
        let branch_name = head.shorthand().unwrap_or("HEAD");
        println!("Current branch: {}", branch_name);
        
        // Setup progress bar
        let progress_bar = ProgressBar::new(100);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        progress_bar.set_message("Fetching updates...");
        
        // Setup fetch options with authentication
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        
        // Setup credentials callback
        let credentials_callback = self.setup_credentials_callback();
        callbacks.credentials(credentials_callback);
        
        // Setup progress callback
        callbacks.transfer_progress(|stats| {
            let pct = if stats.total_objects() > 0 {
                (100 * stats.received_objects()) / stats.total_objects()
            } else {
                0
            };
            progress_bar.set_position(pct as u64);
            progress_bar.set_message(format!("Receiving objects: {}%", pct));
            true
        });
        
        fetch_options.remote_callbacks(callbacks);
        
        // Fetch from origin
        let mut remote = repo.find_remote("origin")
            .map_err(|e| GitError::RemoteFailed { 
                message: format!("Failed to find origin remote: {}", e) 
            })?;
        
        remote.fetch(&[branch_name], Some(&mut fetch_options), None)
            .map_err(|e| GitError::PullFailed { 
                path: repo_path.to_path_buf(),
                message: format!("Fetch failed: {}", e)
            })?;
        
        progress_bar.finish_with_message("Fetch completed");
        
        // Get the remote tracking branch
        let remote_branch_name = format!("origin/{}", branch_name);
        let remote_oid = repo.refname_to_id(&format!("refs/remotes/{}", remote_branch_name))
            .map_err(|e| GitError::InvalidState { 
                message: format!("Failed to get remote branch OID: {}", e) 
            })?;
        
        let local_oid = head.target().unwrap();
        
        // Check if update is needed
        if local_oid == remote_oid {
            println!("Repository is already up to date");
            return Ok(());
        }
        
        // Perform merge (fast-forward if possible)
        let remote_annotated_commit = repo.find_annotated_commit(remote_oid)
            .map_err(|e| GitError::InvalidState { 
                message: format!("Failed to find remote commit: {}", e) 
            })?;
        
        let analysis = repo.merge_analysis(&[&remote_annotated_commit])
            .map_err(|e| GitError::OperationFailed { 
                message: format!("Merge analysis failed: {}", e) 
            })?;
        
        if analysis.0.is_fast_forward() {
            println!("Performing fast-forward merge");
            let refname = format!("refs/heads/{}", branch_name);
            let mut reference = repo.find_reference(&refname)
                .map_err(|e| GitError::InvalidState { 
                    message: format!("Failed to find branch reference: {}", e) 
                })?;
            
            reference.set_target(remote_oid, "Fast-forward")
                .map_err(|e| GitError::OperationFailed { 
                    message: format!("Fast-forward failed: {}", e) 
                })?;
            
            repo.set_head(&refname)
                .map_err(|e| GitError::InvalidState { 
                    message: format!("Failed to set HEAD: {}", e) 
                })?;
            
            repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
                .map_err(|e| GitError::OperationFailed { 
                    message: format!("Checkout failed: {}", e) 
                })?;
            
            println!("Repository updated successfully");
            Ok(())
        } else if analysis.0.is_normal() {
            Err(GitError::InvalidState { 
                message: "Repository requires manual merge - has local changes".to_string() 
            })
        } else {
            Err(GitError::InvalidState { 
                message: "Repository is in an invalid state for update".to_string() 
            })
        }
    }
    
    /// Get the status of a repository
    pub fn get_repository_status(&self, repo_path: &Path) -> GitResult<RepositoryStatus> {
        // Check if path exists
        if !repo_path.exists() {
            return Ok(RepositoryStatus::Missing);
        }
        
        // Check if it's a git repository
        let repo = match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(_) => return Ok(RepositoryStatus::NotGitRepository),
        };
        
        // Get current HEAD
        let head = match repo.head() {
            Ok(head) => head,
            Err(_) => return Ok(RepositoryStatus::NotGitRepository),
        };
        
        let local_oid = match head.target() {
            Some(oid) => oid,
            None => return Ok(RepositoryStatus::NotGitRepository),
        };
        
        // Try to get remote tracking branch
        let branch_name = head.shorthand().unwrap_or("HEAD");
        let remote_branch_name = format!("refs/remotes/origin/{}", branch_name);
        
        let remote_oid = match repo.refname_to_id(&remote_branch_name) {
            Ok(oid) => oid,
            Err(_) => {
                // No remote tracking branch, assume up to date
                return Ok(RepositoryStatus::UpToDate);
            }
        };
        
        // Compare local and remote
        if local_oid == remote_oid {
            Ok(RepositoryStatus::UpToDate)
        } else {
            // Check if local has uncommitted changes
            let statuses = repo.statuses(None)
                .map_err(|e| GitError::OperationFailed {
                    message: format!("Failed to get repository status: {}", e)
                })?;
            
            if !statuses.is_empty() {
                Ok(RepositoryStatus::HasConflicts)
            } else {
                Ok(RepositoryStatus::NeedsUpdate)
            }
        }
    }
    
    /// Get detailed repository status including file-level changes
    pub fn get_detailed_repository_status(&self, repo_path: &Path) -> GitResult<DetailedRepositoryStatus> {
        // Check if path exists
        if !repo_path.exists() {
            return Ok(DetailedRepositoryStatus {
                status: RepositoryStatus::Missing,
                files: Vec::new(),
            });
        }
        
        // Check if it's a git repository
        let repo = match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(_) => return Ok(DetailedRepositoryStatus {
                status: RepositoryStatus::NotGitRepository,
                files: Vec::new(),
            }),
        };
        
        // Get current HEAD
        let head = match repo.head() {
            Ok(head) => head,
            Err(_) => return Ok(DetailedRepositoryStatus {
                status: RepositoryStatus::NotGitRepository,
                files: Vec::new(),
            }),
        };
        
        let local_oid = match head.target() {
            Some(oid) => oid,
            None => return Ok(DetailedRepositoryStatus {
                status: RepositoryStatus::NotGitRepository,
                files: Vec::new(),
            }),
        };
        
        // Get file status information
        let statuses = repo.statuses(None)
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to get repository status: {}", e)
            })?;
        
        let mut files = Vec::new();
        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                let status = entry.status();
                
                // Convert git2::Status flags to our FileChangeType
                let working_dir_status = if status.is_wt_new() {
                    FileChangeType::New
                } else if status.is_wt_modified() {
                    FileChangeType::Modified
                } else if status.is_wt_deleted() {
                    FileChangeType::Deleted
                } else if status.is_wt_renamed() {
                    FileChangeType::Renamed
                } else if status.is_ignored() {
                    FileChangeType::Ignored
                } else {
                    FileChangeType::Unmodified
                };
                
                let index_status = if status.is_index_new() {
                    FileChangeType::New
                } else if status.is_index_modified() {
                    FileChangeType::Modified
                } else if status.is_index_deleted() {
                    FileChangeType::Deleted
                } else if status.is_index_renamed() {
                    FileChangeType::Renamed
                } else {
                    FileChangeType::Unmodified
                };
                
                files.push(FileStatus {
                    path: path.to_string(),
                    working_dir_status,
                    index_status,
                });
            }
        }
        
        // Try to get remote tracking branch
        let branch_name = head.shorthand().unwrap_or("HEAD");
        let remote_branch_name = format!("refs/remotes/origin/{}", branch_name);
        
        let remote_oid = match repo.refname_to_id(&remote_branch_name) {
            Ok(oid) => oid,
            Err(_) => {
                // No remote tracking branch, assume up to date
                return Ok(DetailedRepositoryStatus {
                    status: RepositoryStatus::UpToDate,
                    files,
                });
            }
        };
        
        // Determine overall status
        let status = if local_oid == remote_oid {
            RepositoryStatus::UpToDate
        } else {
            if !statuses.is_empty() {
                RepositoryStatus::HasConflicts
            } else {
                RepositoryStatus::NeedsUpdate
            }
        };
        
        Ok(DetailedRepositoryStatus {
            status,
            files,
        })
    }
    
    /// Get working directory diff (unstaged changes)
    pub fn get_working_directory_diff(&self, repo_path: &Path) -> GitResult<Option<String>> {
        // Check if path exists and is a git repository
        if !repo_path.exists() {
            return Ok(None);
        }
        
        let repo = match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(_) => return Ok(None),
        };
        
        // Refresh git index to ensure synchronization
        let mut index = repo.index()
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to get repository index: {}", e)
            })?;
        
        // Force index refresh from disk
        index.read(true)
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to refresh git index: {}", e)
            })?;
        
        // Get the current HEAD tree explicitly
        let head_tree = match repo.head() {
            Ok(head) => {
                let head_commit = head.peel_to_commit()
                    .map_err(|e| GitError::OperationFailed {
                        message: format!("Failed to get HEAD commit: {}", e)
                    })?;
                Some(head_commit.tree()
                    .map_err(|e| GitError::OperationFailed {
                        message: format!("Failed to get HEAD tree: {}", e)
                    })?)
            }
            Err(_) => None, // No HEAD (empty repository)
        };
        
        // Get diff between HEAD and working directory with proper options
        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);
        diff_options.interhunk_lines(0);
        // Ignore whitespace and line ending issues to prevent false positives
        diff_options.ignore_whitespace_change(true);
        diff_options.ignore_whitespace_eol(true);
        
        let diff = repo.diff_tree_to_workdir_with_index(head_tree.as_ref(), Some(&mut diff_options))
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to get working directory diff: {}", e)
            })?;
        
        self.format_diff(diff)
    }
    
    /// Get staged diff (index changes)
    pub fn get_staged_diff(&self, repo_path: &Path) -> GitResult<Option<String>> {
        // Check if path exists and is a git repository
        if !repo_path.exists() {
            return Ok(None);
        }
        
        let repo = match Repository::open(repo_path) {
            Ok(repo) => repo,
            Err(_) => return Ok(None),
        };
        
        // Refresh git index to ensure synchronization
        let mut index = repo.index()
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to get repository index: {}", e)
            })?;
        
        // Force index refresh from disk
        index.read(true)
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to refresh git index: {}", e)
            })?;
        
        // Get the current HEAD tree
        let head_tree = match repo.head() {
            Ok(head) => {
                let head_commit = head.peel_to_commit()
                    .map_err(|e| GitError::OperationFailed {
                        message: format!("Failed to get HEAD commit: {}", e)
                    })?;
                Some(head_commit.tree()
                    .map_err(|e| GitError::OperationFailed {
                        message: format!("Failed to get HEAD tree: {}", e)
                    })?)
            }
            Err(_) => None, // No HEAD (empty repository)
        };
        
        // Get diff between HEAD and index with proper options
        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);
        diff_options.interhunk_lines(0);
        // Ignore whitespace and line ending issues to prevent false positives
        diff_options.ignore_whitespace_change(true);
        diff_options.ignore_whitespace_eol(true);
        
        let diff = repo.diff_tree_to_index(head_tree.as_ref(), None, Some(&mut diff_options))
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to get staged diff: {}", e)
            })?;
        
        self.format_diff(diff)
    }
    
    /// Get combined repository diff (both working directory and staged)
    pub fn get_repository_diff(&self, repo_path: &Path) -> GitResult<RepositoryDiff> {
        let working_diff = self.get_working_directory_diff(repo_path)?;
        let staged_diff = self.get_staged_diff(repo_path)?;
        
        Ok(RepositoryDiff {
            working_diff,
            staged_diff,
        })
    }
    
    /// Format a git2::Diff into a string representation
    fn format_diff(&self, diff: Diff) -> GitResult<Option<String>> {
        let mut diff_output = Vec::<u8>::new();
        
        // Check if there are any changes
        if diff.deltas().len() == 0 {
            return Ok(None);
        }
        
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                '+' | '-' | ' ' => {
                    diff_output.push(line.origin() as u8);
                    diff_output.extend_from_slice(line.content());
                }
                'F' => {
                    // File header
                    diff_output.extend_from_slice(b"diff --git ");
                    diff_output.extend_from_slice(line.content());
                }
                'H' => {
                    // Hunk header
                    diff_output.extend_from_slice(line.content());
                }
                _ => {
                    diff_output.extend_from_slice(line.content());
                }
            }
            true
        }).map_err(|e| GitError::OperationFailed {
            message: format!("Failed to format diff: {}", e)
        })?;
        
        if diff_output.is_empty() {
            Ok(None)
        } else {
            String::from_utf8(diff_output)
                .map(|s| Some(s))
                .map_err(|e| GitError::OperationFailed {
                    message: format!("Failed to convert diff to string: {}", e)
                })
        }
    }
    
    /// Clone repository with specific branch
    fn clone_with_branch(&self, repo: &RepoConfig, target_path: &Path, progress_bar: &ProgressBar) -> GitResult<()> {
        // Setup Git builder with callbacks
        let mut builder = git2::build::RepoBuilder::new();
        
        // Setup fetch options with authentication
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        
        // Setup credentials callback with retry limits
        let credentials_callback = self.setup_credentials_callback();
        callbacks.credentials(credentials_callback);
        
        // Setup progress callback
        callbacks.transfer_progress(|stats| {
            let network_pct = (100 * stats.received_objects()) / stats.total_objects();
            let index_pct = (100 * stats.indexed_objects()) / stats.total_objects();
            let co_pct = if stats.total_objects() > 0 {
                (100 * stats.received_objects()) / stats.total_objects()
            } else {
                0
            };
            let kbytes = stats.received_bytes() / 1024;
            
            progress_bar.set_position(std::cmp::max(network_pct, index_pct) as u64);
            progress_bar.set_message(format!("net {network_pct}% ({kbytes} kb), idx {index_pct}%, chk {co_pct}%"));
            true
        });
        
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);
        
        // Set the specific branch
        builder.branch(&repo.branch);
        
        // Perform the clone
        match builder.clone(&repo.git, target_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(GitError::CloneFailed {
                url: repo.git.clone(),
                path: target_path.to_path_buf(),
                message: e.message().to_string(),
            })
        }
    }
    
    /// Clone repository without specifying branch (gets default branch)
    fn clone_without_branch(&self, repo: &RepoConfig, target_path: &Path, progress_bar: &ProgressBar) -> GitResult<Repository> {
        // Setup Git builder with callbacks
        let mut builder = git2::build::RepoBuilder::new();
        
        // Setup fetch options with authentication
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        
        // Setup credentials callback with retry limits
        let credentials_callback = self.setup_credentials_callback();
        callbacks.credentials(credentials_callback);
        
        // Setup progress callback
        callbacks.transfer_progress(|stats| {
            let network_pct = (100 * stats.received_objects()) / stats.total_objects();
            let index_pct = (100 * stats.indexed_objects()) / stats.total_objects();
            let co_pct = if stats.total_objects() > 0 {
                (100 * stats.received_objects()) / stats.total_objects()
            } else {
                0
            };
            let kbytes = stats.received_bytes() / 1024;
            
            progress_bar.set_position(std::cmp::max(network_pct, index_pct) as u64);
            progress_bar.set_message(format!("net {network_pct}% ({kbytes} kb), idx {index_pct}%, chk {co_pct}%"));
            true
        });
        
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);
        
        // Don't set a specific branch - let it use the default
        
        // Perform the clone
        match builder.clone(&repo.git, target_path) {
            Ok(repository) => Ok(repository),
            Err(e) => Err(GitError::CloneFailed {
                url: repo.git.clone(),
                path: target_path.to_path_buf(),
                message: e.message().to_string(),
            })
        }
    }
    
    /// Create a local branch and switch to it
    fn create_local_branch(&self, repository: &Repository, branch_name: &str) -> GitResult<()> {
        // Get the current HEAD commit
        let head_commit = repository.head()
            .map_err(|e| GitError::InvalidState {
                message: format!("Failed to get HEAD: {}", e)
            })?
            .peel_to_commit()
            .map_err(|e| GitError::InvalidState {
                message: format!("Failed to get HEAD commit: {}", e)
            })?;
        
        // Check if branch already exists locally
        let branch_ref_name = format!("refs/heads/{}", branch_name);
        if repository.find_reference(&branch_ref_name).is_ok() {
            // Branch exists, just switch to it
            println!("Local branch '{}' already exists, switching to it", branch_name);
        } else {
            // Create the new branch
            repository.branch(branch_name, &head_commit, false)
                .map_err(|e| GitError::OperationFailed {
                    message: format!("Failed to create branch '{}': {}", branch_name, e)
                })?;
            
            println!("Created local branch '{}'", branch_name);
        }
        
        // Switch to the branch
        repository.set_head(&branch_ref_name)
            .map_err(|e| GitError::InvalidState {
                message: format!("Failed to set HEAD to branch '{}': {}", branch_name, e)
            })?;
        
        // Update working directory to match the branch
        repository.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| GitError::OperationFailed {
                message: format!("Failed to checkout branch '{}': {}", branch_name, e)
            })?;
        
        Ok(())
    }
    
    /// Check if the error is related to branch not existing
    fn is_branch_error(&self, error: &GitError) -> bool {
        match error {
            GitError::CloneFailed { message, .. } => {
                // Common patterns for branch-related errors
                message.contains("Remote branch") ||
                message.contains("not found") ||
                message.contains("does not exist") ||
                message.contains("couldn't find remote ref") ||
                message.contains("reference is not valid")
            }
            _ => false,
        }
    }
    
    /// Setup credentials callback with retry limits and SSH fallback chain
    fn setup_credentials_callback(&self) -> impl FnMut(&str, Option<&str>, git2::CredentialType) -> Result<Cred, git2::Error> + '_ {
        let retry_count = Arc::new(AtomicUsize::new(0));
        let max_retries = 3; // Maximum authentication attempts
        
        move |url: &str, username: Option<&str>, allowed_types: git2::CredentialType| {
            let current_retry = retry_count.fetch_add(1, Ordering::SeqCst);
            
            println!("Authentication attempt {} for {}", current_retry + 1, url);
            
            // Check if we've exceeded max retries
            if current_retry >= max_retries {
                println!("Max authentication attempts ({}) exceeded", max_retries);
                return Err(git2::Error::from_str("Max authentication attempts exceeded"));
            }
            
            let username = username.unwrap_or("git");
            
            // Try SSH authentication if supported
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                println!("Attempting SSH key authentication for user: {}", username);
                
                // Try SSH agent first (if available and usable)
                if self.ssh_manager.has_usable_agent() {
                    println!("Trying SSH agent authentication");
                    match Cred::ssh_key_from_agent(username) {
                        Ok(cred) => {
                            println!("SSH agent authentication successful");
                            return Ok(cred);
                        }
                        Err(e) => {
                            println!("SSH agent authentication failed: {}", e);
                        }
                    }
                }
                
                // Try filesystem keys
                let available_keys = self.ssh_manager.get_available_keys();
                for key_path in available_keys {
                    println!("Trying SSH key: {}", key_path.display());
                    
                    let public_key_path = key_path.with_extension("pub");
                    if public_key_path.exists() {
                        match Cred::ssh_key(username, Some(&public_key_path), key_path, None) {
                            Ok(cred) => {
                                println!("SSH key authentication successful with: {}", key_path.display());
                                return Ok(cred);
                            }
                            Err(e) => {
                                println!("SSH key authentication failed with {}: {}", key_path.display(), e);
                                continue;
                            }
                        }
                    } else {
                        println!("Public key not found for: {}", key_path.display());
                    }
                }
            }
            
            // If we get here, all authentication methods failed
            println!("All authentication methods failed for attempt {}", current_retry + 1);
            Err(git2::Error::from_str("Authentication failed"))
        }
    }
}

impl Default for GitManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            println!("Warning: Failed to initialize Git manager: {}", e);
            GitManager {
                ssh_manager: SshManager::default(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_git_manager_creation() {
        let result = GitManager::new();
        match result {
            Ok(_) => println!("Git manager created successfully"),
            Err(e) => println!("Git manager creation failed (expected if no SSH keys): {}", e),
        }
    }
    
    #[test]
    fn test_repository_status_missing() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent_path = temp_dir.path().join("non-existent");
        
        let git_manager = GitManager::default();
        let status = git_manager.get_repository_status(&non_existent_path).unwrap();
        assert_eq!(status, RepositoryStatus::Missing);
    }
    
    #[test]
    fn test_repository_status_not_git() {
        let temp_dir = TempDir::new().unwrap();
        let non_git_path = temp_dir.path();
        
        let git_manager = GitManager::default();
        let status = git_manager.get_repository_status(non_git_path).unwrap();
        assert_eq!(status, RepositoryStatus::NotGitRepository);
    }
}