//! Sync command implementation
//!
//! This module implements the functionality of the sync command,
//! which clones all repositories defined in a mirror.toml file.

use std::fs;
use std::path::{Path, PathBuf};
use git2::{Repository as GitRepository, Cred, RemoteCallbacks, FetchOptions, build::RepoBuilder, CredentialType};
use mirror_sdk::{MirrorConfig, Repository};
use crate::cli::sync::SyncArgs;
use crate::output::OutputFormatter;
use crate::utils::{resolve_ssh_key_path, get_ssh_key, set_ssh_key};
use super::{CommandResult, CommandError};

/// Represents the result of syncing a single repository
#[derive(Debug)]
struct RepositoryResult {
    repository: Repository,
    success: bool,
    error: Option<String>,
}

/// Execute the sync command
pub fn execute(args: SyncArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    // Load the mirror.toml file
    let config = if let Some(path) = config_path {
        formatter.info(&format!("Loading mirror.toml from {}", path));
        MirrorConfig::load_from(path)
    } else {
        formatter.info("Loading mirror.toml from default location");
        MirrorConfig::load()
    }?;

    // Get repositories, optionally filtered by tag
    let repositories = if let Some(tag) = &args.tag {
        formatter.info(&format!("Filtering repositories by tag: {}", tag));
        config.get_repositories_by_tag(tag)
    } else {
        formatter.info("Processing all repositories");
        config.get_repositories().iter().collect()
    };

    if repositories.is_empty() {
        formatter.warning("No repositories found");
        return Ok(());
    }

    formatter.info(&format!("Found {} repositories to sync", repositories.len()));

    // Store authentication settings
    let use_auth = !args.no_auth;

    // Determine SSH key path with fallback hierarchy
    let ssh_key_path = if let Some(cli_ssh_key) = &args.ssh_key {
        // User provided via CLI - save to config for future use
        formatter.info(&format!("Saving SSH key path '{}' to config for future use", cli_ssh_key));
        if let Err(e) = set_ssh_key(cli_ssh_key) {
            formatter.warning(&format!("Failed to save SSH key to config: {}", e));
        }
        match resolve_ssh_key_path(cli_ssh_key) {
            Ok(path) => Some(path),
            Err(e) => {
                formatter.error(&format!("Failed to resolve SSH key path: {}", e));
                None
            }
        }
    } else {
        // Check config file for saved SSH key
        match get_ssh_key() {
            Ok(Some(saved_ssh_key)) => {
                formatter.info(&format!("Using saved SSH key from config: {}", &saved_ssh_key));
                match resolve_ssh_key_path(&saved_ssh_key) {
                    Ok(path) => Some(path),
                    Err(e) => {
                        formatter.warning(&format!("Failed to resolve saved SSH key path: {}", e));
                        None
                    }
                }
            }
            Ok(None) => {
                if use_auth {
                    formatter.info("No SSH key specified, will use SSH agent authentication");
                }
                None
            }
            Err(e) => {
                formatter.warning(&format!("Failed to get SSH key from config: {}", e));
                None
            }
        }
    };

    // Collect results for all repositories
    let mut results = Vec::new();

    // Process each repository
    for repo in repositories {
        let result = sync_single_repository(
            repo,
            &args,
            formatter,
            use_auth,
            &ssh_key_path,
        );
        results.push(result);
    }

    // Display summary
    display_sync_summary(&results, formatter);

    // Return success if at least one repository was synced successfully
    let successful_count = results.iter().filter(|r| r.success).count();
    let failed_count = results.len() - successful_count;

    if failed_count > 0 && successful_count == 0 {
        // All repositories failed
        Err(CommandError::Other(format!("All {} repositories failed to sync", failed_count)))
    } else if failed_count > 0 {
        // Some repositories failed, but at least one succeeded
        formatter.warning(&format!("Sync completed with {} errors (see above for details)", failed_count));
        Ok(())
    } else {
        // All repositories succeeded
        Ok(())
    }
}

