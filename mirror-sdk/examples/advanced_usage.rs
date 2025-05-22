//! Advanced usage example for mirror-sdk
//!
//! This example demonstrates more advanced usage:
//! 1. Initializing a mirror.toml file
//! 2. Adding repositories from a list of git URLs
//! 3. Organizing repositories with tags
//! 4. Finding repositories by path
//! 5. Error handling

use mirror_sdk::{MirrorConfig, Repository, Error};
use std::path::Path;
use std::collections::HashMap;

// Simulated input data - in a real application, this might come from CLI args or a config file
struct RepoInput {
    origin: String,
    path: String,
    tags: Vec<String>,
    branch: Option<String>,
}

fn main() -> Result<(), Error> {
    // Path for our example
    let config_path = Path::new("advanced_mirror.toml");
    
    // Initialize a new mirror.toml file
    println!("Initializing a new mirror.toml file at {}...", config_path.display());
    let mut config = match MirrorConfig::init_at(config_path) {
        Ok(config) => config,
        Err(Error::Other(msg)) if msg.contains("already exists") => {
            println!("File already exists, loading it instead...");
            MirrorConfig::load_from(config_path)?
        },
        Err(e) => return Err(e),
    };
    
    // Simulated input data - repositories to add
    let repos_to_add = vec![
        RepoInput {
            origin: "git@github.com:mirrorboards/frontend.git".to_string(),
            path: "projects/frontend".to_string(),
            tags: vec!["frontend".to_string(), "web".to_string()],
            branch: Some("main".to_string()),
        },
        RepoInput {
            origin: "git@github.com:mirrorboards/backend.git".to_string(),
            path: "projects/backend".to_string(),
            tags: vec!["backend".to_string(), "api".to_string()],
            branch: Some("develop".to_string()),
        },
        RepoInput {
            origin: "git@github.com:mirrorboards/docs.git".to_string(),
            path: "docs".to_string(),
            tags: vec!["documentation".to_string()],
            branch: None,
        },
        RepoInput {
            origin: "git@github.com:mirrorboards/shared.git".to_string(),
            path: "shared".to_string(),
            tags: vec!["shared".to_string(), "frontend".to_string(), "backend".to_string()],
            branch: None,
        },
    ];
    
    // Add repositories
    println!("Adding repositories...");
    for repo_input in repos_to_add {
        // Check if a repository with this path already exists
        let path_exists = config.get_repositories().iter()
            .any(|r| r.path == repo_input.path);
        
        if path_exists {
            println!("Repository with path '{}' already exists, skipping...", repo_input.path);
            continue;
        }
        
        // Create the repository
        let mut repo = Repository::new(repo_input.origin.clone(), repo_input.path.clone())?;
        
        // Add tags
        if !repo_input.tags.is_empty() {
            repo = repo.with_tags(repo_input.tags.clone());
        }
        
        // Set branch if specified
        if let Some(ref branch) = repo_input.branch {
            repo = repo.with_branch(branch.clone());
        }
        
        // Add to config
        match config.add_repository(repo) {
            Ok(_) => println!("Added repository successfully"),
            Err(Error::DuplicateId(id)) => {
                println!("Repository with ID '{}' already exists, generating a new ID...", id);
                // Create a new repository with a different ID
                let mut repo = Repository::new(repo_input.origin, repo_input.path)?;
                if let Some(branch) = repo_input.branch {
                    repo = repo.with_branch(branch);
                }
                if !repo_input.tags.is_empty() {
                    repo = repo.with_tags(repo_input.tags);
                }
                config.add_repository(repo)?;
            },
            Err(e) => return Err(e),
        }
    }
    
    // Save the configuration
    println!("Saving configuration...");
    config.save()?;
    
    // Analyze the repositories by tag
    println!("\nRepository analysis by tag:");
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    
    for repo in config.get_repositories() {
        if let Some(tags) = &repo.tags {
            for tag in tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
    }
    
    for (tag, count) in tag_counts.iter() {
        println!("- {}: {} repositories", tag, count);
        
        // List repositories with this tag
        for repo in config.get_repositories_by_tag(tag) {
            println!("  * {}: {}", repo.id.as_ref().unwrap_or(&"<no-id>".to_string()), repo.path);
        }
    }
    
    // Find repositories by path prefix
    let prefix = "projects/";
    println!("\nRepositories in '{}' directory:", prefix);
    for repo in config.get_repositories() {
        if repo.path.starts_with(prefix) {
            println!("- {}: {}", repo.id.as_ref().unwrap_or(&"<no-id>".to_string()), repo.path);
        }
    }
    
    println!("\nDone! Check {} for the result.", config_path.display());
    
    // Clean up (comment this out if you want to inspect the file)
    std::fs::remove_file(config_path)?;
    println!("Cleaned up example file.");
    
    Ok(())
}