//! Sync command implementation
//!
//! This module implements the functionality of the sync command,
//! which clones all repositories defined in a mirror.toml file.

use std::fs;
use std::path::{Path, PathBuf};
use git2::{Repository as GitRepository, Cred, RemoteCallbacks, FetchOptions, build::RepoBuilder, CredentialType};
use mirror_sdk::MirrorConfig;
use crate::cli::sync::SyncArgs;
use crate::output::OutputFormatter;
use crate::utils::{resolve_ssh_key_path, get_ssh_key, set_ssh_key};
use super::{CommandResult, CommandError};

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
        Some(resolve_ssh_key_path(cli_ssh_key)?)
    } else {
        // Check config file for saved SSH key
        match get_ssh_key()? {
            Some(saved_ssh_key) => {
                formatter.info(&format!("Using saved SSH key from config: {}", &saved_ssh_key));
                Some(resolve_ssh_key_path(&saved_ssh_key)?)
            }
            None => {
                if use_auth {
                    formatter.info("No SSH key specified, will use SSH agent authentication");
                }
                None
            }
        }
    };

    // Process each repository
    for repo in repositories {
        let repo_path = Path::new(&repo.path);
        let repo_path_str = repo_path.display();

        // Check if repository already exists
        if repo_path.exists() {
            if args.skip_existing {
                formatter.info(&format!("Skipping existing repository at {}", repo_path_str));
                continue;
            } else {
                // Check if it's a valid git repository
                match GitRepository::open(repo_path) {
                    Ok(_) => {
                        formatter.info(&format!("Repository already exists at {}", repo_path_str));
                        continue;
                    }
                    Err(_) => {
                        formatter.warning(&format!("Directory exists at {} but is not a valid git repository", repo_path_str));
                        return Err(CommandError::Other(format!("Directory exists at {} but is not a valid git repository", repo_path_str)));
                    }
                }
            }
        }

        // Create parent directories if they don't exist
        if let Some(parent) = repo_path.parent() {
            if !parent.exists() {
                formatter.info(&format!("Creating directory: {}", parent.display()));
                fs::create_dir_all(parent).map_err(|e| {
                    CommandError::File(format!("Failed to create directory {}: {}", parent.display(), e))
                })?;
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
            }
            Err(e) => {
                formatter.error(&format!("Failed to clone {}: {}", repo.origin, e));
                return Err(CommandError::Other(format!("Failed to clone {}: {}", repo.origin, e)));
            }
        }
    }

    formatter.success("Sync completed successfully");
    Ok(())
}