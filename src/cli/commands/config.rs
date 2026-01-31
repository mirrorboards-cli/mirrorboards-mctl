//! Config command - remote config management.

use crate::cli::commands::{print_error, print_info, print_success, print_warning};
use crate::core::config::{ConfigManager, MirrorConfig, RawMirrorConfig, RemoteConfig};
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Initialize remote config.
pub fn init_remote(config_path: &str, git_url: &str, branch: &str, remote_path: &str) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        print_info("Run 'mctl init' first to create a configuration");
        return Ok(());
    }

    let mut manager = ConfigManager::open(config_file)?;

    let remote = RemoteConfig {
        git: git_url.to_string(),
        branch: branch.to_string(),
        path: remote_path.to_string(),
    };

    manager.set_remote(remote);
    manager.save()?;

    print_success("Remote config initialized");
    println!("  Git: {}", git_url);
    println!("  Branch: {}", branch);
    println!("  Path: {}", remote_path);
    println!();
    println!("Next steps:");
    println!("  {} - push local config to remote", "mctl config push".cyan());
    println!("  {} - pull config from remote", "mctl config pull".cyan());

    Ok(())
}

/// Collect all include files recursively from a config file.
fn collect_include_files(config_path: &Path, base_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut visited = HashSet::new();
    collect_includes_recursive(config_path, base_dir, &mut files, &mut visited)?;
    Ok(files)
}

fn collect_includes_recursive(
    config_path: &Path,
    base_dir: &Path,
    files: &mut Vec<PathBuf>,
    visited: &mut HashSet<PathBuf>,
) -> Result<()> {
    let canonical = config_path.canonicalize()?;
    if visited.contains(&canonical) {
        return Ok(());
    }
    visited.insert(canonical.clone());

    // Parse config to get includes
    let content = std::fs::read_to_string(config_path)?;
    let raw_config: RawMirrorConfig = toml::from_str(&content)?;

    let config_dir = config_path.parent().unwrap_or(Path::new("."));

    for include_path in raw_config.get_includes() {
        let resolved = if Path::new(&include_path).is_absolute() {
            PathBuf::from(&include_path)
        } else {
            config_dir.join(&include_path)
        };

        if resolved.exists() {
            // Store relative path from base_dir
            if let Ok(relative) = resolved.strip_prefix(base_dir) {
                files.push(relative.to_path_buf());
            } else {
                // If outside base_dir, use the include path as-is
                files.push(PathBuf::from(&include_path));
            }

            // Recursively collect from included file
            collect_includes_recursive(&resolved, base_dir, files, visited)?;
        }
    }

    Ok(())
}

/// Pull config from remote.
pub fn pull(config_path: &str, verbose: bool) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    let remote = match &config.remote {
        Some(r) => r,
        None => {
            print_error("No remote config configured");
            print_info("Run 'mctl config init <git-url>' to set up remote config");
            return Ok(());
        }
    };

    print_info(&format!("Pulling from {}...", remote.git));

    let git = GitClient::new();

    // Clone to temp directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let version = crate::core::repository::VersionSpec::Branch(remote.branch.clone());

    if let Err(e) = git.clone(&remote.git, temp_path, &version) {
        print_error(&format!("Failed to clone remote: {}", e));
        return Ok(());
    }

    // Read remote config
    let remote_config_path = temp_path.join(&remote.path);

    if !remote_config_path.exists() {
        print_error(&format!(
            "Remote config file not found: {}",
            remote.path
        ));
        return Ok(());
    }

    // Get local base directory
    let local_base = config_file.parent().unwrap_or(Path::new("."));
    let remote_base = remote_config_path.parent().unwrap_or(temp_path);

    // Collect all include files from remote
    let include_files = collect_include_files(&remote_config_path, remote_base).unwrap_or_default();

    // Copy main config
    std::fs::copy(&remote_config_path, config_path)?;
    let mut copied_count = 1;

    // Copy all include files
    for include_file in &include_files {
        let remote_file = remote_base.join(include_file);
        let local_file = local_base.join(include_file);

        if remote_file.exists() {
            // Create parent directories
            if let Some(parent) = local_file.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::copy(&remote_file, &local_file)?;
            copied_count += 1;

            if verbose {
                print_info(&format!("  Pulled: {}", include_file.display()));
            }
        }
    }

    print_success(&format!("Config pulled from remote ({} files)", copied_count));

    if verbose {
        let new_config = MirrorConfig::load(config_file)?;
        println!();
        println!("Loaded {} repositories", new_config.repositories.len());
    }

    Ok(())
}

