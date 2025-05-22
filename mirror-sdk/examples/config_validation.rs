//! Config validation example for the Mirror SDK.

use mirror_sdk::{MirrorSdk, Repository, RepositoryBuilder, MirrorError};
use std::path::Path;

fn main() -> Result<(), MirrorError> {
    // Create a new SDK instance
    let sdk = MirrorSdk::new();
    
    // Create a new configuration
    let mut config = sdk.new_config();
    println!("Created new configuration");
    
    // Add valid repositories
    println!("\nAdding valid repositories...");
    add_valid_repositories(&sdk, &mut config)?;
    
    // Validate the configuration
    match sdk.validate_config(&config) {
        Ok(_) => println!("Configuration is valid"),
        Err(e) => println!("Validation error: {}", e),
    }
    
    // Try to add invalid repositories
    println!("\nTrying to add invalid repositories...");
    try_add_invalid_repositories(&sdk, &mut config);
    
    // Try to create a configuration with conflicts
    println!("\nTrying to create a configuration with conflicts...");
    try_create_config_with_conflicts(&sdk)?;
    
    println!("\nExample completed successfully!");
    Ok(())
}

fn add_valid_repositories(sdk: &MirrorSdk, config: &mut mirror_sdk::MirrorConfig) -> Result<(), MirrorError> {
    // Create valid repositories
    let repo1 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo1.git")
        .branch("main")
        .path("example/repo1")
        .id("repo1-id")
        .build()?;
    
    let repo2 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo2.git")
        .branch("main")
        .path("example/repo2")
        .id("repo2-id")
        .build()?;
    
    // Add repositories to the configuration
    sdk.add_repository(config, repo1)?;
    sdk.add_repository(config, repo2)?;
    
    println!("Added 2 valid repositories");
    
    Ok(())
}

fn try_add_invalid_repositories(sdk: &MirrorSdk, config: &mut mirror_sdk::MirrorConfig) {
    // Try to add a repository with an invalid origin
    let invalid_origin_result = RepositoryBuilder::new()
        .origin("invalid-origin")
        .branch("main")
        .path("example/invalid")
        .build();
    
    match invalid_origin_result {
        Ok(repo) => {
            println!("Created repository with invalid origin (unexpected)");
            let add_result = sdk.add_repository(config, repo);
            match add_result {
                Ok(_) => println!("Added repository with invalid origin (unexpected)"),
                Err(e) => println!("Failed to add repository with invalid origin: {}", e),
            }
        },
        Err(e) => println!("Failed to create repository with invalid origin: {}", e),
    }
    
    // Try to add a repository with an invalid path
    let invalid_path_result = RepositoryBuilder::new()
        .origin("git@github.com:example/invalid.git")
        .branch("main")
        .path("../example/invalid")
        .build();
    
    match invalid_path_result {
        Ok(repo) => {
            println!("Created repository with invalid path (unexpected)");
            let add_result = sdk.add_repository(config, repo);
            match add_result {
                Ok(_) => println!("Added repository with invalid path (unexpected)"),
                Err(e) => println!("Failed to add repository with invalid path: {}", e),
            }
        },
        Err(e) => println!("Failed to create repository with invalid path: {}", e),
    }
    
    // Try to add a repository with a duplicate path
    let duplicate_path_result = RepositoryBuilder::new()
        .origin("git@github.com:example/duplicate.git")
        .branch("main")
        .path("example/repo1") // Same path as repo1
        .build();
    
    match duplicate_path_result {
        Ok(repo) => {
            println!("Created repository with duplicate path");
            let add_result = sdk.add_repository(config, repo);
            match add_result {
                Ok(_) => println!("Added repository with duplicate path (unexpected)"),
                Err(e) => println!("Failed to add repository with duplicate path: {}", e),
            }
        },
        Err(e) => println!("Failed to create repository with duplicate path: {}", e),
    }
    
    // Try to add a repository with a duplicate ID
    let duplicate_id_result = RepositoryBuilder::new()
        .origin("git@github.com:example/duplicate.git")
        .branch("main")
        .path("example/duplicate")
        .id("repo1-id") // Same ID as repo1
        .build();
    
    match duplicate_id_result {
        Ok(repo) => {
            println!("Created repository with duplicate ID");
            let add_result = sdk.add_repository(config, repo);
            match add_result {
                Ok(_) => println!("Added repository with duplicate ID (unexpected)"),
                Err(e) => println!("Failed to add repository with duplicate ID: {}", e),
            }
        },
        Err(e) => println!("Failed to create repository with duplicate ID: {}", e),
    }
}

fn try_create_config_with_conflicts(sdk: &MirrorSdk) -> Result<(), MirrorError> {
    // Create a new configuration
    let mut config = sdk.new_config();
    
    // Add repositories with path conflicts
    let repo1 = RepositoryBuilder::new()
        .origin("git@github.com:example/parent.git")
        .branch("main")
        .path("example/parent")
        .build()?;
    
    let repo2 = RepositoryBuilder::new()
        .origin("git@github.com:example/child.git")
        .branch("main")
        .path("example/parent/child")
        .build()?;
    
    // Add the first repository
    sdk.add_repository(&mut config, repo1)?;
    println!("Added parent repository");
    
    // Try to add the second repository (should fail due to path conflict)
    match sdk.add_repository(&mut config, repo2) {
        Ok(_) => println!("Added child repository (unexpected)"),
        Err(e) => println!("Failed to add child repository due to path conflict: {}", e),
    }
    
    // Create a new configuration
    let mut config = sdk.new_config();
    
    // Add repositories with duplicate IDs
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
    println!("Added repository with ID 'duplicate-id'");
    
    // Try to add the second repository (should fail due to duplicate ID)
    match sdk.add_repository(&mut config, repo2) {
        Ok(_) => println!("Added second repository with duplicate ID (unexpected)"),
        Err(e) => println!("Failed to add second repository due to duplicate ID: {}", e),
    }
    
    Ok(())
}