/// Sync a single repository, handling all errors gracefully
fn sync_single_repository(
    repo: &Repository,
    args: &SyncArgs,
    formatter: &mut dyn OutputFormatter,
    use_auth: bool,
    ssh_key_path: &Option<PathBuf>,
) -> RepositoryResult {
    let repo_path = Path::new(&repo.path);
    let repo_path_str = repo_path.display();

    // Check if repository already exists
    if repo_path.exists() {
        if args.skip_existing {
            formatter.info(&format!("Skipping existing repository at {}", repo_path_str));
            return RepositoryResult {
                repository: repo.clone(),
                success: true,
                error: None,
            };
        } else {
            // Check if it's a valid git repository
            match GitRepository::open(repo_path) {
                Ok(_) => {
                    formatter.info(&format!("Repository already exists at {}", repo_path_str));
                    return RepositoryResult {
                        repository: repo.clone(),
                        success: true,
                        error: None,
                    };
                }
                Err(_) => {
                    let error_msg = format!("Directory exists at {} but is not a valid git repository", repo_path_str);
                    formatter.error(&error_msg);
                    return RepositoryResult {
                        repository: repo.clone(),
                        success: false,
                        error: Some(error_msg),
                    };
                }
            }
        }
    }

    // Create parent directories if they don't exist
    if let Some(parent) = repo_path.parent() {
        if !parent.exists() {
            formatter.info(&format!("Creating directory: {}", parent.display()));
            if let Err(e) = fs::create_dir_all(parent) {
                let error_msg = format!("Failed to create directory {}: {}", parent.display(), e);
                formatter.error(&error_msg);
                return RepositoryResult {
                    repository: repo.clone(),
                    success: false,
                    error: Some(error_msg),
                };
            }
        }
    }

    // Clone the repository
    formatter.info(&format!("Cloning {} to {}", repo.origin, repo_path_str));
    
    let mut builder = RepoBuilder::new();
    
    // Set up SSH authentication for this repository if needed
    if use_auth {
        // Log authentication information outside the closure
        formatter.info("Setting up SSH authentication");
        if let Some(ref key_path) = ssh_key_path {
            formatter.info(&format!("Using SSH key from path: {}", key_path.display()));
        } else {
            formatter.info("Using SSH key from agent");
        }
        
        let mut callbacks = RemoteCallbacks::new();
        let ssh_key = ssh_key_path.clone();
        callbacks.credentials(move |_url, username_from_url, allowed_types| {
            // Check if SSH key authentication is allowed
            if allowed_types.contains(CredentialType::SSH_KEY) ||
               allowed_types.contains(CredentialType::SSH_MEMORY) {
                // Use the provided SSH key if specified
                if let Some(ref key_path) = ssh_key {
                    Cred::ssh_key(
                        username_from_url.unwrap_or("git"),
                        None,
                        key_path,
                        None,
                    )
                } else {
                    // Try to use the default SSH key from the agent
                    Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
                }
            } else if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                // Fall back to default credentials if SSH is not allowed
                Cred::default()
            } else {
                // Last resort: try SSH key authentication anyway
                Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            }
        });
        
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);
    }
    
    // Set branch if specified
    if let Some(branch) = &repo.branch {
        builder.branch(branch);
    }
    
    match builder.clone(&repo.origin, repo_path) {
        Ok(_) => {
            formatter.success(&format!("Successfully cloned {} to {}", repo.origin, repo_path_str));
            RepositoryResult {
                repository: repo.clone(),
                success: true,
                error: None,
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to clone {}: {}", repo.origin, e);
            formatter.error(&error_msg);
            RepositoryResult {
                repository: repo.clone(),
                success: false,
                error: Some(error_msg),
            }
        }
    }
}

/// Display a summary of sync results
fn display_sync_summary(results: &[RepositoryResult], formatter: &mut dyn OutputFormatter) {
    let total_count = results.len();
    let successful_count = results.iter().filter(|r| r.success).count();
    let failed_count = total_count - successful_count;

    formatter.info("");
    formatter.info("=== Sync Summary ===");
    formatter.info(&format!("Total repositories: {}", total_count));
    
    if successful_count > 0 {
        formatter.success(&format!("Successfully synced: {}", successful_count));
    }
    
    if failed_count > 0 {
        formatter.error(&format!("Failed to sync: {}", failed_count));
        formatter.info("");
        formatter.info("Failed repositories:");
        for result in results.iter().filter(|r| !r.success) {
            if let Some(error) = &result.error {
                formatter.error(&format!("  • {} ({}): {}",
                    result.repository.origin,
                    result.repository.path,
                    error));
            }
        }
    }

    if failed_count == 0 {
        formatter.success("All repositories synced successfully!");
    } else if successful_count > 0 {
        formatter.warning("Sync completed with some errors. Check failed repositories above.");
    }
}