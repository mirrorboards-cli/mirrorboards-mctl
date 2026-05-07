//! Sync command - clone/pull repositories.

use crate::cli::commands::{print_error, print_info, print_success, print_warning};
use crate::core::config::MirrorConfig;
use crate::core::error::ConfigError;
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
    _force: bool,
    create_missing_branches: bool,
    _verbose: bool,
) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!("Configuration file not found: {}", config_path));
        return Ok(());
    }

    let git = GitClient::new();

    // Check git is available
    if let Err(e) = git.check_git_available() {
        print_error(&format!("Git is not available: {}", e));
        return Ok(());
    }

    let config = load_sync_config(config_file, &git, dry_run, create_missing_branches)?;

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

    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for repo in repos {
        let local_path = repo.resolve_local_path(config_file);
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
            if local_path.exists() && git.is_git_repository(&local_path) {
                pb.finish_with_message(format!("{} {} - would pull", "→".blue(), repo.path));
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

        // Check if repo exists - skip if already cloned
        if local_path.exists() && git.is_git_repository(&local_path) {
            pb.finish_with_message(format!(
                "{} {} - skipped (already cloned)",
                "→".blue(),
                repo.path
            ));
            skip_count += 1;
            continue;
        } else {
            // Clone / bootstrap
            match clone_repository(&git, repo, &local_path, &version, create_missing_branches) {
                Ok(_) => {
                    pb.finish_with_message(format!("{} {} - cloned", "✓".green(), repo.path));
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

fn load_sync_config(
    config_file: &Path,
    git: &GitClient,
    dry_run: bool,
    create_missing_branches: bool,
) -> Result<MirrorConfig> {
    match MirrorConfig::load(config_file) {
        Ok(config) => Ok(config),
        Err(ConfigError::IncludeNotFound { .. }) if !dry_run => {
            if bootstrap_top_level_repositories(config_file, git, create_missing_branches)? {
                print_info("Bootstrapped top-level repositories for include resolution");
            }
            Ok(MirrorConfig::load(config_file)?)
        }
        Err(err) => Err(err.into()),
    }
}

fn bootstrap_top_level_repositories(
    config_file: &Path,
    git: &GitClient,
    create_missing_branches: bool,
) -> Result<bool> {
    let raw_config = MirrorConfig::load_raw(config_file)?;
    let mut bootstrapped = false;

    for repo in &raw_config.repositories {
        let local_path = repo.resolve_local_path(config_file);

        if local_path.exists() && git.is_git_repository(&local_path) {
            continue;
        }

        clone_repository(
            git,
            repo,
            &local_path,
            &repo.version_spec(),
            create_missing_branches,
        )?;
        bootstrapped = true;
    }

    Ok(bootstrapped)
}

fn clone_repository(
    git: &GitClient,
    repo: &Repository,
    local_path: &Path,
    version: &crate::core::repository::VersionSpec,
    create_missing_branches: bool,
) -> Result<()> {
    match (repo.path == ".", create_missing_branches, version) {
        (true, true, crate::core::repository::VersionSpec::Branch(branch)) => {
            git.clone_into_existing_dir_or_create_branch(
                &repo.git,
                local_path,
                branch,
                !repo.skip_push,
            )?;
        }
        (true, _, _) => {
            git.clone_into_existing_dir(&repo.git, local_path, version)?;
        }
        (false, true, crate::core::repository::VersionSpec::Branch(branch)) => {
            git.clone_or_create_branch(&repo.git, local_path, branch, !repo.skip_push)?;
        }
        (false, _, _) => {
            git.clone(&repo.git, local_path, version)?;
        }
    }

    Ok(())
}
