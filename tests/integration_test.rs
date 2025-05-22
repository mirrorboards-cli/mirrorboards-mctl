use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

// This test demonstrates the integration between the mirror-sdk and mirror-cli
// It uses the CLI to create and manipulate a mirror.toml file, then uses the SDK
// to verify the results programmatically.

#[test]
fn test_sdk_cli_integration() {
    // Create a temporary directory for our test
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let temp_path = temp_dir.path();
    let config_path = temp_path.join("mirror.toml");
    
    // Step 1: Use the CLI to initialize a new mirror.toml file
    let init_output = Command::new("cargo")
        .args(&["run", "--bin", "mirror-cli", "--", "init", "--config", config_path.to_str().unwrap()])
        .output()
        .expect("Failed to execute CLI init command");
    
    assert!(init_output.status.success(), "CLI init command failed");
    assert!(config_path.exists(), "mirror.toml file was not created");
    
    // Step 2: Use the CLI to add a repository
    let add_output = Command::new("cargo")
        .args(&[
            "run", "--bin", "mirror-cli", "--", 
            "add", 
            "--config", config_path.to_str().unwrap(),
            "--origin", "git@github.com:example/repo.git",
            "--path", "example/repo",
            "--branch", "main",
            "--tags", "test,example"
        ])
        .output()
        .expect("Failed to execute CLI add command");
    
    assert!(add_output.status.success(), "CLI add command failed");
    
    // Step 3: Use the SDK to verify the repository was added correctly
    let verify_code = r#"
        use mirror_sdk::{MirrorSdk, MirrorError};
        use std::path::PathBuf;
        
        fn main() -> Result<(), MirrorError> {
            let sdk = MirrorSdk::new();
            let config_path = PathBuf::from(std::env::args().nth(1).unwrap());
            let config = sdk.load_config(config_path)?;
            
            // Verify the repository count
            assert_eq!(config.repositories.len(), 1, "Expected 1 repository");
            
            // Verify the repository details
            let repo = &config.repositories[0];
            assert_eq!(repo.origin, "git@github.com:example/repo.git");
            assert_eq!(repo.path, "example/repo");
            assert_eq!(repo.branch, "main");
            assert_eq!(repo.tags, vec!["test", "example"]);
            
            println!("Verification successful");
            Ok(())
        }
    "#;
    
    // Write the verification code to a temporary file
    let verify_path = temp_path.join("verify.rs");
    fs::write(&verify_path, verify_code).expect("Failed to write verification code");
    
    // Compile and run the verification code
    let build_output = Command::new("rustc")
        .args(&[
            "-L", "target/debug/deps",
            "--extern", "mirror_sdk=target/debug/libmirror_sdk.rlib",
            verify_path.to_str().unwrap(),
            "-o", temp_path.join("verify").to_str().unwrap()
        ])
        .output()
        .expect("Failed to compile verification code");
    
    assert!(build_output.status.success(), "Failed to compile verification code");
    
    let verify_output = Command::new(temp_path.join("verify"))
        .arg(config_path.to_str().unwrap())
        .output()
        .expect("Failed to run verification code");
    
    assert!(verify_output.status.success(), "Verification failed");
    
    // Step 4: Use the CLI to list repositories and verify the output
    let list_output = Command::new("cargo")
        .args(&[
            "run", "--bin", "mirror-cli", "--", 
            "list", 
            "--config", config_path.to_str().unwrap()
        ])
        .output()
        .expect("Failed to execute CLI list command");
    
    assert!(list_output.status.success(), "CLI list command failed");
    
    let list_output_str = String::from_utf8_lossy(&list_output.stdout);
    assert!(list_output_str.contains("example/repo"), "Repository path not found in list output");
    assert!(list_output_str.contains("git@github.com:example/repo.git"), "Repository origin not found in list output");
    
    // Step 5: Use the CLI to update the repository
    let update_output = Command::new("cargo")
        .args(&[
            "run", "--bin", "mirror-cli", "--", 
            "update", 
            "--config", config_path.to_str().unwrap(),
            "--path", "example/repo",
            "--branch", "develop",
            "--add-tags", "updated"
        ])
        .output()
        .expect("Failed to execute CLI update command");
    
    assert!(update_output.status.success(), "CLI update command failed");
    
    // Step 6: Use the SDK to verify the update
    let verify_update_code = r#"
        use mirror_sdk::{MirrorSdk, MirrorError};
        use std::path::PathBuf;
        
        fn main() -> Result<(), MirrorError> {
            let sdk = MirrorSdk::new();
            let config_path = PathBuf::from(std::env::args().nth(1).unwrap());
            let config = sdk.load_config(config_path)?;
            
            // Verify the repository details after update
            let repo = &config.repositories[0];
            assert_eq!(repo.branch, "develop", "Branch was not updated");
            assert!(repo.tags.contains(&"updated".to_string()), "Tag 'updated' was not added");
            
            println!("Update verification successful");
            Ok(())
        }
    "#;
    
    // Write the update verification code to a temporary file
    let verify_update_path = temp_path.join("verify_update.rs");
    fs::write(&verify_update_path, verify_update_code).expect("Failed to write update verification code");
    
    // Compile and run the update verification code
    let build_update_output = Command::new("rustc")
        .args(&[
            "-L", "target/debug/deps",
            "--extern", "mirror_sdk=target/debug/libmirror_sdk.rlib",
            verify_update_path.to_str().unwrap(),
            "-o", temp_path.join("verify_update").to_str().unwrap()
        ])
        .output()
        .expect("Failed to compile update verification code");
    
    assert!(build_update_output.status.success(), "Failed to compile update verification code");
    
    let verify_update_output = Command::new(temp_path.join("verify_update"))
        .arg(config_path.to_str().unwrap())
        .output()
        .expect("Failed to run update verification code");
    
    assert!(verify_update_output.status.success(), "Update verification failed");
    
    // Step 7: Use the CLI to remove the repository
    let remove_output = Command::new("cargo")
        .args(&[
            "run", "--bin", "mirror-cli", "--", 
            "remove", 
            "--config", config_path.to_str().unwrap(),
            "--path", "example/repo"
        ])
        .output()
        .expect("Failed to execute CLI remove command");
    
    assert!(remove_output.status.success(), "CLI remove command failed");
    
    // Step 8: Use the SDK to verify the repository was removed
    let verify_remove_code = r#"
        use mirror_sdk::{MirrorSdk, MirrorError};
        use std::path::PathBuf;
        
        fn main() -> Result<(), MirrorError> {
            let sdk = MirrorSdk::new();
            let config_path = PathBuf::from(std::env::args().nth(1).unwrap());
            let config = sdk.load_config(config_path)?;
            
            // Verify the repository was removed
            assert_eq!(config.repositories.len(), 0, "Repository was not removed");
            
            println!("Remove verification successful");
            Ok(())
        }
    "#;
    
    // Write the remove verification code to a temporary file
    let verify_remove_path = temp_path.join("verify_remove.rs");
    fs::write(&verify_remove_path, verify_remove_code).expect("Failed to write remove verification code");
    
    // Compile and run the remove verification code
    let build_remove_output = Command::new("rustc")
        .args(&[
            "-L", "target/debug/deps",
            "--extern", "mirror_sdk=target/debug/libmirror_sdk.rlib",
            verify_remove_path.to_str().unwrap(),
            "-o", temp_path.join("verify_remove").to_str().unwrap()
        ])
        .output()
        .expect("Failed to compile remove verification code");
    
    assert!(build_remove_output.status.success(), "Failed to compile remove verification code");
    
    let verify_remove_output = Command::new(temp_path.join("verify_remove"))
        .arg(config_path.to_str().unwrap())
        .output()
        .expect("Failed to run remove verification code");
    
    assert!(verify_remove_output.status.success(), "Remove verification failed");
}