//! Save command implementation
//!
//! This module implements the functionality of the save command,
//! which commits and pushes all changes in repositories defined in a mirror.toml file.

use std::path::{Path, PathBuf};
use git2::{Repository as GitRepository, Cred, RemoteCallbacks, PushOptions, CredentialType, Signature};
use mirror_sdk::MirrorConfig;
use crate::cli::save::SaveArgs;
use crate::output::OutputFormatter;
use crate::utils::{resolve_relative_path, resolve_ssh_key_path, get_ssh_key, set_ssh_key};
use super::{CommandResult, CommandError};
use chrono::Local;

/// Execute the save command
pub fn execute(args: SaveArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    // Load the mirror.toml file
    let config_path_str = config_path.clone().unwrap_or_else(|| "mirror.toml".to_string());
    let config_path_buf = PathBuf::from(&config_path_str);
    
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

    formatter.info(&format!("Found {} repositories to save", repositories.len()));

    // Store authentication settings
    let use_auth = !args.no_auth;
    let use_push = !args.no_push;

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

    // Generate timestamp for commit messages
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Process each repository
    for repo in repositories {
        let repo_path_str = &repo.path;
        let repo_path = resolve_relative_path(&config_path_buf, repo_path_str);
        
        // Check if repository exists
        if !repo_path.exists() {
            formatter.warning(&format!("Repository not found at {}", repo_path.display()));
            continue;
        }

        // Open the git repository
        let git_repo = match GitRepository::open(&repo_path) {
            Ok(repo) => repo,
            Err(e) => {
                formatter.error(&format!("Failed to open repository at {}: {}", repo_path.display(), e));
                continue;
            }
        };

        // Get repository name for commit message
        let repo_name = repo_path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| repo.id.clone().unwrap_or_else(|| "unknown".to_string()));

        formatter.info(&format!("Processing repository: {}", repo_name));

        // Stage all changes (git add .)
        let mut index = match git_repo.index() {
            Ok(index) => index,
            Err(e) => {
                formatter.error(&format!("Failed to get index for {}: {}", repo_name, e));
                continue;
            }
        };

        // Add all files in working directory to index
        if let Err(e) = index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None) {
            formatter.error(&format!("Failed to stage changes in {}: {}", repo_name, e));
            continue;
        }

        // Write the index
        if let Err(e) = index.write() {
            formatter.error(&format!("Failed to write index for {}: {}", repo_name, e));
            continue;
        }

        // Check if there are any changes to commit
        let tree_id = match index.write_tree() {
            Ok(id) => id,
            Err(e) => {
                formatter.error(&format!("Failed to write tree for {}: {}", repo_name, e));
                continue;
            }
        };

        // Get the tree object
        let tree = match git_repo.find_tree(tree_id) {
            Ok(tree) => tree,
            Err(e) => {
                formatter.error(&format!("Failed to find tree for {}: {}", repo_name, e));
                continue;
            }
        };

        // Check if this is the same as HEAD (no changes to commit)
        let has_head = git_repo.head().is_ok();
        if has_head {
            if let Ok(head_commit) = git_repo.head().and_then(|r| r.peel_to_commit()) {
                if head_commit.tree_id() == tree_id {
                    formatter.info(&format!("No changes to commit in {}", repo_name));
                    continue;
                }
            }
        }

        // Create commit message
        let commit_message = if let Some(ref custom_message) = args.message {
            custom_message.clone()
        } else {
            // Extract org from origin URL for default message format
            let org = extract_org_from_origin(&repo.origin).unwrap_or_else(|| "unknown".to_string());
            format!("{}/{} - {}", org, repo_name, timestamp)
        };

        // Create signature
        let signature = match create_git_signature(&git_repo) {
            Ok(sig) => sig,
            Err(e) => {
                formatter.error(&format!("Failed to create signature for {}: {}", repo_name, e));
                continue;
            }
        };

        // Create the commit with proper parent handling
        let commit_id = if has_head {
            if let Ok(head_commit) = git_repo.head().and_then(|r| r.peel_to_commit()) {
                git_repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &commit_message,
                    &tree,
                    &[&head_commit],
                )
            } else {
                git_repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &commit_message,
                    &tree,
                    &[],
                )
            }
        } else {
            git_repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &commit_message,
                &tree,
                &[],
            )
        };

        let commit_id = match commit_id {
            Ok(id) => id,
            Err(e) => {
                formatter.error(&format!("Failed to create commit in {}: {}", repo_name, e));
                continue;
            }
        };

        formatter.success(&format!("Committed changes in {} ({})", repo_name, commit_id));

        // Push to remote if not skipped
        if use_push {
            if let Err(e) = push_to_remote(&git_repo, &ssh_key_path, formatter, &repo_name, use_auth) {
                formatter.error(&format!("Failed to push {}: {}", repo_name, e));
                continue;
            }
            formatter.success(&format!("Pushed changes in {}", repo_name));
        }
    }

    if use_push {
        formatter.success("Save completed successfully (committed and pushed)");
    } else {
        formatter.success("Save completed successfully (committed only)");
    }
    Ok(())
}

