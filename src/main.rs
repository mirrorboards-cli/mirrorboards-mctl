use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use clap::{Parser, Subcommand};
use log::{debug, error, info, warn, LevelFilter};
use serde::Deserialize;
use simple_logger::SimpleLogger;
use toml;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to the mirror.toml configuration file
    #[arg(short, long, default_value = "mirror.toml")]
    config: String,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Synchronize repositories from git to local computer
    Sync,
}

#[derive(Debug, Deserialize)]
struct Config {
    repositories: Vec<Repository>,
}

#[derive(Debug, Deserialize)]
struct Repository {
    origin: String,
    path: String,
    branch: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logger
    let log_level = if cli.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    SimpleLogger::new()
        .with_level(log_level)
        .init()
        .expect("Failed to initialize logger");

    match &cli.command {
        Commands::Sync => {
            info!("Starting repository synchronization");
            sync_repositories(&cli.config)?;
        }
    }

    Ok(())
}

fn sync_repositories(config_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse the configuration file
    let config_content = fs::read_to_string(config_path)
        .map_err(|e| format!("Failed to read config file {}: {}", config_path, e))?;
    
    let config: Config = toml::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config file {}: {}", config_path, e))?;

    info!("Found {} repositories in configuration", config.repositories.len());

    // Process each repository
    for repo in config.repositories {
        if repo.origin.starts_with('#') || repo.path.starts_with('#') {
            debug!("Skipping commented repository: {}", repo.path);
            continue;
        }

        let path = PathBuf::from(&repo.path);
        
        if path.exists() {
            update_repository(&repo)?;
        } else {
            clone_repository(&repo)?;
        }
    }

    info!("Repository synchronization completed successfully");
    Ok(())
}

fn clone_repository(repo: &Repository) -> Result<(), Box<dyn std::error::Error>> {
    info!("Cloning repository {} to {}", repo.origin, repo.path);
    
    // Create parent directories if they don't exist
    if let Some(parent) = Path::new(&repo.path).parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory {}: {}", parent.display(), e))?;
    }

    // Prepare git clone command
    let mut cmd = Command::new("git");
    cmd.arg("clone").arg(&repo.origin).arg(&repo.path);

    // Add branch if specified
    if let Some(branch) = &repo.branch {
        cmd.arg("--branch").arg(branch);
    }

    // Execute the command
    let output = cmd.output()
        .map_err(|e| format!("Failed to execute git clone: {}", e))?;

    handle_git_output(output, "clone")?;
    Ok(())
}

fn update_repository(repo: &Repository) -> Result<(), Box<dyn std::error::Error>> {
    info!("Updating repository in {}", repo.path);
    
    // Check if it's a git repository
    let git_dir = Path::new(&repo.path).join(".git");
    if !git_dir.exists() {
        warn!("{} exists but is not a git repository. Skipping.", repo.path);
        return Ok(());
    }

    // Fetch updates
    let output = Command::new("git")
        .current_dir(&repo.path)
        .arg("fetch")
        .output()
        .map_err(|e| format!("Failed to execute git fetch in {}: {}", repo.path, e))?;

    handle_git_output(output, "fetch")?;

    // Check if we need to switch branches
    if let Some(branch) = &repo.branch {
        // Get current branch
        let current_branch = get_current_branch(&repo.path)?;
        
        if current_branch != *branch {
            info!("Switching from branch {} to {}", current_branch, branch);
            
            // Check if the branch exists locally
            let branches = get_local_branches(&repo.path)?;
            
            if branches.contains(branch) {
                // Checkout existing branch
                let output = Command::new("git")
                    .current_dir(&repo.path)
                    .args(["checkout", branch])
                    .output()
                    .map_err(|e| format!("Failed to checkout branch {}: {}", branch, e))?;
                
                handle_git_output(output, "checkout")?;
            } else {
                // Create and checkout new branch
                let output = Command::new("git")
                    .current_dir(&repo.path)
                    .args(["checkout", "-b", branch, &format!("origin/{}", branch)])
                    .output()
                    .map_err(|e| format!("Failed to checkout new branch {}: {}", branch, e))?;
                
                handle_git_output(output, "checkout -b")?;
            }
        }
    }

    // Pull updates
    let output = Command::new("git")
        .current_dir(&repo.path)
        .arg("pull")
        .output()
        .map_err(|e| format!("Failed to execute git pull in {}: {}", repo.path, e))?;

    handle_git_output(output, "pull")?;
    Ok(())
}

fn get_current_branch(repo_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("Failed to get current branch: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to get current branch: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(branch)
}

fn get_local_branches(repo_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["branch", "--list"])
        .output()
        .map_err(|e| format!("Failed to list branches: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to list branches: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    let branches = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().trim_start_matches('*').trim().to_string())
        .filter(|branch| !branch.is_empty())
        .collect();

    Ok(branches)
}

fn handle_git_output(output: Output, operation: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        error!("Git {} failed: {}", operation, error_msg);
        return Err(format!("Git {} operation failed: {}", operation, error_msg).into());
    }
    
    let output_msg = String::from_utf8_lossy(&output.stdout);
    debug!("Git {} output: {}", operation, output_msg);
    
    Ok(())
}