/// Push config to remote.
pub fn push(config_path: &str, message: &str, verbose: bool) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    let remote = match &config.remote {
        Some(r) => r,
        None => {
            print_error("No remote config configured");
            print_info("Run 'mctl config init <git-url>' to set up remote config");
            return Ok(());
        }
    };

    print_info(&format!("Pushing to {}...", remote.git));

    let git = GitClient::new();

    // Clone to temp directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let version = crate::core::repository::VersionSpec::Branch(remote.branch.clone());

    // Try to clone the specified branch, if it fails try default branch and create new
    let mut created_new_branch = false;
    if let Err(_) = git.clone(&remote.git, temp_path, &version) {
        // Branch doesn't exist, clone default and create new branch
        print_info(&format!("Branch '{}' doesn't exist, creating...", remote.branch));

        let default_version = crate::core::repository::VersionSpec::DefaultBranch;
        if let Err(e) = git.clone(&remote.git, temp_path, &default_version) {
            print_error(&format!("Failed to clone remote: {}", e));
            return Ok(());
        }

        // Create and checkout new branch
        if let Err(e) = git.checkout_new_branch(temp_path, &remote.branch) {
            print_error(&format!("Failed to create branch '{}': {}", remote.branch, e));
            return Ok(());
        }
        created_new_branch = true;
    }

    // Get local base directory
    let local_base = config_file.parent().unwrap_or(Path::new("."));

    // Copy local config to temp
    let remote_config_path = temp_path.join(&remote.path);
    let remote_base = remote_config_path.parent().unwrap_or(temp_path);

    // Create parent directories if needed
    if let Some(parent) = remote_config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Collect all include files from local config
    let include_files = collect_include_files(config_file, local_base).unwrap_or_default();

    // Copy main config
    std::fs::copy(config_path, &remote_config_path)?;
    let mut copied_count = 1;

    if verbose {
        print_info(&format!("  Pushing: {}", remote.path));
    }

    // Copy all include files
    for include_file in &include_files {
        let local_file = local_base.join(include_file);
        let remote_file = remote_base.join(include_file);

        if local_file.exists() {
            // Create parent directories
            if let Some(parent) = remote_file.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::copy(&local_file, &remote_file)?;
            copied_count += 1;

            if verbose {
                print_info(&format!("  Pushing: {}", include_file.display()));
            }
        }
    }

    // Commit and push
    git.add_all(temp_path)?;

    match git.commit(temp_path, message) {
        Ok(_) => {}
        Err(crate::core::error::GitError::NoChangesToCommit) => {
            print_info("No changes to push (config is up to date)");
            return Ok(());
        }
        Err(e) => {
            print_error(&format!("Failed to commit: {}", e));
            return Ok(());
        }
    }

    // Push (with -u for new branch)
    if created_new_branch {
        // Need to push with set-upstream
        let push_result = std::process::Command::new("git")
            .args(["push", "-u", "origin", &remote.branch])
            .current_dir(temp_path)
            .output();

        match push_result {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                print_error(&format!("Failed to push: {}", stderr));
                return Ok(());
            }
            Err(e) => {
                print_error(&format!("Failed to push: {}", e));
                return Ok(());
            }
        }
    } else {
        if let Err(e) = git.push(temp_path) {
            print_error(&format!("Failed to push: {}", e));
            return Ok(());
        }
    }

    print_success(&format!("Config pushed to remote ({} files)", copied_count));
    if created_new_branch {
        print_info(&format!("Created new branch: {}", remote.branch));
    }

    Ok(())
}

