//! Integration tests for mirror-sdk
//!
//! These tests verify that the library works correctly as a whole.

use mirror_sdk::{MirrorConfig, Repository, Error};
use tempfile::tempdir;

#[test]
fn test_create_load_modify_save() -> Result<(), Error> {
    // Create a temporary directory for our test
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("mirror.toml");
    
    // Create a new configuration
    let mut config = MirrorConfig::new();
    
    // Add some repositories
    let repo1 = Repository::new(
        "git@github.com:mirrorboards/repo1.git",
        "path/to/repo1",
    )?
    .with_id("repo1-id");
    
    let repo2 = Repository::new(
        "git@github.com:mirrorboards/repo2.git",
        "path/to/repo2",
    )?
    .with_branch("develop")
    .with_tags(vec!["test", "example"]);
    
    // Add repositories to config
    config.add_repository(repo1)?;
    config.add_repository(repo2)?;
    
    // Save the configuration
    config.save_to(&config_path)?;
    
    // Load the configuration back
    let mut loaded_config = MirrorConfig::load_from(&config_path)?;
    
    // Verify repositories were loaded correctly
    assert_eq!(loaded_config.get_repositories().len(), 2);
    
    // Get repository by ID
    let repo = loaded_config.get_repository("repo1-id")?;
    assert_eq!(repo.origin, "git@github.com:mirrorboards/repo1.git");
    assert_eq!(repo.path, "path/to/repo1");
    
    // Modify a repository
    let repo = loaded_config.get_repository_mut("repo1-id")?;
    repo.path = "new/path/to/repo1".to_string();
    
    // Save the changes
    loaded_config.save_to(&config_path)?;
    
    // Load again to verify changes were saved
    let reloaded_config = MirrorConfig::load_from(&config_path)?;
    let repo = reloaded_config.get_repository("repo1-id")?;
    assert_eq!(repo.path, "new/path/to/repo1");
    
    // Test get_repositories_by_tag
    let tagged_repos = reloaded_config.get_repositories_by_tag("test");
    assert_eq!(tagged_repos.len(), 1);
    assert!(tagged_repos[0].tags.as_ref().unwrap().contains(&"test".to_string()));
    
    Ok(())
}

#[test]
fn test_duplicate_id_error() {
    // Create a new configuration
    let mut config = MirrorConfig::new();
    
    // Add a repository with ID "test-id"
    let repo1 = Repository::new(
        "git@github.com:mirrorboards/repo1.git",
        "path/to/repo1",
    ).unwrap()
    .with_id("test-id");
    
    config.add_repository(repo1).unwrap();
    
    // Try to add another repository with the same ID
    let repo2 = Repository::new(
        "git@github.com:mirrorboards/repo2.git",
        "path/to/repo2",
    ).unwrap()
    .with_id("test-id");
    
    let result = config.add_repository(repo2);
    
    // Verify that we get a DuplicateId error
    match result {
        Err(Error::DuplicateId(id)) => {
            assert_eq!(id, "test-id");
        },
        _ => panic!("Expected DuplicateId error"),
    }
}

#[test]
fn test_repository_not_found_error() {
    // Create a new configuration
    let config = MirrorConfig::new();
    
    // Try to get a repository that doesn't exist
    let result = config.get_repository("non-existent-id");
    
    // Verify that we get a RepositoryNotFound error
    match result {
        Err(Error::RepositoryNotFound(id)) => {
            assert_eq!(id, "non-existent-id");
        },
        _ => panic!("Expected RepositoryNotFound error"),
    }
}

#[test]
fn test_path_collision_allowed() -> Result<(), Error> {
    // Create a new configuration
    let mut config = MirrorConfig::new();
    
    // Add a repository with path "same/path"
    let repo1 = Repository::new(
        "git@github.com:mirrorboards/repo1.git",
        "same/path",
    )?;
    
    config.add_repository(repo1)?;
    
    // Add another repository with the same path
    let repo2 = Repository::new(
        "git@github.com:mirrorboards/repo2.git",
        "same/path",
    )?;
    
    // This should succeed because path collision is allowed
    config.add_repository(repo2)?;
    
    // Verify that both repositories were added
    assert_eq!(config.get_repositories().len(), 2);
    
    Ok(())
}