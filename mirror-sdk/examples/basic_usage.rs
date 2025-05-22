//! Basic usage example for mirror-sdk
//!
//! This example demonstrates how to:
//! 1. Create a new mirror.toml file
//! 2. Add repositories to it
//! 3. Save it to disk
//! 4. Load it back
//! 5. Modify repositories
//! 6. Save the changes

use mirror_sdk::{MirrorConfig, Repository, Error};
use std::path::Path;

fn main() -> Result<(), Error> {
    // Path for our example
    let config_path = Path::new("example_mirror.toml");
    
    // Create a new mirror configuration
    println!("Creating a new mirror configuration...");
    let mut config = MirrorConfig::new();
    
    // Add some repositories
    println!("Adding repositories...");
    
    // Repository with auto-generated ID
    let repo1 = Repository::new(
        "git@github.com:mirrorboards/example-repo-1.git",
        "example/repo1",
    )?;
    config.add_repository(repo1)?;
    
    // Repository with custom ID and branch
    let repo2 = Repository::new(
        "git@github.com:mirrorboards/example-repo-2.git",
        "example/repo2",
    )?
    .with_id("custom-id")
    .with_branch("develop");
    config.add_repository(repo2)?;
    
    // Repository with tags
    let repo3 = Repository::new(
        "git@github.com:mirrorboards/example-repo-3.git",
        "example/repo3",
    )?
    .with_tags(vec!["monorepo", "important"]);
    config.add_repository(repo3)?;
    
    // Save the configuration to disk
    println!("Saving configuration to {}...", config_path.display());
    config.save_to(config_path)?;
    
    // Load the configuration back
    println!("Loading configuration from {}...", config_path.display());
    let mut loaded_config = MirrorConfig::load_from(config_path)?;
    
    // Print all repositories
    println!("\nRepositories:");
    for (i, repo) in loaded_config.get_repositories().iter().enumerate() {
        println!("{}. ID: {:?}, Origin: {}, Path: {}", 
            i + 1, 
            repo.id, 
            repo.origin, 
            repo.path
        );
    }
    
    // Get repositories by tag
    println!("\nRepositories with 'monorepo' tag:");
    for repo in loaded_config.get_repositories_by_tag("monorepo") {
        println!("- ID: {:?}, Path: {}", repo.id, repo.path);
    }
    
    // Modify a repository
    println!("\nModifying repository with ID 'custom-id'...");
    let repo = loaded_config.get_repository_mut("custom-id")?;
    repo.path = "new/path/for/repo2".to_string();
    
    // Save the changes
    println!("Saving changes...");
    loaded_config.save_to(config_path)?;
    
    println!("\nDone! Check {} for the result.", config_path.display());
    
    // Clean up (comment this out if you want to inspect the file)
    std::fs::remove_file(config_path)?;
    println!("Cleaned up example file.");
    
    Ok(())
}