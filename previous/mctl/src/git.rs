use anyhow::{Context, Result};
use git2::Repository;
use std::path::Path;
use std::process::Command;
use crate::output;
use std::fs;

pub struct GitHandler;

impl GitHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn clone_repository(&self, url: &str, path: &Path) -> Result<()> {
        if path.exists() {
            output::print_skipping(path);
            return Ok(());
        }

        output::print_cloning(url, path);

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Use system git command which will use system's SSH configuration
        let output = Command::new("git")
            .arg("clone")
            .arg("--recurse-submodules")  // Clone with submodules
            .arg(url)
            .arg(path)
            .output()
            .with_context(|| format!("Failed to execute git clone command"))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git clone failed: {}", error);
        }

        Ok(())
    }

    pub fn remove_git_directory(&self, path: &Path) -> Result<()> {
        let git_dir = path.join(".git");
        if git_dir.exists() {
            output::print_info(&format!("Removing .git directory from {}", path.display()));
            fs::remove_dir_all(git_dir).with_context(|| format!("Failed to remove .git directory from {}", path.display()))?;
        }
        Ok(())
    }

    pub fn repository_exists(path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        match Repository::open(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn update_submodules(&self, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        output::print_info(&format!("Updating submodules in {}", path.display()));

        // Initialize and update submodules
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("submodule")
            .arg("update")
            .arg("--init")
            .arg("--recursive")
            .output()
            .with_context(|| format!("Failed to update submodules in {}", path.display()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Submodule update failed: {}", error);
        }

        Ok(())
    }
    
    /// Checks if a repository has any uncommitted changes
    pub fn has_changes(&self, path: &Path) -> Result<bool> {
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
    
    /// Commits all changes in a repository with the specified message
    pub fn commit_changes(&self, path: &Path, message: &str) -> Result<()> {
        // Stage all changes
        let stage_output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("add")
            .arg("--all")
            .output()
            .with_context(|| format!("Failed to stage changes in {}", path.display()))?;
            
        if !stage_output.status.success() {
            let error = String::from_utf8_lossy(&stage_output.stderr);
            anyhow::bail!("Git add failed: {}", error);
        }
        
        // Commit changes
        let commit_output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("commit")
            .arg("-m")
            .arg(message)
            .output()
            .with_context(|| format!("Failed to commit changes in {}", path.display()))?;
            
        if !commit_output.status.success() {
            let error = String::from_utf8_lossy(&commit_output.stderr);
            // Don't treat "nothing to commit" as an error
            if error.contains("nothing to commit") {
                return Ok(());
            }
            anyhow::bail!("Git commit failed: {}", error);
        }
        
        Ok(())
    }
    
    /// Pushes committed changes to the remote repository
    pub fn push_changes(&self, path: &Path) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("push")
            .output()
            .with_context(|| format!("Failed to push changes in {}", path.display()))?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git push failed: {}", error);
        }
        
        Ok(())
    }
    
    /// Gets the origin URL of a repository
    pub fn get_origin_url(&self, path: &Path) -> Result<String> {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .output()
            .with_context(|| format!("Failed to get origin URL for {}", path.display()))?;
            
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to get origin URL: {}", error);
        }
        
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(url)
    }
    
    /// Extracts repository name from a Git URL
    /// For example: "git@github.com:org/repo.git" -> "org/repo"
    pub fn extract_repo_name_from_url(&self, url: &str) -> String {
        // For SSH URLs like git@github.com:org/repo.git
        if url.contains('@') && url.contains(':') {
            let parts: Vec<&str> = url.split(':').collect();
            if parts.len() > 1 {
                let repo_part = parts[1];
                return repo_part.trim_end_matches(".git").to_string();
            }
        }
        
        // For HTTPS URLs like https://github.com/org/repo.git
        if url.contains("://") {
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 3 {
                let org_idx = parts.len() - 2;
                let repo_idx = parts.len() - 1;
                
                let org = parts[org_idx];
                let mut repo = parts[repo_idx].to_string();
                
                // Remove .git suffix if present
                if repo.ends_with(".git") {
                    repo = repo.trim_end_matches(".git").to_string();
                }
                
                return format!("{}/{}", org, repo);
            }
        }
        
        // If we can't parse it, return a placeholder
        String::from("unknown-repo")
    }
}