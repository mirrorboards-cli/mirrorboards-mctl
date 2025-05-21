use crate::config::Repository;
use crate::error::{MctlError, MctlResult};
use git2::{Cred, FetchOptions, PushOptions, RemoteCallbacks, Repository as Git2Repository};
use log::{debug, error, info, warn};
use std::env;
use std::path::{Path, PathBuf};

/// Git operations for repository management
pub struct GitOperations;

impl GitOperations {
    /// Clone a repository
    pub fn clone(repository: &Repository, base_dir: &Path) -> MctlResult<()> {
        let repo_path = repository.absolute_path(base_dir);
        
        // Check if the repository already exists
        if repo_path.exists() && repo_path.join(".git").exists() {
            info!("Repository already exists at {}", repo_path.display());
            return Ok(());
        }
        
        // Create parent directories if they don't exist
        if let Some(parent) = repo_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        info!("Cloning {} to {}", repository.git_url, repo_path.display());
        
        // Set up callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            Self::credentials_callback(url, username_from_url, allowed_types)
        });
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        // Clone options
        let mut clone_options = git2::CloneOptions::new();
        clone_options.fetch_options(fetch_options);
        
        // Set branch if specified
        if let Some(branch) = &repository.branch {
            clone_options.checkout_branch(branch);
        }
        
        // Clone the repository
        match git2::Repository::clone(&repository.git_url, &repo_path) {
            Ok(_) => {
                info!("Successfully cloned repository to {}", repo_path.display());
                Ok(())
            },
            Err(e) => {
                error!("Failed to clone repository: {}", e);
                Err(MctlError::GitError(format!("Failed to clone repository: {}", e)))
            }
        }
    }
    
    /// Check the status of a repository
    pub fn status(repository: &Repository, base_dir: &Path) -> MctlResult<RepositoryStatus> {
        let repo_path = repository.absolute_path(base_dir);
        
        // Check if the repository exists
        if !repo_path.exists() || !repo_path.join(".git").exists() {
            return Err(MctlError::RepositoryNotFound(repo_path));
        }
        
        // Open the repository
        let repo = Git2Repository::open(&repo_path)?;
        
        // Get the current branch
        let head = repo.head()?;
        let branch_name = head.shorthand().unwrap_or("HEAD detached").to_string();
        
        // Check if the branch is ahead/behind remote
        let mut branch_status = "".to_string();
        if let Ok(upstream) = repo.branch_upstream_name(&head.name().unwrap_or("")) {
            if let Ok(upstream_str) = upstream.as_str() {
                if let Ok((ahead, behind)) = repo.graph_ahead_behind(
                    head.target().unwrap(),
                    repo.revparse_single(upstream_str)?.id(),
                ) {
                    if ahead > 0 && behind > 0 {
                        branch_status = format!("diverged from upstream (ahead by {}, behind by {})", ahead, behind);
                    } else if ahead > 0 {
                        branch_status = format!("ahead of upstream by {} commit(s)", ahead);
                    } else if behind > 0 {
                        branch_status = format!("behind upstream by {} commit(s)", behind);
                    } else {
                        branch_status = "up to date with upstream".to_string();
                    }
                }
            }
        }
        
        // Get the status of files
        let mut status_options = git2::StatusOptions::new();
        status_options.include_untracked(true);
        status_options.recurse_untracked_dirs(true);
        status_options.include_ignored(false);
        
        let statuses = repo.statuses(Some(&mut status_options))?;
        
        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();
        
        for entry in statuses.iter() {
            let path_str = entry.path().unwrap_or("").to_string();
            let path = repo_path.join(&path_str);
            
            if entry.status().is_wt_new() {
                untracked_files.push((path, "??".to_string()));
            } else {
                let status_code = if entry.status().is_wt_modified() {
                    "M"
                } else if entry.status().is_wt_deleted() {
                    "D"
                } else if entry.status().is_wt_renamed() {
                    "R"
                } else if entry.status().is_wt_typechange() {
                    "T"
                } else if entry.status().is_index_new() {
                    "A"
                } else if entry.status().is_index_modified() {
                    "M"
                } else if entry.status().is_index_deleted() {
                    "D"
                } else if entry.status().is_index_renamed() {
                    "R"
                } else if entry.status().is_index_typechange() {
                    "T"
                } else if entry.status().is_conflicted() {
                    "U"
                } else {
                    "?"
                };
                
                modified_files.push((path, status_code.to_string()));
            }
        }
        
        Ok(RepositoryStatus {
            path: repo_path,
            branch: branch_name,
            branch_status,
            modified_files,
            untracked_files,
            is_clean: modified_files.is_empty() && untracked_files.is_empty(),
        })
    }
    
    /// Update a repository with the latest changes
    pub fn update(repository: &Repository, base_dir: &Path, force: bool) -> MctlResult<UpdateResult> {
        let repo_path = repository.absolute_path(base_dir);
        
        // Check if the repository exists
        if !repo_path.exists() || !repo_path.join(".git").exists() {
            return Err(MctlError::RepositoryNotFound(repo_path));
        }
        
        // Open the repository
        let repo = Git2Repository::open(&repo_path)?;
        
        // Check for uncommitted changes if not force
        if !force {
            let status = Self::status(repository, base_dir)?;
            if !status.is_clean {
                return Err(MctlError::UncommittedChanges(repo_path));
            }
        }
        
        // Set up callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            Self::credentials_callback(url, username_from_url, allowed_types)
        });
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        
        // Get the remote
        let mut remote = repo.find_remote("origin")?;
        
        // Fetch from remote
        info!("Fetching latest changes for {}", repo_path.display());
        remote.fetch(&[], Some(&mut fetch_options), None)?;
        
        // Get the current branch
        let head = repo.head()?;
        let branch_name = head.shorthand().unwrap_or("HEAD detached").to_string();
        
        // Find the upstream branch
        let upstream_branch = match repo.branch_upstream_name(&head.name().unwrap_or("")) {
            Ok(name) => name,
            Err(_) => format!("refs/remotes/origin/{}", branch_name).into(),
        };
        
        // Get the upstream commit
        let upstream_commit = match repo.revparse_single(upstream_branch.as_str().unwrap_or("")) {
            Ok(obj) => obj.peel_to_commit()?,
            Err(_) => {
                warn!("Could not find upstream branch for {}", branch_name);
                return Ok(UpdateResult::AlreadyUpToDate);
            }
        };
        
        // Get the local commit
        let local_commit = head.peel_to_commit()?;
        
        // Check if we're already up to date
        if local_commit.id() == upstream_commit.id() {
            info!("Repository {} is already up to date", repo_path.display());
            return Ok(UpdateResult::AlreadyUpToDate);
        }
        
        // Perform the merge
        let mut merge_options = git2::MergeOptions::new();
        let mut checkout_options = git2::CheckoutOptions::new();
        
        if force {
            checkout_options.force();
        }
        
        // Try to merge
        match repo.merge(&[upstream_commit.into_object()], Some(&mut merge_options), Some(&mut checkout_options)) {
            Ok(_) => {
                // Check if we have a merge conflict
                let index = repo.index()?;
                if index.has_conflicts() {
                    return Err(MctlError::MergeConflict(repo_path));
                }
                
                // Create the merge commit
                let sig = repo.signature()?;
                let message = format!("Merge remote-tracking branch '{}'", upstream_branch.as_str().unwrap_or(""));
                let tree = repo.find_tree(index.write_tree()?)?;
                let parent_commits = [local_commit, upstream_commit];
                
                repo.commit(Some("HEAD"), &sig, &sig, &message, &tree, &parent_commits)?;
                
                info!("Successfully merged changes for {}", repo_path.display());
                Ok(UpdateResult::Updated)
            },
            Err(e) => {
                error!("Failed to merge changes: {}", e);
                Err(MctlError::GitError(format!("Failed to merge changes: {}", e)))
            }
        }
    }
    
    /// Save changes in a repository
    pub fn save(repository: &Repository, base_dir: &Path, message: Option<&str>) -> MctlResult<SaveResult> {
        let repo_path = repository.absolute_path(base_dir);
        
        // Check if the repository exists
        if !repo_path.exists() || !repo_path.join(".git").exists() {
            return Err(MctlError::RepositoryNotFound(repo_path));
        }
        
        // Open the repository
        let repo = Git2Repository::open(&repo_path)?;
        
        // Check if there are any changes
        let status = Self::status(repository, base_dir)?;
        if status.is_clean {
            info!("No changes to save in {}", repo_path.display());
            return Ok(SaveResult::NoChanges);
        }
        
        // Stage all changes
        let mut index = repo.index()?;
        index.add_all(&["."], git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        
        // Create the commit
        let sig = repo.signature()?;
        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        
        let parent_commit = match repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None,
        };
        
        let parents = match parent_commit {
            Some(commit) => vec![&commit],
            None => vec![],
        };
        
        // Use the provided message or generate a default one
        let commit_message = message.unwrap_or_else(|| {
            let timestamp = chrono::Utc::now().to_rfc3339();
            let repo_name = repository.git_url.split('/').last().unwrap_or("repo");
            format!("{} - {}", repo_name, timestamp)
        });
        
        let commit_id = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &commit_message,
            &tree,
            &parents,
        )?;
        
        info!("Created commit {} in {}", commit_id, repo_path.display());
        
        // Push the changes
        let mut remote = repo.find_remote("origin")?;
        
        // Set up callbacks for authentication
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|url, username_from_url, allowed_types| {
            Self::credentials_callback(url, username_from_url, allowed_types)
        });
        
        // Set up push options
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);
        
        // Get the current branch name
        let branch_name = repo.head()?.shorthand().unwrap_or("HEAD").to_string();
        
        // Push to remote
        info!("Pushing changes to remote for {}", repo_path.display());
        match remote.push(&[&format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)], Some(&mut push_options)) {
            Ok(_) => {
                info!("Successfully pushed changes for {}", repo_path.display());
                Ok(SaveResult::Saved)
            },
            Err(e) => {
                error!("Failed to push changes: {}", e);
                Err(MctlError::GitError(format!("Failed to push changes: {}", e)))
            }
        }
    }
    
    /// Credentials callback for git operations
    fn credentials_callback(
        url: &str,
        username_from_url: Option<&str>,
        allowed_types: git2::CredentialType,
    ) -> Result<Cred, git2::Error> {
        debug!("Authentication required for {}", url);
        
        // Try SSH key authentication
        if allowed_types.contains(git2::CredentialType::SSH_KEY) {
            debug!("Trying SSH key authentication");
            
            // Try to use the SSH key from the standard location
            let ssh_key_path = dirs::home_dir()
                .map(|p| p.join(".ssh").join("id_rsa"))
                .unwrap_or_else(|| PathBuf::from("~/.ssh/id_rsa"));
            
            if ssh_key_path.exists() {
                return Cred::ssh_key(
                    username_from_url.unwrap_or("git"),
                    None,
                    &ssh_key_path,
                    None,
                );
            }
        }
        
        // Try username/password authentication
        if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
            debug!("Trying username/password authentication");
            
            // Try to get credentials from environment variables
            if let (Ok(username), Ok(password)) = (env::var("GIT_USERNAME"), env::var("GIT_PASSWORD")) {
                return Cred::userpass_plaintext(&username, &password);
            }
        }
        
        // Default to anonymous authentication
        debug!("Falling back to anonymous authentication");
        Cred::default()
    }
}

/// Status of a repository
#[derive(Debug)]
pub struct RepositoryStatus {
    /// Path to the repository
    pub path: PathBuf,
    
    /// Current branch
    pub branch: String,
    
    /// Status of the branch relative to upstream
    pub branch_status: String,
    
    /// Modified files (path, status code)
    pub modified_files: Vec<(PathBuf, String)>,
    
    /// Untracked files (path, status code)
    pub untracked_files: Vec<(PathBuf, String)>,
    
    /// Whether the repository is clean (no changes)
    pub is_clean: bool,
}

/// Result of an update operation
#[derive(Debug)]
pub enum UpdateResult {
    /// Repository was updated
    Updated,
    
    /// Repository was already up to date
    AlreadyUpToDate,
}

/// Result of a save operation
#[derive(Debug)]
pub enum SaveResult {
    /// Changes were saved
    Saved,
    
    /// No changes to save
    NoChanges,
}