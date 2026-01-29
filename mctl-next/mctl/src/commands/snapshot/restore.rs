use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use mirror_sdk::{
    ConfigLoader, ConfigRepoManager, ConfigValidator, GitManager, MirrorError, RefSpec,
    Repository, SnapshotManager,
};

pub fn run(config_path: &str, name: &str) -> Result<()> {
    // Load and validate config
    let config = ConfigLoader::new()
        .load(config_path)
        .context("Failed to load configuration")?;

    ConfigValidator::validate(&config).context("Configuration validation failed")?;

    // Check if config-repo is configured
    let config_repo = config
        .config_repo
        .clone()
        .ok_or(MirrorError::ConfigRepoNotConfigured)
        .context("No config-repo configured in mirror.toml")?;

    println!("{}", format!("Restoring snapshot '{}'...", name).cyan());
    println!();

    // Get base path
    let base_path = Path::new(config_path)
        .parent()
        .unwrap_or(Path::new("."));

    // Sync config repo
    let config_repo_path = base_path.join(".mctl").join("config-repo");
    let config_repo_manager = ConfigRepoManager::new(config_repo, &config_repo_path);

    print!("  Syncing config repository... ");
    config_repo_manager
        .sync()
        .context("Failed to sync config repository")?;
    println!("{}", "✓".green());

    // Load snapshot
    let snapshots_dir = config_repo_manager.snapshots_dir();
    let snapshot_manager = SnapshotManager::new(&snapshots_dir);

    let snapshot = snapshot_manager
        .load(name)
        .context("Failed to load snapshot")?;

    println!(
        "  Snapshot created: {}",
        snapshot.created_at.format("%Y-%m-%d %H:%M:%S UTC")
    );
    if let Some(desc) = &snapshot.description {
        println!("  Description: {}", desc);
    }
    println!();

    let git_manager = GitManager::new(base_path);

    let mut restored = 0;
    let mut skipped = 0;
    let mut failed = 0;

    for snap_repo in &snapshot.repositories {
        print!("  {} ", snap_repo.path);

        // Find matching repo in current config
        let current_repo = config.find_by_path(&snap_repo.path);

        if current_repo.is_none() {
            println!("{}", "[not in current config, skipping]".dimmed());
            skipped += 1;
            continue;
        }

        // Create temporary repo with rev refspec for checkout
        let temp_repo = Repository {
            git: snap_repo.git.clone(),
            path: snap_repo.path.clone(),
            ref_spec: RefSpec::Rev(snap_repo.rev.clone()),
            workspaces: vec![],
        };

        // Check if repo exists locally
        if !git_manager.exists(&temp_repo) {
            println!("{}", "[not cloned, skipping]".yellow());
            skipped += 1;
            continue;
        }

        // Checkout the specific revision
        match git_manager.update(&temp_repo) {
            Ok(()) => {
                println!(
                    "{}",
                    format!("→ {}", &snap_repo.rev[..7.min(snap_repo.rev.len())]).green()
                );
                restored += 1;
            }
            Err(e) => {
                println!("{}", format!("[error: {}]", e).red());
                failed += 1;
            }
        }
    }

    // Print summary
    println!();
    println!("{}", "Summary:".bold());
    if restored > 0 {
        println!("  {} {}", "Restored:".green(), restored);
    }
    if skipped > 0 {
        println!("  {} {}", "Skipped:".dimmed(), skipped);
    }
    if failed > 0 {
        println!("  {} {}", "Failed:".red(), failed);
    }

    if failed > 0 {
        anyhow::bail!("{} repositories failed to restore", failed);
    }

    println!();
    println!(
        "{}",
        format!("Snapshot '{}' restored successfully!", name)
            .green()
            .bold()
    );
    println!(
        "{}",
        "Note: Repositories are now in detached HEAD state.".yellow()
    );

    Ok(())
}
