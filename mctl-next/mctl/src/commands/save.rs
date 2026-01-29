use anyhow::{Context, Result};
use chrono::Utc;
use colored::Colorize;

use mirror_sdk::{ConfigLoader, ConfigValidator, GitManager, WorkspaceManager};

pub fn run(config_path: &str, workspace: &str, message: Option<&str>) -> Result<()> {
    // Load and validate config
    let config = ConfigLoader::new()
        .load(config_path)
        .context("Failed to load configuration")?;

    ConfigValidator::validate(&config).context("Configuration validation failed")?;

    // Check workspace exists
    let ws_manager = WorkspaceManager::new(&config);
    if !ws_manager.workspace_exists(workspace) {
        anyhow::bail!("Workspace '{}' not found", workspace);
    }

    // Get base path
    let base_path = std::path::Path::new(config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    let git_manager = GitManager::new(base_path);

    // Get repositories for workspace
    let repositories = ws_manager.repositories_for(workspace);

    if repositories.is_empty() {
        println!(
            "{}",
            format!("No repositories in workspace '{}'", workspace).yellow()
        );
        return Ok(());
    }

    // Generate commit message
    let commit_message = message.map(String::from).unwrap_or_else(|| {
        format!("save {} UTC", Utc::now().format("%Y-%m-%d %H:%M:%S"))
    });

    println!(
        "{}",
        format!("Saving {} repositories in workspace '{}'...", repositories.len(), workspace).cyan()
    );
    println!("Commit message: {}", commit_message.dimmed());
    println!();

    let mut saved = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for repo in repositories {
        print!("  {} ", repo.path);

        // Check if repo exists
        if !git_manager.exists(repo) {
            println!("{}", "[not cloned, skipping]".dimmed());
            skipped += 1;
            continue;
        }

        // Check if there are changes
        match git_manager.has_changes(repo) {
            Ok(false) => {
                println!("{}", "[no changes]".dimmed());
                skipped += 1;
                continue;
            }
            Ok(true) => {}
            Err(e) => {
                println!("{}", format!("[error: {}]", e).red());
                failed += 1;
                continue;
            }
        }

        // Show changed files
        match git_manager.get_changed_files(repo) {
            Ok(files) => {
                if files.len() <= 3 {
                    println!("{}", format!("({})", files.join(", ")).dimmed());
                } else {
                    println!(
                        "{}",
                        format!("({} and {} more)", files[..2].join(", "), files.len() - 2).dimmed()
                    );
                }
            }
            Err(_) => println!(),
        }

        // Save (add, commit, push)
        match git_manager.save(repo, &commit_message) {
            Ok(()) => {
                println!("    {} committed and pushed", "✓".green());
                saved += 1;
            }
            Err(e) => {
                println!("    {} {}", "✗".red(), e);
                failed += 1;
            }
        }
    }

    // Print summary
    println!();
    println!("{}", "Summary:".bold());
    if saved > 0 {
        println!("  {} {}", "Saved:".green(), saved);
    }
    if skipped > 0 {
        println!("  {} {}", "Skipped:".dimmed(), skipped);
    }
    if failed > 0 {
        println!("  {} {}", "Failed:".red(), failed);
    }

    if failed > 0 {
        anyhow::bail!("{} repositories failed to save", failed);
    }

    Ok(())
}
