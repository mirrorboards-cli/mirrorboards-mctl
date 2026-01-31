//! Snapshot command - create a snapshot of current repository states.

use crate::cli::commands::{print_error, print_info, print_success, print_warning};
use crate::core::config::{create_snapshot, MirrorConfig, DEFAULT_SNAPSHOT_FILE};
use crate::git::GitClient;
use anyhow::Result;
use chrono::Utc;
use colored::Colorize;
use std::path::Path;

pub fn execute(
    config_path: &str,
    workspace: Option<String>,
    output: Option<String>,
) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    // Determine output file
    let output_path = output.as_deref().unwrap_or(DEFAULT_SNAPSHOT_FILE);

    // Print header
    if let Some(ws) = &workspace {
        println!(
            "{} {}",
            "Creating snapshot for workspace:".bold(),
            ws.cyan()
        );
    } else {
        println!("{}", "Creating snapshot for all repositories".bold());
    }
    println!();

    let git = GitClient::new();

    // Collect current revisions
    let mut revisions: Vec<(String, String)> = Vec::new();
    let mut missing_count = 0;

    for repo in &config.repositories {
        // Apply workspace filter
        if let Some(ws) = &workspace {
            if !repo.is_in_workspace(ws) {
                continue;
            }
        }

        let local_path = Path::new(&repo.path);

        if !local_path.exists() || !git.is_git_repository(local_path) {
            print_warning(&format!("{}: Not cloned, skipping", repo.path));
            missing_count += 1;
            continue;
        }

        // Check for uncommitted changes
        match git.status(local_path) {
            Ok(status) => {
                if !status.is_clean() {
                    print_warning(&format!(
                        "{}: Has uncommitted changes ({})",
                        repo.path,
                        status.summary()
                    ));
                }
            }
            Err(e) => {
                print_warning(&format!("{}: Could not check status: {}", repo.path, e));
            }
        }

        // Get current HEAD revision
        match git.get_head_rev(local_path) {
            Ok(rev) => {
                print_info(&format!("{}: {}", repo.path, &rev[..12.min(rev.len())]));
                revisions.push((repo.path.clone(), rev));
            }
            Err(e) => {
                print_error(&format!("{}: Failed to get revision: {}", repo.path, e));
                missing_count += 1;
            }
        }
    }

    if revisions.is_empty() {
        print_error("No repositories available for snapshot");
        return Ok(());
    }

    // Create snapshot
    let snapshot = create_snapshot(&config.repositories, &revisions, workspace.as_deref());

    // Generate TOML with header comment
    let toml_content = toml::to_string_pretty(&snapshot)?;

    let header = format!(
        "# Snapshot created: {}\n# Source: {}\n{}",
        Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        config_path,
        if let Some(ws) = &workspace {
            format!("# Workspace: {}\n", ws)
        } else {
            String::new()
        }
    );

    let final_content = format!("{}\n{}", header, toml_content);

    // Write to file
    std::fs::write(output_path, final_content)?;

    // Summary
    println!();
    print_success(&format!(
        "Snapshot created: {} ({} repositories)",
        output_path,
        revisions.len()
    ));

    if missing_count > 0 {
        print_warning(&format!("{} repositories were skipped", missing_count));
    }

    println!();
    println!("To restore from this snapshot:");
    println!(
        "  {} --config {} sync",
        "mctl".cyan(),
        output_path.cyan()
    );

    Ok(())
}
