use crate::config::Config;
use crate::git::GitHandler;
use crate::output;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Checks the status of all repositories defined in the configuration
/// Only displays status for repositories with changes
pub fn check_status(config: Config) -> Result<()> {
    let _git_handler = GitHandler::new(); // Prefix with underscore to indicate it's intentionally unused
    let mut with_changes = 0;
    let mut clean = 0;
    let total = config.repositories.len();

    for repo in config.repositories {
        // Skip repositories without git
        if !repo.git {
            continue;
        }

        let path = std::path::PathBuf::from(&repo.path);
        
        // Skip repositories that don't exist or aren't git repositories
        if !GitHandler::repository_exists(&path) {
            continue;
        }

        // Check if repository has changes
        if has_changes(&path)? {
            // Display repository name with color
            println!("\n{}", output::colorize(&format!("Repository: {}", path.display()), "bold blue"));
            
            // Display git status
            display_status(&path)?;
            with_changes += 1;
        } else {
            clean += 1;
        }
    }

    // Print summary
    print_status_summary(total, with_changes, clean);
    Ok(())
}

/// Checks if a repository has any uncommitted changes
fn has_changes(path: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .with_context(|| format!("Failed to check status in {}", path.display()))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git status check failed: {}", error);
    }

    // If output is not empty, there are changes
    Ok(!output.stdout.is_empty())
}

/// Displays the git status for a repository with colorized output
fn display_status(path: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("status")
        .arg("--short")
        .output()
        .with_context(|| format!("Failed to get status in {}", path.display()))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git status display failed: {}", error);
    }

    let status = String::from_utf8_lossy(&output.stdout);
    
    // Colorize the output line by line
    for line in status.lines() {
        if line.is_empty() {
            continue;
        }
        
        // First two characters indicate the status
        let status_code = if line.len() >= 2 { &line[0..2] } else { "  " };
        let file_part = if line.len() > 3 { &line[3..] } else { "" };
        
        // Color based on status code
        let colored_status = match status_code {
            "M " | " M" => output::colorize(status_code, "green"),    // Modified
            "A " => output::colorize(status_code, "cyan"),            // Added
            " D" | "D " => output::colorize(status_code, "red"),      // Deleted
            "R " => output::colorize(status_code, "blue"),            // Renamed
            "??" => output::colorize(status_code, "yellow"),          // Untracked
            "UU" => output::colorize(status_code, "magenta"),         // Conflict
            _ => output::colorize(status_code, "white"),              // Other
        };
        
        // Output colorized line
        println!("{} {}", colored_status, output::colorize(file_part, "bold"));
    }

    Ok(())
}

/// Prints a summary of status check results with colors
fn print_status_summary(total: usize, with_changes: usize, clean: usize) {
    println!("\n{}", output::colorize("Summary:", "bold"));
    println!("Total repositories: {}", output::colorize(&total.to_string(), "bold"));
    
    // Color with_changes number based on whether there are changes
    let changes_color = if with_changes > 0 { "yellow" } else { "green" };
    println!("With changes: {}", output::colorize(&with_changes.to_string(), changes_color));
    
    // Color clean repositories in green
    println!("Clean: {}", output::colorize(&clean.to_string(), "green"));
}