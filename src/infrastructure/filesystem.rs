//! # Filesystem Operations Module
//!
//! This module provides filesystem operations for the application.
//! It includes path resolution, file reading/writing, and directory operations.

use std::path::{Path, PathBuf};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use anyhow::{Result, Context};

/// Filesystem operations provider
pub struct FilesystemProvider;

impl FilesystemProvider {
    /// Create a new filesystem provider
    pub fn new() -> Self {
        Self {}
    }
    
    /// Check if a path exists
    pub fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
    
    /// Check if a path is a directory
    pub fn is_directory(&self, path: &Path) -> bool {
        path.is_dir()
    }
    
    /// Create a directory and all parent directories if they don't exist
    pub fn create_directory(&self, path: &Path) -> Result<()> {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory at {}", path.display()))
    }
    
    /// Read a file to a string
    pub fn read_file(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read file at {}", path.display()))
    }
    
    /// Write a string to a file
    pub fn write_file(&self, path: &Path, content: &str, overwrite: bool) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            self.create_directory(parent)?;
        }
        
        // Open file with appropriate options
        let file_result = if overwrite {
            File::create(path)
        } else {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
        };
        
        let mut file = file_result
            .with_context(|| format!("Failed to open file at {} for writing", path.display()))?;
        
        // Write content
        file.write_all(content.as_bytes())
            .with_context(|| format!("Failed to write to file at {}", path.display()))?;
        
        Ok(())
    }
    
    /// Copy a file from source to destination
    pub fn copy_file(&self, source: &Path, destination: &Path) -> Result<u64> {
        // Create parent directories if they don't exist
        if let Some(parent) = destination.parent() {
            self.create_directory(parent)?;
        }
        
        fs::copy(source, destination)
            .with_context(|| format!("Failed to copy from {} to {}", 
                source.display(), destination.display()))
    }
    
    /// List files in a directory
    pub fn list_directory(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(path)
            .with_context(|| format!("Failed to read directory at {}", path.display()))?;
        
        let mut files = Vec::new();
        for entry in entries {
            let entry = entry?;
            files.push(entry.path());
        }
        
        Ok(files)
    }
    
    /// Remove a file or empty directory
    pub fn remove(&self, path: &Path) -> Result<()> {
        if path.is_dir() {
            fs::remove_dir(path)
                .with_context(|| format!("Failed to remove directory at {}", path.display()))
        } else {
            fs::remove_file(path)
                .with_context(|| format!("Failed to remove file at {}", path.display()))
        }
    }
    
    /// Remove a directory and all its contents
    pub fn remove_recursive(&self, path: &Path) -> Result<()> {
        fs::remove_dir_all(path)
            .with_context(|| format!("Failed to remove directory and contents at {}", path.display()))
    }
}

impl Default for FilesystemProvider {
    fn default() -> Self {
        Self::new()
    }
}