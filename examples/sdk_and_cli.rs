//! This example demonstrates how to use the mirror-sdk and mirror-cli together.
//! 
//! It creates a mirror.toml file using the SDK, then uses the CLI to add a repository,
//! and finally uses the SDK again to verify the repository was added correctly.

use mirror_sdk::{MirrorSdk, MirrorError};
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), MirrorError> {
    // Create a temporary directory for our example
    let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join("mirror.toml");
    
    println!("Using temporary directory: {}", temp_path.display());
    println!("Config path: {}", config_path.display());
    
    // Step 1: Use the SDK to create a new mirror.toml file
    let sdk = MirrorSdk::new();
    let config = sdk.new_config();
    sdk.save_config(&config, &config_path)?;
    
    println!("Created empty mirror.toml file");
    
    // Step 2: Use the CLI to add a repository
    let cli_path = get_cli_path()?;
    
    let add_output = Command::new(cli_path)
        .args(&[
            "--config", config_path.to_str().unwrap(),
            "add", 
            "--origin", "git@github.com:example/repo.git",
            "--path", "example/repo",
            "--branch", "main",
            "--tags", "test,example"
        ])
        .output()
        .expect("Failed to execute CLI add command");
    
    if add_output.status.success() {
        println!("Successfully added repository using CLI");
        println!("{}", String::from_utf8_lossy(&add_output.stdout));
    } else {
        let error = String::from_utf8_lossy(&add_output.stderr);
        println!("Failed to add repository: {}", error);
        return Err(MirrorError::InvalidConfiguration(
            format!("CLI command failed: {}", error)
        ));
    }
    
    // Step 3: Use the SDK to verify the repository was added correctly
    let updated_config = sdk.load_config(&config_path)?;
    
    // Verify the repository count
    assert_eq!(updated_config.repositories.len(), 1, "Expected 1 repository");
    
    // Verify the repository details
    let repo = &updated_config.repositories[0];
    assert_eq!(repo.origin, "git@github.com:example/repo.git");
    assert_eq!(repo.path, "example/repo");
    assert_eq!(repo.branch, "main");
    assert_eq!(repo.tags, vec!["test", "example"]);
    
    println!("Verification successful");
    println!("Repository details:");
    println!("  Origin: {}", repo.origin);
    println!("  Path: {}", repo.path);
    println!("  Branch: {}", repo.branch);
    println!("  Tags: {:?}", repo.tags);
    
    // Step 4: Use the CLI to list repositories
    let list_output = Command::new(cli_path)
        .args(&[
            "--config", config_path.to_str().unwrap(),
            "list"
        ])
        .output()
        .expect("Failed to execute CLI list command");
    
    if list_output.status.success() {
        println!("\nCLI list output:");
        println!("{}", String::from_utf8_lossy(&list_output.stdout));
    } else {
        let error = String::from_utf8_lossy(&list_output.stderr);
        println!("Failed to list repositories: {}", error);
    }
    
    // The temporary directory will be automatically cleaned up when it goes out of scope
    println!("\nExample completed successfully");
    Ok(())
}

/// Get the path to the mirror-cli executable
fn get_cli_path() -> Result<PathBuf, MirrorError> {
    // Try to find the CLI in the target directory
    let target_debug = PathBuf::from("target/debug/mirror-cli");
    let target_release = PathBuf::from("target/release/mirror-cli");
    
    if target_debug.exists() {
        return Ok(target_debug);
    } else if target_release.exists() {
        return Ok(target_release);
    } else {
        // If not found, try to build it
        println!("Building mirror-cli...");
        let build_output = Command::new("cargo")
            .args(&["build", "--bin", "mirror-cli"])
            .output()
            .expect("Failed to execute cargo build command");
        
        if build_output.status.success() {
            println!("Successfully built mirror-cli");
            Ok(target_debug)
        } else {
            let error = String::from_utf8_lossy(&build_output.stderr);
            Err(MirrorError::InvalidConfiguration(
                format!("Failed to build mirror-cli: {}", error)
            ))
        }
    }
}