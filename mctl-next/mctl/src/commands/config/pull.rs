use std::path::Path;

use anyhow::{Context, Result};
use colored::Colorize;

use mirror_sdk::{ConfigLoader, ConfigRepoManager, MirrorError};

pub fn run(config_path: &str) -> Result<()> {
    // Load config to get config-repo settings
    let config = ConfigLoader::new()
        .load(config_path)
        .context("Failed to load configuration")?;

    // Check if config-repo is configured
    let config_repo = config
        .config_repo
        .ok_or(MirrorError::ConfigRepoNotConfigured)
        .context("No config-repo configured in mirror.toml")?;

    println!("{}", "Pulling configuration from config repository...".cyan());
    println!("  Remote: {}", config_repo.git.dimmed());
    println!("  Branch: {}", config_repo.branch.dimmed());
    println!();

    // Get config repo local path
    let base_path = Path::new(config_path)
        .parent()
        .unwrap_or(Path::new("."));
    let config_repo_path = base_path.join(".mctl").join("config-repo");

    let manager = ConfigRepoManager::new(config_repo, &config_repo_path);

    // Pull config
    print!("  Pulling from remote... ");
    let content = manager
        .pull_config()
        .context("Failed to pull configuration")?;
    println!("{}", "✓".green());

    // Write to local config file
    print!("  Updating local config... ");
    std::fs::write(config_path, &content).context("Failed to write config file")?;
    println!("{}", "✓".green());

    println!();
    println!("{}", "Configuration pulled successfully!".green().bold());

    Ok(())
}
