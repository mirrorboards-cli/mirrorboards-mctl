//! Init command - create a new mirror.toml configuration.

use crate::cli::commands::{print_error, print_success, print_warning};
use crate::core::config::ConfigManager;
use anyhow::Result;
use std::path::Path;

pub fn execute(config_path: &str, force: bool) -> Result<()> {
    let path = Path::new(config_path);

    if path.exists() && !force {
        print_error(&format!(
            "Configuration file already exists: {}",
            config_path
        ));
        print_warning("Use --force to overwrite");
        return Ok(());
    }

    if path.exists() && force {
        print_warning(&format!("Overwriting existing configuration: {}", config_path));
    }

    // Create empty config
    let manager = if force && path.exists() {
        std::fs::remove_file(path)?;
        ConfigManager::create(path)?
    } else {
        ConfigManager::create(path)?
    };

    manager.save()?;

    print_success(&format!("Created configuration file: {}", config_path));

    // Print next steps
    println!();
    println!("Next steps:");
    println!("  1. Add repositories:");
    println!("     mctl add git@github.com:owner/repo.git");
    println!();
    println!("  2. Sync repositories:");
    println!("     mctl sync");
    println!();
    println!("  3. Check status:");
    println!("     mctl status");

    Ok(())
}
