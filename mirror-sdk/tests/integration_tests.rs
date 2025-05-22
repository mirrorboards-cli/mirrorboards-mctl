//! Integration tests for the Mirror SDK.

use mirror_sdk::{MirrorSdk, Repository, RepositoryBuilder, MirrorError};
use std::path::Path;
use tempfile::tempdir;

#[test]
fn test_basic_workflow() -> Result<(), MirrorError> {
    // Create a temporary directory for the test
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("mirror.toml");
    
    // Create a new SDK instance
    let sdk = MirrorSdk::new();
    
    // Initialize a new configuration file
    let mut config = sdk.init_config(&config_path, false)?;
    assert!(config.repositories.is_empty());
    
    // Create and add repositories
    let repo1 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo1.git")
        .branch("main")
        .path("example/repo1")
        .tag("example")
        .build()?;
    
    let repo2 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo2.git")
        .branch("develop")
        .path("example/repo2")
        .id("repo2-id")
        .branch_lock(true)
        .tag("example")
        .tag("locked")
        .build()?;
    
    // Add repositories to the configuration
    sdk.add_repository(&mut config, repo1)?;
    sdk.add_repository(&mut config, repo2)?;
    assert_eq!(config.repositories.len(), 2);
    
    // Save the configuration
    sdk.save_config(&config, &config_path)?;
    assert!(config_path.exists());
    
    // Load the configuration
    let loaded_config = sdk.load_config(&config_path)?;
    assert_eq!(loaded_config.repositories.len(), 2);
    
    // Find repositories
    let repos_by_tag = sdk.find_repositories_by_tag(&loaded_config, "example");
    assert_eq!(repos_by_tag.len(), 2);
    
    let repo_by_id = sdk.find_repository_by_id(&loaded_config, "repo2-id");
    assert!(repo_by_id.is_some());
    assert_eq!(repo_by_id.unwrap().path, "example/repo2");
    
    let repo_by_path = sdk.find_repository_by_path(&loaded_config, "example/repo1");
    assert!(repo_by_path.is_some());
    assert_eq!(repo_by_path.unwrap().origin, "git@github.com:example/repo1.git");
    
    // Update a repository
    let updated_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/repo1-updated.git")
        .branch("main")
        .path("example/repo1")
        .tag("example")
        .tag("updated")
        .build()?;
    
    let mut updated_config = loaded_config.clone();
    sdk.update_repository(&mut updated_config, updated_repo)?;
    
    let updated_repo_path = sdk.find_repository_by_path(&updated_config, "example/repo1");
    assert!(updated_repo_path.is_some());
    assert_eq!(updated_repo_path.unwrap().origin, "git@github.com:example/repo1-updated.git");
    
    // Remove a repository
    let mut final_config = updated_config.clone();
    sdk.remove_repository_by_path(&mut final_config, "example/repo2")?;
    assert_eq!(final_config.repositories.len(), 1);
    
    let removed_repo = sdk.find_repository_by_path(&final_config, "example/repo2");
    assert!(removed_repo.is_none());
    
    Ok(())
}

#[test]
fn test_validation() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Create a configuration with duplicate paths
    let mut config = sdk.new_config();
    
    let repo1 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo1.git")
        .branch("main")
        .path("example/repo")
        .build()?;
    
    let repo2 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo2.git")
        .branch("main")
        .path("example/repo")
        .build()?;
    
    // Add the first repository
    sdk.add_repository(&mut config, repo1)?;
    
    // Adding the second repository should fail due to path conflict
    let result = sdk.add_repository(&mut config, repo2);
    assert!(result.is_err());
    
    // Create a configuration with duplicate IDs
    let mut config = sdk.new_config();
    
    let repo1 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo1.git")
        .branch("main")
        .path("example/repo1")
        .id("duplicate-id")
        .build()?;
    
    let repo2 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo2.git")
        .branch("main")
        .path("example/repo2")
        .id("duplicate-id")
        .build()?;
    
    // Add the first repository
    sdk.add_repository(&mut config, repo1)?;
    
    // Adding the second repository should fail due to ID conflict
    let result = sdk.add_repository(&mut config, repo2);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_config_path() -> Result<(), MirrorError> {
    // Create a new SDK instance with default settings
    let sdk = MirrorSdk::new();
    
    // Get the config path (should be in the current directory)
    let path = sdk.get_config_path()?;
    assert!(path.ends_with("mirror.toml"));
    
    // Create a new SDK instance with custom settings
    let settings = mirror_sdk::ConfigSettings::default()
        .with_default_config_path(Path::new("/custom/path/mirror.toml"));
    
    let sdk = MirrorSdk::with_settings(settings);
    
    // Get the config path (should be the custom path)
    let path = sdk.get_config_path()?;
    assert_eq!(path, Path::new("/custom/path/mirror.toml"));
    
    Ok(())
}