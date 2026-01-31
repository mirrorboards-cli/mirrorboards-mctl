//! Sync command - clone/pull repositories.

use crate::cli::commands::{print_error, print_info, print_success, print_warning};
use crate::core::config::MirrorConfig;
use crate::core::repository::Repository;
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

pub fn execute(
    config_path: &str,
    workspace: Option<String>,
    dry_run: bool,
    force: bool,
    _verbose: bool,
) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    // Filter repositories
    let repos: Vec<&Repository> = if let Some(ws) = &workspace {
        config.filter_by_workspace(ws)
    } else {
        config.repositories.iter().collect()
    };

    if repos.is_empty() {
        if let Some(ws) = &workspace {
            println!("No repositories in workspace '{}'", ws);
        } else {
            println!("No repositories configured");
        }
        return Ok(());
    }

    // Print header
    if let Some(ws) = &workspace {
        println!(
            "{} {} ({} repositories)",
            "Syncing workspace:".bold(),
            ws.cyan(),
            repos.len()
        );
    } else {
        println!(
            "{} ({} repositories)",
            "Syncing all repositories".bold(),
            repos.len()
        );
    }

    if dry_run {
        println!("{}", "(dry run)".yellow());
    }
    println!();

    let git = GitClient::new();

    // Check git is available
    if let Err(e) = git.check_git_available() {
        print_error(&format!("Git is not available: {}", e));
        return Ok(());
    }

    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for repo in repos {
        let local_path = Path::new(&repo.path);
        let version = repo.version_spec();

        // Create progress spinner
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb.set_message(format!("{} ({})", repo.path, version));

        if dry_run {
            if local_path.exists() && git.is_git_repository(local_path) {
                pb.finish_with_message(format!(
                    "{} {} - would pull",
                    "→".blue(),
                    repo.path
                ));
            } else {
                pb.finish_with_message(format!(
                    "{} {} - would clone from {}",
                    "→".blue(),
                    repo.path,
                    repo.git
                ));
            }
            continue;
        }

        // Check if repo exists
        if local_path.exists() && git.is_git_repository(local_path) {
            // Check for local changes
            let status = match git.status_fast(local_path) {
                Ok(s) => s,
                Err(e) => {
                    pb.finish_with_message(format!(
                        "{} {} - failed to get status: {}",
                        "✗".red(),
                        repo.path,
                        e
                    ));
                    error_count += 1;
                    continue;
                }
            };

            if status.has_uncommitted_changes() && !force {
                pb.finish_with_message(format!(
                    "{} {} - skipped (has local changes)",
                    "!".yellow(),
                    repo.path
                ));
                skip_count += 1;
                continue;
            }

            // Sync
            match git.sync(local_path, &version) {
                Ok(_) => {
                    pb.finish_with_message(format!(
                        "{} {} - synced",
                        "✓".green(),
                        repo.path
                    ));
                    success_count += 1;
                }
                Err(e) => {
                    pb.finish_with_message(format!(
                        "{} {} - sync failed: {}",
                        "✗".red(),
                        repo.path,
                        e
                    ));
                    error_count += 1;
                }
            }
        } else {
            // Clone
            match git.clone(&repo.git, local_path, &version) {
                Ok(_) => {
                    pb.finish_with_message(format!(
                        "{} {} - cloned",
                        "✓".green(),
                        repo.path
                    ));
                    success_count += 1;
                }
                Err(e) => {
                    pb.finish_with_message(format!(
                        "{} {} - clone failed: {}",
                        "✗".red(),
                        repo.path,
                        e
                    ));
                    error_count += 1;
                }
            }
        }
    }

    // Summary
    println!();
    if dry_run {
        print_info("Dry run complete - no changes made");
    } else {
        let mut summary_parts = Vec::new();
        if success_count > 0 {
            summary_parts.push(format!("{} synced", success_count));
        }
        if skip_count > 0 {
            summary_parts.push(format!("{} skipped", skip_count));
        }
        if error_count > 0 {
            summary_parts.push(format!("{} failed", error_count));
        }

        if error_count > 0 {
            print_warning(&format!("Sync complete: {}", summary_parts.join(", ")));
        } else {
            print_success(&format!("Sync complete: {}", summary_parts.join(", ")));
        }
    }

    Ok(())
}
