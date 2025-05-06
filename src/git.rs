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
}