use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use mirror_sdk::{ConfigLoader, ConfigRepoManager, MirrorError};

pub fn run(config_path: &str, message: &str) -> Result<()> {
    // Load config
    let config = ConfigLoader::new()
        .load(config_path)
        .context("Failed to load configuration")?;

    // Check if config-repo is configured
    let config_repo = config
        .config_repo
        .ok_or(MirrorError::ConfigRepoNotConfigured)
        .context("No config-repo configured in mirror.toml")?;

    println!("{}", "Saving configuration to config repository...".cyan());
    println!("  Remote: {}", config_repo.git.dimmed());
    println!("  Branch: {}", config_repo.branch.dimmed());
    println!();

    // Read the current config file content
    let config_content = std::fs::read_to_string(config_path)
        .context("Failed to read config file")?;

    // Get config repo local path (in .mctl directory)
    let base_path = Path::new(config_path)
        .parent()
        .unwrap_or(Path::new("."));
    let config_repo_path = base_path.join(".mctl").join("config-repo");

    let manager = ConfigRepoManager::new(config_repo, &config_repo_path);

    // Save config and push
    print!("  Syncing config repository... ");
    manager.sync().context("Failed to sync config repository")?;
    println!("{}", "✓".green());

    print!("  Writing config... ");
    let config_file_path = manager.config_file_path();
    if let Some(parent) = config_file_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&config_file_path, &config_content)?;
    println!("{}", "✓".green());

    print!("  Committing... ");
    manager.commit(message).context("Failed to commit")?;
    println!("{}", "✓".green());

    print!("  Pushing... ");
    manager.push().context("Failed to push")?;
    println!("{}", "✓".green());

    println!();
    println!("{}", "Configuration saved successfully!".green().bold());

    Ok(())
}
