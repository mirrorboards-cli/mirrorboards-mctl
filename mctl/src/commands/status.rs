//! Status command implementation
//!
//! This module implements the functionality of the status command,
//! which shows the git status of all repositories defined in a mirror.toml file.

use std::path::{Path, PathBuf};
use git2::{Repository as GitRepository, Status as GitStatus, StatusOptions};
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
                // Configure status options to exclude ignored files
                let mut status_opts = StatusOptions::new();
                status_opts.include_ignored(false);
                status_opts.include_untracked(true);
                status_opts.exclude_submodules(false);
                
                // Get the repository status
                let statuses = git_repo.statuses(Some(&mut status_opts)).map_err(|e| {
                    CommandError::Other(format!("Failed to get status for {}: {}", repo_path.display(), e))
                })?;

                if statuses.is_empty() {
                    // Skip displaying clean repositories unless show_clean flag is set
                    if args.show_clean {
                        // Get the repository name from the path
                        let repo_name = repo_path.file_name()
                            .map(|name| name.to_string_lossy().to_string())
                            .unwrap_or_else(|| repo_path_str.clone());
                        
                        // Display clean repository with a modern, clean format
                        formatter.success(&format!("{} {} {}",
                            "→".bold(),
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
                    // Display repository name with a more modern, clean format
                    formatter.warning(&format!("{} {}", "→".bold(), repo_name.bold().yellow()));
                    
                    
                    // Collect changed and untracked files separately
                    let mut changed_files = Vec::new();
                    let mut untracked_files = Vec::new();
                    
                    for entry in statuses.iter() {
                        let status = entry.status();
                        let path = entry.path().unwrap_or("unknown");
                        
                        // Make the path relative to the mirror.toml file location
                        let full_path = repo_path.join(path);
                        let relative_path = make_path_relative_to_config(&config_path_buf, &full_path);
                        
                        // Format the status with color
                        let status_str = format_git_status(status);
                        
                        // Determine file type and add to appropriate collection
                        if status.is_wt_new() {
                            untracked_files.push((status, relative_path));
                        } else {
                            changed_files.push((status, relative_path));
                        }
                    }
                    
                    // Display changed files
                    if !changed_files.is_empty() {
                        formatter.info("  Changed files:");
                        for (status, path) in changed_files {
                            let colored_path = color_path_by_status(status, &path);
                            formatter.info(&format!("    {}", colored_path));
                        }
                    }
                    
                    // Display untracked files
                    if !untracked_files.is_empty() {
                        formatter.info("  Untracked files:");
                        for (status, path) in untracked_files {
                            let colored_path = color_path_by_status(status, &path);
                            formatter.info(&format!("    {}", colored_path));
                        }
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
        formatter.success("All repositories are clean");
    }

    Ok(())
}

/// Format git status as a colored string (for internal use)
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

/// Color the file path based on its git status
fn color_path_by_status(status: GitStatus, path: &str) -> String {
    if status.is_index_new() {
        path.green().to_string()
    } else if status.is_index_modified() {
        path.blue().to_string()
    } else if status.is_index_deleted() {
        path.red().to_string()
    } else if status.is_index_renamed() {
        path.cyan().to_string()
    } else if status.is_index_typechange() {
        path.magenta().to_string()
    } else if status.is_wt_new() {
        path.bright_green().to_string()
    } else if status.is_wt_modified() {
        path.bright_blue().to_string()
    } else if status.is_wt_deleted() {
        path.bright_red().to_string()
    } else if status.is_wt_renamed() {
        path.bright_cyan().to_string()
    } else if status.is_wt_typechange() {
        path.bright_magenta().to_string()
    } else if status.is_conflicted() {
        path.bright_yellow().to_string()
    } else {
        path.white().to_string()
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