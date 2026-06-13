//! Save command - commit and push changes in repositories.

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
    message: &str,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!("Configuration file not found: {}", config_path));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    // Filter repositories (exclude skip-push and readonly)
    let repos: Vec<&Repository> = if let Some(ws) = &workspace {
        config
            .filter_by_workspace(ws)
            .into_iter()
            .filter(|r| !r.skip_push && !r.readonly)
            .collect()
    } else {
        config
            .repositories
            .iter()
            .filter(|r| !r.skip_push && !r.readonly)
            .collect()
    };

    if repos.is_empty() {
        if let Some(ws) = &workspace {
            println!("No pushable repositories in workspace '{}'", ws);
        } else {
            println!("No pushable repositories configured");
        }
        return Ok(());
    }

    // Print header
    if let Some(ws) = &workspace {
        println!(
            "{} {} ({} repositories)",
            "Saving workspace:".bold(),
            ws.cyan(),
            repos.len()
        );
    } else {
        println!(
            "{} ({} repositories)",
            "Saving all repositories".bold(),
            repos.len()
        );
    }

    if dry_run {
        println!("{}", "(dry run)".yellow());
    }
    println!();

    let git = GitClient::new();

    let mut saved_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for repo in repos {
        let local_path = repo.resolve_local_path(config_file);

        // Skip if not cloned
        if !local_path.exists() || !git.is_git_repository(&local_path) {
            if verbose {
                print_info(&format!("{}: Not cloned, skipping", repo.path));
            }
            skip_count += 1;
            continue;
        }

        // Check status (use status_fast to handle repos without commits)
        let status = match git.status_fast(&local_path) {
            Ok(s) => s,
            Err(e) => {
                print_error(&format!("{}: Failed to get status: {}", repo.path, e));
                error_count += 1;
                continue;
            }
        };

        // Skip if fully synced (no changes and no unpushed commits)
        if status.is_fully_synced() {
            if verbose {
                print_info(&format!("{}: No changes, skipping", repo.path));
            }
            skip_count += 1;
            continue;
        }

        // Check if we only need to push (no uncommitted changes but have unpushed commits)
        let only_push = status.is_clean() && status.has_unpushed_commits();

        // Create spinner
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.blue} {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        if only_push {
            pb.set_message(format!(
                "{}: Pushing {} commit(s)...",
                repo.path, status.branch.ahead
            ));
        } else {
            pb.set_message(format!("{}: Saving...", repo.path));
        }

        if dry_run {
            pb.finish_and_clear();
            if only_push {
                println!(
                    "{} {} - would push {} commit(s)",
                    "→".blue(),
                    repo.path,
                    status.branch.ahead
                );
            } else {
                println!(
                    "{} {} - would save {} changes",
                    "→".blue(),
                    repo.path,
                    status.files.len()
                );
            }
            continue;
        }

        // If we have uncommitted changes, stage and commit first
        if !only_push {
            // Stage all changes
            if let Err(e) = git.add_all(&local_path) {
                pb.finish_and_clear();
                println!("{} {} - failed to stage: {}", "✗".red(), repo.path, e);
                error_count += 1;
                continue;
            }

            // Commit
            if let Err(e) = git.commit(&local_path, message) {
                pb.finish_and_clear();
                println!("{} {} - failed to commit: {}", "✗".red(), repo.path, e);
                error_count += 1;
                continue;
            }
        }

        // Push. If this branch has no upstream yet, publish it to origin using
        // the current branch name and set upstream so later saves can use a
        // plain push.
        pb.finish_and_clear();
        let push_result = if status.branch.upstream.is_none() && !status.branch.is_detached() {
            git.push_set_upstream(&local_path, "origin", &status.branch.name)
        } else {
            git.push(&local_path)
        };

        match push_result {
            Ok(_) => {
                if only_push {
                    println!(
                        "{} {} - pushed {} commit(s)",
                        "✓".green(),
                        repo.path,
                        status.branch.ahead
                    );
                } else {
                    println!("{} {} - saved and pushed", "✓".green(), repo.path);
                }
                saved_count += 1;
            }
            Err(e) => {
                if only_push {
                    println!("{} {} - push failed: {}", "✗".red(), repo.path, e);
                    error_count += 1;
                } else {
                    println!(
                        "{} {} - committed but push failed: {}",
                        "!".yellow(),
                        repo.path,
                        e
                    );
                    // Still count as partial success
                    saved_count += 1;
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
        if saved_count > 0 {
            summary_parts.push(format!("{} saved", saved_count));
        }
        if skip_count > 0 {
            summary_parts.push(format!("{} skipped", skip_count));
        }
        if error_count > 0 {
            summary_parts.push(format!("{} failed", error_count));
        }

        if error_count > 0 {
            print_warning(&format!("Save complete: {}", summary_parts.join(", ")));
        } else {
            print_success(&format!("Save complete: {}", summary_parts.join(", ")));
        }
    }

    Ok(())
}