/// Show diff between local and remote config.
pub fn diff(config_path: &str) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    let remote = match &config.remote {
        Some(r) => r,
        None => {
            print_error("No remote config configured");
            return Ok(());
        }
    };

    print_info(&format!("Fetching from {}...", remote.git));

    let git = GitClient::new();

    // Clone to temp directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let version = crate::core::repository::VersionSpec::Branch(remote.branch.clone());

    if let Err(e) = git.clone(&remote.git, temp_path, &version) {
        print_error(&format!("Failed to clone remote: {}", e));
        return Ok(());
    }

    // Read remote config
    let remote_config_path = temp_path.join(&remote.path);

    if !remote_config_path.exists() {
        print_warning("Remote config file does not exist");
        println!();
        println!("Local config would be created on push");
        return Ok(());
    }

    // Get base directories
    let local_base = config_file.parent().unwrap_or(Path::new("."));
    let remote_base = remote_config_path.parent().unwrap_or(temp_path);

    // Collect all files to compare
    let local_includes = collect_include_files(config_file, local_base).unwrap_or_default();
    let remote_includes = collect_include_files(&remote_config_path, remote_base).unwrap_or_default();

    let mut all_files: HashSet<PathBuf> = HashSet::new();
    all_files.insert(PathBuf::from(&remote.path));
    for f in &local_includes {
        all_files.insert(f.clone());
    }
    for f in &remote_includes {
        all_files.insert(f.clone());
    }

    let mut has_diff = false;

    for file in all_files {
        let local_file = if file == PathBuf::from(&remote.path) {
            PathBuf::from(config_path)
        } else {
            local_base.join(&file)
        };
        let remote_file = remote_base.join(&file);

        let local_content = if local_file.exists() {
            std::fs::read_to_string(&local_file).ok()
        } else {
            None
        };

        let remote_content = if remote_file.exists() {
            std::fs::read_to_string(&remote_file).ok()
        } else {
            None
        };

        match (&local_content, &remote_content) {
            (Some(local), Some(remote)) if local != remote => {
                if !has_diff {
                    println!();
                    println!("{}", "Differences:".bold());
                }
                has_diff = true;
                println!();
                println!("{} {}", "File:".cyan(), file.display());
                print_file_diff(remote, local);
            }
            (Some(_), None) => {
                if !has_diff {
                    println!();
                    println!("{}", "Differences:".bold());
                }
                has_diff = true;
                println!();
                println!("{} {} {}", "File:".cyan(), file.display(), "(new, not in remote)".green());
            }
            (None, Some(_)) => {
                if !has_diff {
                    println!();
                    println!("{}", "Differences:".bold());
                }
                has_diff = true;
                println!();
                println!("{} {} {}", "File:".cyan(), file.display(), "(deleted locally)".red());
            }
            _ => {}
        }
    }

    if !has_diff {
        print_success("Local and remote configs are identical");
    }

    Ok(())
}

fn print_file_diff(remote_content: &str, local_content: &str) {
    let local_lines: Vec<&str> = local_content.lines().collect();
    let remote_lines: Vec<&str> = remote_content.lines().collect();

    println!("{}", "--- remote".red());
    println!("{}", "+++ local".green());

    let max_lines = local_lines.len().max(remote_lines.len());
    for i in 0..max_lines {
        let local_line = local_lines.get(i).copied().unwrap_or("");
        let remote_line = remote_lines.get(i).copied().unwrap_or("");

        if local_line != remote_line {
            if !remote_line.is_empty() {
                println!("{}", format!("- {}", remote_line).red());
            }
            if !local_line.is_empty() {
                println!("{}", format!("+ {}", local_line).green());
            }
        }
    }
}
