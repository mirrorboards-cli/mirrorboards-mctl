//! Basic usage example for the Mirror SDK.

use mirror_sdk::{MirrorSdk, Repository, RepositoryBuilder, MirrorError};
use std::path::Path;

fn main() -> Result<(), MirrorError> {
    // Create a new SDK instance
    let sdk = MirrorSdk::new();
    
    // Path to the mirror.toml file
    let config_path = Path::new("example_mirror.toml");
    
    // Initialize a new configuration file
    let mut config = sdk.init_config(config_path, true)?;
    println!("Initialized new configuration at {}", config_path.display());
    
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
    println!("Added 2 repositories to the configuration");
    
    // Save the configuration
    sdk.save_config(&config, config_path)?;
    println!("Saved configuration to {}", config_path.display());
    
    // Load the configuration
    let loaded_config = sdk.load_config(config_path)?;
    println!("Loaded configuration from {}", config_path.display());
    
    // Find repositories
    let repos_by_tag = sdk.find_repositories_by_tag(&loaded_config, "example");
    println!("Found {} repositories with tag 'example'", repos_by_tag.len());
    
    let repo_by_id = sdk.find_repository_by_id(&loaded_config, "repo2-id");
    if let Some(repo) = repo_by_id {
        println!("Found repository with ID 'repo2-id': {}", repo.origin);
    }
    
    let repo_by_path = sdk.find_repository_by_path(&loaded_config, "example/repo1");
    if let Some(repo) = repo_by_path {
        println!("Found repository with path 'example/repo1': {}", repo.origin);
    }
    
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
    println!("Updated repository 'example/repo1'");
    
    // Save the updated configuration
    sdk.save_config(&updated_config, "example_mirror_updated.toml")?;
    println!("Saved updated configuration to example_mirror_updated.toml");
    
    // Remove a repository
    let mut final_config = updated_config.clone();
    sdk.remove_repository_by_path(&mut final_config, "example/repo2")?;
    println!("Removed repository 'example/repo2'");
    
    // Save the final configuration
    sdk.save_config(&final_config, "example_mirror_final.toml")?;
    println!("Saved final configuration to example_mirror_final.toml");
    
    println!("Example completed successfully!");
    Ok(())
}