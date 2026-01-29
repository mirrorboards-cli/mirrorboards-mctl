use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;
use tabled::{Table, Tabled};

use mirror_sdk::{ConfigLoader, ConfigRepoManager, MirrorError, SnapshotManager};

#[derive(Tabled)]
struct SnapshotRow {
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Created")]
    created_at: String,
    #[tabled(rename = "Repos")]
    repos: usize,
    #[tabled(rename = "Description")]
    description: String,
}

pub fn run(config_path: &str) -> Result<()> {
    // Load config
    let config = ConfigLoader::new()
        .load(config_path)
        .context("Failed to load configuration")?;

    // Check if config-repo is configured
    let config_repo = config
        .config_repo
        .clone()
        .ok_or(MirrorError::ConfigRepoNotConfigured)
        .context("No config-repo configured in mirror.toml")?;

    // Get base path
    let base_path = Path::new(config_path)
        .parent()
        .unwrap_or(Path::new("."));

    // Sync config repo
    let config_repo_path = base_path.join(".mctl").join("config-repo");
    let config_repo_manager = ConfigRepoManager::new(config_repo, &config_repo_path);

    print!("Syncing config repository... ");
    config_repo_manager
        .sync()
        .context("Failed to sync config repository")?;
    println!("{}", "✓".green());
    println!();

    // List snapshots
    let snapshots_dir = config_repo_manager.snapshots_dir();
    let snapshot_manager = SnapshotManager::new(&snapshots_dir);

    let snapshots = snapshot_manager.list().context("Failed to list snapshots")?;

    if snapshots.is_empty() {
        println!("{}", "No snapshots found".yellow());
        return Ok(());
    }

    let rows: Vec<SnapshotRow> = snapshots
        .iter()
        .map(|s| SnapshotRow {
            name: s.name.clone(),
            created_at: s.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            repos: s.repository_count,
            description: s.description.clone().unwrap_or_default(),
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);

    Ok(())
}
