//! Repository management example for the Mirror SDK.

use mirror_sdk::{MirrorSdk, Repository, RepositoryBuilder, MirrorError};
use std::path::Path;

fn main() -> Result<(), MirrorError> {
    // Create a new SDK instance
    let sdk = MirrorSdk::new();
    
    // Path to the mirror.toml file
    let config_path = Path::new("repo_management.toml");
    
    // Initialize a new configuration file
    let mut config = sdk.init_config(config_path, true)?;
    println!("Initialized new configuration at {}", config_path.display());
    
    // Add multiple repositories
    add_repositories(&sdk, &mut config)?;
    
    // Save the configuration
    sdk.save_config(&config, config_path)?;
    println!("Saved configuration with {} repositories", config.repositories.len());
    
    // Find repositories by tag
    find_repositories_by_tag(&sdk, &config, "frontend")?;
    find_repositories_by_tag(&sdk, &config, "backend")?;
    
    // Update repositories
    update_repositories(&sdk, &mut config)?;
    
    // Save the updated configuration
    sdk.save_config(&config, "repo_management_updated.toml")?;
    println!("Saved updated configuration to repo_management_updated.toml");
    
    // Remove repositories
    remove_repositories(&sdk, &mut config)?;
    
    // Save the final configuration
    sdk.save_config(&config, "repo_management_final.toml")?;
    println!("Saved final configuration to repo_management_final.toml");
    
    println!("Example completed successfully!");
    Ok(())
}

fn add_repositories(sdk: &MirrorSdk, config: &mut mirror_sdk::MirrorConfig) -> Result<(), MirrorError> {
    println!("Adding repositories...");
    
    // Frontend repositories
    let frontend_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/frontend.git")
        .branch("main")
        .path("projects/frontend")
        .id("frontend-main")
        .tag("frontend")
        .build()?;
    
    let ui_components_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/ui-components.git")
        .branch("main")
        .path("projects/ui-components")
        .tag("frontend")
        .tag("components")
        .build()?;
    
    // Backend repositories
    let backend_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/backend.git")
        .branch("main")
        .path("projects/backend")
        .id("backend-main")
        .tag("backend")
        .build()?;
    
    let api_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/api.git")
        .branch("main")
        .path("projects/api")
        .tag("backend")
        .tag("api")
        .build()?;
    
    // Add repositories to the configuration
    sdk.add_repository(config, frontend_repo)?;
    sdk.add_repository(config, ui_components_repo)?;
    sdk.add_repository(config, backend_repo)?;
    sdk.add_repository(config, api_repo)?;
    
    println!("Added 4 repositories to the configuration");
    
    Ok(())
}

fn find_repositories_by_tag(sdk: &MirrorSdk, config: &mirror_sdk::MirrorConfig, tag: &str) -> Result<(), MirrorError> {
    let repos = sdk.find_repositories_by_tag(config, tag);
    println!("Found {} repositories with tag '{}':", repos.len(), tag);
    
    for (i, repo) in repos.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, repo.path, repo.origin);
    }
    
    Ok(())
}

fn update_repositories(sdk: &MirrorSdk, config: &mut mirror_sdk::MirrorConfig) -> Result<(), MirrorError> {
    println!("Updating repositories...");
    
    // Update frontend repository to use a different branch
    let updated_frontend = RepositoryBuilder::new()
        .origin("git@github.com:example/frontend.git")
        .branch("develop")
        .path("projects/frontend")
        .id("frontend-main")
        .tag("frontend")
        .tag("development")
        .build()?;
    
    sdk.update_repository(config, updated_frontend)?;
    println!("Updated frontend repository to use 'develop' branch");
    
    // Update backend repository with branch lock
    let updated_backend = RepositoryBuilder::new()
        .origin("git@github.com:example/backend.git")
        .branch("main")
        .branch_lock(true)
        .path("projects/backend")
        .id("backend-main")
        .tag("backend")
        .tag("locked")
        .build()?;
    
    sdk.update_repository(config, updated_backend)?;
    println!("Updated backend repository with branch lock");
    
    Ok(())
}

fn remove_repositories(sdk: &MirrorSdk, config: &mut mirror_sdk::MirrorConfig) -> Result<(), MirrorError> {
    println!("Removing repositories...");
    
    // Remove repository by path
    sdk.remove_repository_by_path(config, "projects/ui-components")?;
    println!("Removed ui-components repository by path");
    
    // Remove repository by ID
    sdk.remove_repository_by_id(config, "backend-main")?;
    println!("Removed backend repository by ID");
    
    Ok(())
}