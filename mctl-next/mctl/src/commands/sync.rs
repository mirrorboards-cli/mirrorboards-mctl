use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use mirror_sdk::{ConfigLoader, ConfigValidator, GitManager, SyncResult};

pub fn run(config_path: &str, workspace: Option<&str>) -> Result<()> {
    // Load and validate config
    let config = ConfigLoader::new()
        .load(config_path)
        .context("Failed to load configuration")?;

    ConfigValidator::validate(&config).context("Configuration validation failed")?;

    // Get base path (directory containing config file)
    let base_path = std::path::Path::new(config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    let git_manager = GitManager::new(base_path);

    // Get repositories to sync
    let repositories = config.get_repositories(workspace);

    if repositories.is_empty() {
        if let Some(ws) = workspace {
            println!(
                "{}",
                format!("No repositories found in workspace '{}'", ws).yellow()
            );
        } else {
            println!("{}", "No repositories configured".yellow());
        }
        return Ok(());
    }

    println!(
        "{}",
        format!("Syncing {} repositories...", repositories.len()).cyan()
    );

    // Create progress bar
    let pb = ProgressBar::new(repositories.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut cloned = 0;
    let mut updated = 0;
    let mut failed = 0;

    for repo in repositories {
        pb.set_message(repo.path.clone());

        match git_manager.sync(repo) {
            Ok(SyncResult::Cloned) => {
                cloned += 1;
            }
            Ok(SyncResult::Updated) => {
                updated += 1;
            }
            Err(e) => {
                failed += 1;
                pb.println(format!("{} {} - {}", "✗".red(), repo.path.red(), e));
            }
        }

        pb.inc(1);
    }

    pb.finish_and_clear();

    // Print summary
    println!();
    println!("{}", "Summary:".bold());

    if cloned > 0 {
        println!("  {} {}", "Cloned:".green(), cloned);
    }
    if updated > 0 {
        println!("  {} {}", "Updated:".blue(), updated);
    }
    if failed > 0 {
        println!("  {} {}", "Failed:".red(), failed);
    }

    if failed > 0 {
        anyhow::bail!("{} repositories failed to sync", failed);
    }

    Ok(())
}