/// Extract organization name from git origin URL
fn extract_org_from_origin(origin: &str) -> Option<String> {
    // Handle GitHub SSH URLs like git@github.com:org/repo.git
    if origin.starts_with("git@github.com:") {
        if let Some(path) = origin.strip_prefix("git@github.com:") {
            if let Some(org) = path.split('/').next() {
                return Some(org.to_string());
            }
        }
    }
    
    // Handle HTTPS URLs like https://github.com/org/repo.git
    if origin.starts_with("https://github.com/") {
        if let Some(path) = origin.strip_prefix("https://github.com/") {
            if let Some(org) = path.split('/').next() {
                return Some(org.to_string());
            }
        }
    }
    
    // For other URLs, try to extract from path
    if let Some(last_slash) = origin.rfind('/') {
        if let Some(second_last_slash) = origin[..last_slash].rfind('/') {
            let org = &origin[second_last_slash + 1..last_slash];
            if !org.is_empty() {
                return Some(org.to_string());
            }
        }
    }
    
    None
}

/// Create a git signature for commits
fn create_git_signature(repo: &GitRepository) -> Result<Signature, git2::Error> {
    // Try to get signature from git config
    let config = repo.config()?;
    
    let name = config.get_string("user.name")
        .unwrap_or_else(|_| "mctl user".to_string());
    let email = config.get_string("user.email")
        .unwrap_or_else(|_| "mctl@example.com".to_string());
    
    Signature::now(&name, &email)
}

/// Push changes to remote repository
fn push_to_remote(
    git_repo: &GitRepository,
    ssh_key_path: &Option<PathBuf>,
    formatter: &mut dyn OutputFormatter,
    repo_name: &str,
    use_auth: bool,
) -> Result<(), git2::Error> {
    // Get the current branch name
    let head = git_repo.head()?;
    let branch_name = if let Some(name) = head.shorthand() {
        name
    } else {
        return Err(git2::Error::from_str("Could not determine current branch"));
    };

    // Get remote
    let mut remote = git_repo.find_remote("origin")?;

    // Set up authentication if needed
    let mut push_options = PushOptions::new();
    if use_auth {
        formatter.info(&format!("Setting up SSH authentication for {}", repo_name));
        if let Some(ref key_path) = ssh_key_path {
            formatter.info(&format!("Using SSH key from path: {}", key_path.display()));
        } else {
            formatter.info(&format!("Using SSH key from agent for {}", repo_name));
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
                        key_path.as_path(),
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
        
        push_options.remote_callbacks(callbacks);
    }

    // Push the current branch
    let refspec = format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name);
    remote.push(&[&refspec], Some(&mut push_options))?;

    Ok(())
}