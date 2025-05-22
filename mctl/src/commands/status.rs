//! Status command implementation
//!
//! This module implements the functionality of the status command,
//! which shows the git status of all repositories defined in a mirror.toml file.

use std::path::{Path, PathBuf};
use git2::{Repository as GitRepository, Status as GitStatus};
use mirror_sdk::MirrorConfig;
use crate::cli::status::StatusArgs;
use crate::output::OutputFormatter;
use crate::utils::resolve_relative_path;
use super::{CommandResult, CommandError};
use colored::*;

/// Execute the status command
pub fn execute(args: StatusArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
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

    formatter.info(&format!("Found {} repositories to check", repositories.len()));

    // Process each repository
    let mut has_changes = false;

    for repo in repositories {
        let repo_path_str = &repo.path;
        let repo_path = resolve_relative_path(&config_path_buf, repo_path_str);
        
        // Check if repository exists
        if !repo_path.exists() {
            formatter.warning(&format!("Repository not found at {}", repo_path.display()));
            continue;
        }

        // Open the git repository
        match GitRepository::open(&repo_path) {
            Ok(git_repo) => {
                // Get the repository status
                let statuses = git_repo.statuses(None).map_err(|e| {
                    CommandError::Other(format!("Failed to get status for {}: {}", repo_path.display(), e))
                })?;

                if statuses.is_empty() {
                    // Skip displaying clean repositories unless show_clean flag is set
                    if args.show_clean {
                        // Get the repository name from the path
                        let repo_name = repo_path.file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_else(|| repo_path_str.clone());
                        
                        // Display clean repository with a different format
                        formatter.success(&format!("📁 {} {} {}",
                            "Repository:".bold(),
                            repo_name.bold().green(),
                            "(clean)".green()));
                        
                        // Add a separator line between repositories for better readability
                        formatter.info("");
                    }
                    continue;
                } else {
                    has_changes = true;
                    
                    // Get the repository name from the path
                    let repo_name = repo_path.file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| repo_path_str.clone());
                    
                    // Display repository name with a more visually appealing format
                    formatter.warning(&format!("📁 {} {}", "Repository:".bold(), repo_name.bold().yellow()));
                    
                    // Process each status entry
                    for entry in statuses.iter() {
                        let status = entry.status();
                        let path = entry.path().unwrap_or("unknown");
                        
                        // Make the path relative to the mirror.toml file location
                        let full_path = repo_path.join(path);
                        let relative_path = make_path_relative_to_config(&config_path_buf, &full_path);
                        
                        // Format the status with color
                        let status_str = format_git_status(status);
                        formatter.info(&format!("    {} {}", status_str, relative_path));
                    }
                    
                    // Add a separator line between repositories for better readability
                    formatter.info("");
                }
            },
            Err(e) => {
                formatter.error(&format!("Failed to open repository at {}: {}", repo_path.display(), e));
                continue;
            }
        }
    }

    if !has_changes {
        formatter.success("✅ All repositories are clean");
    } else {
        // Add a legend for the status codes
        formatter.info("\n📋 Status Legend:");
        formatter.info(&format!("  {} = Added (staged)", "A".green().bold()));
        formatter.info(&format!("  {} = Modified (staged)", "M".blue().bold()));
        formatter.info(&format!("  {} = Deleted (staged)", "D".red().bold()));
        formatter.info(&format!("  {} = Renamed (staged)", "R".cyan().bold()));
        formatter.info(&format!("  {} = Type changed (staged)", "T".magenta().bold()));
        formatter.info(&format!("  {} = New (unstaged)", "??".bright_green().bold()));
        formatter.info(&format!("  {} = Modified (unstaged)", "M".bright_blue().bold()));
        formatter.info(&format!("  {} = Deleted (unstaged)", "D".bright_red().bold()));
        formatter.info(&format!("  {} = Renamed (unstaged)", "R".bright_cyan().bold()));
        formatter.info(&format!("  {} = Type changed (unstaged)", "T".bright_magenta().bold()));
        formatter.info(&format!("  {} = Conflicted", "!!".bright_yellow().bold()));
    }

    Ok(())
}

/// Format git status as a colored string
fn format_git_status(status: GitStatus) -> String {
    if status.is_index_new() {
        "A".green().bold().to_string()
    } else if status.is_index_modified() {
        "M".blue().bold().to_string()
    } else if status.is_index_deleted() {
        "D".red().bold().to_string()
    } else if status.is_index_renamed() {
        "R".cyan().bold().to_string()
    } else if status.is_index_typechange() {
        "T".magenta().bold().to_string()
    } else if status.is_wt_new() {
        "??".bright_green().bold().to_string()
    } else if status.is_wt_modified() {
        "M".bright_blue().bold().to_string()
    } else if status.is_wt_deleted() {
        "D".bright_red().bold().to_string()
    } else if status.is_wt_renamed() {
        "R".bright_cyan().bold().to_string()
    } else if status.is_wt_typechange() {
        "T".bright_magenta().bold().to_string()
    } else if status.is_conflicted() {
        "!!".bright_yellow().bold().to_string()
    } else {
        " ".to_string()
    }
}

/// Make a path relative to the mirror.toml file location
fn make_path_relative_to_config(config_path: &Path, file_path: &Path) -> String {
    if let Some(config_dir) = config_path.parent() {
        if let Ok(relative) = file_path.strip_prefix(config_dir) {
            return relative.to_string_lossy().to_string();
        }
    }
    
    // Fallback to the full path if we can't make it relative
    file_path.to_string_lossy().to_string()
}