//! Reset command implementation
//!
//! This module implements the functionality of the reset command,
//! which performs git reset operations across all repositories defined in a mirror.toml file.

use std::path::PathBuf;
use std::io::{self, Write};
use git2::{Repository as GitRepository, ResetType};
use mirror_sdk::MirrorConfig;
use crate::cli::reset::ResetArgs;
use crate::output::OutputFormatter;
use crate::utils::resolve_relative_path;
use super::{CommandResult, CommandError};
use colored::*;

/// Execute the reset command
pub fn execute(args: ResetArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    // Load the mirror.toml file
    let config_path_str = config_path.clone().unwrap_or_else(|| "mirror.toml".to_string());
    let config_path_buf = PathBuf::from(&config_path_str);
    
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

    // Validate reset mode
    let reset_type = match args.mode.as_str() {
        "soft" => ResetType::Soft,
        "mixed" => ResetType::Mixed,
        "hard" => ResetType::Hard,
        _ => {
            return Err(CommandError::Input(format!(
                "Invalid reset mode '{}'. Valid modes are: soft, mixed, hard", 
                args.mode
            )));
        }
    };

    formatter.info(&format!("Found {} repositories to reset", repositories.len()));

    // Show confirmation prompt unless --force is used
    if !args.force {
        formatter.warning(&format!(
            "This will perform a {} reset on {} repositories{}.",
            args.mode.yellow().bold(),
            repositories.len().to_string().yellow().bold(),
            if let Some(commit) = &args.commit {
                format!(" to commit {}", commit.cyan())
            } else {
                " to HEAD".to_string()
            }
        ));
        
        // List affected repositories
        formatter.info("\nAffected repositories:");
        for repo in &repositories {
            let repo_path = resolve_relative_path(&config_path_buf, &repo.path);
            let repo_name = repo_path.file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| repo.path.clone());
            formatter.info(&format!("  → {}", repo_name.cyan()));
        }
        
        print!("\nContinue? (y/N): ");
        io::stdout().flush().map_err(|e| CommandError::Other(format!("Failed to flush stdout: {}", e)))?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| CommandError::Other(format!("Failed to read input: {}", e)))?;
        
        let input = input.trim().to_lowercase();
        if input != "y" && input != "yes" {
            formatter.info("Reset operation cancelled");
            return Ok(());
        }
    }

    // Process each repository
    let mut success_count = 0;
    let mut error_count = 0;

    for repo in repositories {
        let repo_path_str = &repo.path;
        let repo_path = resolve_relative_path(&config_path_buf, repo_path_str);
        
        // Get the repository name for display
        let repo_name = repo_path.file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| repo_path_str.clone());

        // Check if repository exists
        if !repo_path.exists() {
            formatter.error(&format!("Repository not found at {}", repo_path.display()));
            error_count += 1;
            continue;
        }

        // Open the git repository
        match GitRepository::open(&repo_path) {
            Ok(git_repo) => {
                match perform_reset(&git_repo, &args.commit, reset_type, &repo_name, formatter) {
                    Ok(()) => {
                        success_count += 1;
                    },
                    Err(e) => {
                        formatter.error(&format!("Failed to reset {}: {}", repo_name, e));
                        error_count += 1;
                    }
                }
            },
            Err(e) => {
                formatter.error(&format!("Failed to open repository at {}: {}", repo_path.display(), e));
                error_count += 1;
                continue;
            }
        }
    }

    // Summary
    if success_count > 0 {
        formatter.success(&format!("Successfully reset {} repositories", success_count));
    }
    if error_count > 0 {
        formatter.error(&format!("Failed to reset {} repositories", error_count));
    }

    Ok(())
}

/// Perform the actual git reset operation on a repository
fn perform_reset(
    git_repo: &GitRepository,
    commit: &Option<String>,
    reset_type: ResetType,
    repo_name: &str,
    formatter: &mut dyn OutputFormatter,
) -> CommandResult<()> {
    // Determine the target commit
    let target_oid = if let Some(commit_ref) = commit {
        // Try to resolve the commit reference
        match git_repo.revparse_single(commit_ref) {
            Ok(object) => object.id(),
            Err(e) => {
                return Err(CommandError::Other(format!(
                    "Invalid commit reference '{}': {}", 
                    commit_ref, e
                )));
            }
        }
    } else {
        // Default to HEAD
        match git_repo.head() {
            Ok(head_ref) => head_ref.target().ok_or_else(|| {
                CommandError::Other("HEAD does not point to a valid commit".to_string())
            })?,
            Err(e) => {
                return Err(CommandError::Other(format!("Failed to get HEAD: {}", e)));
            }
        }
    };

    // Get the commit object
    let commit_object = git_repo.find_commit(target_oid).map_err(|e| {
        CommandError::Other(format!("Failed to find commit {}: {}", target_oid, e))
    })?;

    // Perform the reset
    git_repo.reset(commit_object.as_object(), reset_type, None).map_err(|e| {
        CommandError::Other(format!("Git reset failed: {}", e))
    })?;

    // Show success message with commit info
    let commit_summary = if commit_object.summary().unwrap_or("").len() > 50 {
        format!("{}...", &commit_object.summary().unwrap_or("")[..47])
    } else {
        commit_object.summary().unwrap_or("").to_string()
    };

    formatter.success(&format!(
        "→ {} reset to {} ({})",
        repo_name.bold().green(),
        target_oid.to_string()[..8].cyan(),
        commit_summary.white()
    ));

    Ok(())
}