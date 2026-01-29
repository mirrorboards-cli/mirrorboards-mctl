use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use mirror_sdk::{
    ConfigLoader, ConfigRepoManager, ConfigValidator, GitManager, MirrorError, SnapshotManager,
};

pub fn run(config_path: &str, name: &str, description: Option<&str>) -> Result<()> {
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

    println!("{}", format!("Creating snapshot '{}'...", name).cyan());
    println!();

    // Get base path
    let base_path = Path::new(config_path)
        .parent()
        .unwrap_or(Path::new("."));

    let git_manager = GitManager::new(base_path);

    // Sync config repo first
    let config_repo_path = base_path.join(".mctl").join("config-repo");
    let config_repo_manager = ConfigRepoManager::new(config_repo.clone(), &config_repo_path);

    print!("  Syncing config repository... ");
    config_repo_manager
        .sync()
        .context("Failed to sync config repository")?;
    println!("{}", "✓".green());

    // Create snapshot manager pointing to snapshots dir in config repo
    let snapshots_dir = config_repo_manager.snapshots_dir();
    let snapshot_manager = SnapshotManager::new(&snapshots_dir);

    // Create snapshot
    print!("  Capturing repository states... ");
    let snapshot = snapshot_manager
        .create(name, &config, &git_manager, description)
        .context("Failed to create snapshot")?;
    println!("{}", "✓".green());

    println!(
        "    Captured {} repositories",
        snapshot.repositories.len()
    );

    // Save snapshot to file
    print!("  Saving snapshot... ");
    snapshot_manager
        .save(&snapshot)
        .context("Failed to save snapshot")?;
    println!("{}", "✓".green());

    // Commit and push to config repo
    print!("  Committing to config repository... ");
    let commit_message = format!("snapshot: create '{}'", name);
    config_repo_manager
        .commit(&commit_message)
        .context("Failed to commit snapshot")?;
    println!("{}", "✓".green());

    print!("  Pushing... ");
    config_repo_manager
        .push()
        .context("Failed to push snapshot")?;
    println!("{}", "✓".green());

    println!();
    println!(
        "{}",
        format!("Snapshot '{}' created successfully!", name)
            .green()
            .bold()
    );

    Ok(())
}
