//! Validate command - validate the configuration file.

use crate::cli::commands::{print_error, print_info, print_success, print_warning};
use crate::core::config::MirrorConfig;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub fn execute(config_path: &str) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    println!("{}", "Validating configuration...".bold());
    println!();

    // Try to load and resolve includes
    let config = match MirrorConfig::load(config_file) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to load configuration: {}", e));
            return Ok(());
        }
    };

    // Show source files
    if config.source_files.len() > 1 {
        print_info(&format!(
            "Loaded {} files (with includes):",
            config.source_files.len()
        ));
        for file in &config.source_files {
            println!("    {}", file.display());
        }
        println!();
    }

    // Validate each repository
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // A configuration that uses workspaces at all should name one for every repository;
    // one that uses none (the production mirror) gets no warning per repository.
    let uses_workspaces = config
        .repositories
        .iter()
        .any(|repo| !repo.workspaces.is_empty());

    for repo in &config.repositories {
        // Validation
        if let Err(msg) = repo.validate() {
            errors.push(format!("{}: {}", repo.path, msg));
        }

        // Warnings
        if uses_workspaces && repo.workspaces.is_empty() {
            warnings.push(format!("{}: No workspaces assigned", repo.path));
        }
    }

    // Check for remote config
    if config.remote.is_some() {
        print_info("Remote config is configured");
    }

    // Print results
    if errors.is_empty() && warnings.is_empty() {
        print_success(&format!(
            "Configuration is valid ({} repositories)",
            config.repositories.len()
        ));
    } else {
        if !errors.is_empty() {
            println!();
            println!("{}", "Errors:".red().bold());
            for error in &errors {
                print_error(error);
            }
        }

        if !warnings.is_empty() {
            println!();
            println!("{}", "Warnings:".yellow().bold());
            for warning in &warnings {
                print_warning(warning);
            }
        }

        println!();
        if errors.is_empty() {
            print_success(&format!(
                "Configuration is valid with {} warning(s) ({} repositories)",
                warnings.len(),
                config.repositories.len()
            ));
        } else {
            print_error(&format!(
                "Configuration has {} error(s)",
                errors.len()
            ));
        }
    }

    // Print workspace summary
    let workspaces = config.list_workspaces();
    if !workspaces.is_empty() {
        println!();
        println!("{}: {}", "Workspaces".dimmed(), workspaces.join(", "));
    }

    Ok(())
}
