# Common Workflows

This document provides examples of common workflows using the Mirror SDK and CLI.

## Setting Up a New Project

This workflow demonstrates how to set up a new project with multiple repositories.

### Using the CLI

```bash
# Initialize a new mirror.toml file
mirror-cli init

# Add the main repository
mirror-cli add --origin "git@github.com:example/main.git" --path "main" --tags "core"

# Add frontend repositories
mirror-cli add --origin "git@github.com:example/frontend.git" --path "frontend" --tags "frontend"
mirror-cli add --origin "git@github.com:example/ui-components.git" --path "ui-components" --tags "frontend,components"

# Add backend repositories
mirror-cli add --origin "git@github.com:example/api.git" --path "backend/api" --tags "backend,api"
mirror-cli add --origin "git@github.com:example/database.git" --path "backend/database" --tags "backend,database"

# List all repositories
mirror-cli list
```

### Using the SDK

```rust
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};

fn setup_new_project() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Create a new configuration
    let mut config = sdk.new_config();
    
    // Add the main repository
    let main_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/main.git")
        .branch("main")
        .path("main")
        .tag("core")
        .build()?;
    sdk.add_repository(&mut config, main_repo)?;
    
    // Add frontend repositories
    let frontend_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/frontend.git")
        .branch("main")
        .path("frontend")
        .tag("frontend")
        .build()?;
    sdk.add_repository(&mut config, frontend_repo)?;
    
    let ui_components_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/ui-components.git")
        .branch("main")
        .path("ui-components")
        .tag("frontend")
        .tag("components")
        .build()?;
    sdk.add_repository(&mut config, ui_components_repo)?;
    
    // Add backend repositories
    let api_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/api.git")
        .branch("main")
        .path("backend/api")
        .tag("backend")
        .tag("api")
        .build()?;
    sdk.add_repository(&mut config, api_repo)?;
    
    let database_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/database.git")
        .branch("main")
        .path("backend/database")
        .tag("backend")
        .tag("database")
        .build()?;
    sdk.add_repository(&mut config, database_repo)?;
    
    // Save the configuration
    sdk.save_config(&config, "mirror.toml")?;
    
    Ok(())
}
```

## Managing Repository Tags

This workflow demonstrates how to manage repository tags for better organization.

### Using the CLI

```bash
# List all repositories
mirror-cli list

# List repositories with a specific tag
mirror-cli list --tag "frontend"

# Add tags to a repository
mirror-cli update --path "main" --add-tags "important,production"

# Remove tags from a repository
mirror-cli update --path "main" --remove-tags "production"
```

### Using the SDK

```rust
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};

fn manage_tags() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load the configuration
    let mut config = sdk.load_config("mirror.toml")?;
    
    // Find repositories with a specific tag
    let frontend_repos = sdk.find_repositories_by_tag(&config, "frontend");
    println!("Frontend repositories:");
    for repo in frontend_repos {
        println!("- {}", repo.path);
    }
    
    // Find a repository by path
    if let Some(repo) = sdk.find_repository_by_path(&config, "main") {
        // Create an updated repository with new tags
        let mut builder = RepositoryBuilder::new()
            .origin(&repo.origin)
            .branch(&repo.branch)
            .path(&repo.path)
            .branch_lock(repo.branch_lock);
        
        // Add existing ID if present
        if let Some(id) = &repo.id {
            builder = builder.id(id);
        }
        
        // Add existing tags
        for tag in &repo.tags {
            builder = builder.tag(tag);
        }
        
        // Add new tags
        builder = builder.tag("important").tag("production");
        
        let updated_repo = builder.build()?;
        
        // Update the repository
        sdk.remove_repository_by_path(&mut config, "main")?;
        sdk.add_repository(&mut config, updated_repo)?;
        
        // Save the configuration
        sdk.save_config(&config, "mirror.toml")?;
    }
    
    Ok(())
}
```

## Migrating Repositories

This workflow demonstrates how to migrate repositories from one location to another.

### Using the CLI

```bash
# Update the origin of a repository
mirror-cli update --path "frontend" --origin "git@github.com:new-org/frontend.git"

# Update the path of a repository
mirror-cli update --path "backend/api" --new-path "api"
```

### Using the SDK

```rust
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};

fn migrate_repositories() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load the configuration
    let mut config = sdk.load_config("mirror.toml")?;
    
    // Find a repository by path
    if let Some(repo) = sdk.find_repository_by_path(&config, "frontend") {
        // Create an updated repository with new origin
        let mut builder = RepositoryBuilder::new()
            .origin("git@github.com:new-org/frontend.git")
            .branch(&repo.branch)
            .path(&repo.path)
            .branch_lock(repo.branch_lock);
        
        // Add existing ID if present
        if let Some(id) = &repo.id {
            builder = builder.id(id);
        }
        
        // Add existing tags
        for tag in &repo.tags {
            builder = builder.tag(tag);
        }
        
        let updated_repo = builder.build()?;
        
        // Update the repository
        sdk.remove_repository_by_path(&mut config, "frontend")?;
        sdk.add_repository(&mut config, updated_repo)?;
    }
    
    // Find another repository by path
    if let Some(repo) = sdk.find_repository_by_path(&config, "backend/api") {
        // Create an updated repository with new path
        let mut builder = RepositoryBuilder::new()
            .origin(&repo.origin)
            .branch(&repo.branch)
            .path("api")
            .branch_lock(repo.branch_lock);
        
        // Add existing ID if present
        if let Some(id) = &repo.id {
            builder = builder.id(id);
        }
        
        // Add existing tags
        for tag in &repo.tags {
            builder = builder.tag(tag);
        }
        
        let updated_repo = builder.build()?;
        
        // Update the repository
        sdk.remove_repository_by_path(&mut config, "backend/api")?;
        sdk.add_repository(&mut config, updated_repo)?;
    }
    
    // Save the configuration
    sdk.save_config(&config, "mirror.toml")?;
    
    Ok(())
}
```

## Working with Multiple Configuration Files

This workflow demonstrates how to work with multiple mirror.toml files for different environments or projects.

### Using the CLI

```bash
# Create a development configuration
mirror-cli --config dev-mirror.toml init

# Add repositories to the development configuration
mirror-cli --config dev-mirror.toml add --origin "git@github.com:example/repo.git" --path "repo" --branch "develop"

# Create a production configuration
mirror-cli --config prod-mirror.toml init

# Add repositories to the production configuration
mirror-cli --config prod-mirror.toml add --origin "git@github.com:example/repo.git" --path "repo" --branch "main" --branch-lock

# List repositories in each configuration
mirror-cli --config dev-mirror.toml list
mirror-cli --config prod-mirror.toml list
```

### Using the SDK

```rust
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};
use std::path::Path;

fn work_with_multiple_configs() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Create a development configuration
    let mut dev_config = sdk.new_config();
    
    // Add a repository to the development configuration
    let dev_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/repo.git")
        .branch("develop")
        .path("repo")
        .build()?;
    sdk.add_repository(&mut dev_config, dev_repo)?;
    
    // Save the development configuration
    sdk.save_config(&dev_config, "dev-mirror.toml")?;
    
    // Create a production configuration
    let mut prod_config = sdk.new_config();
    
    // Add a repository to the production configuration
    let prod_repo = RepositoryBuilder::new()
        .origin("git@github.com:example/repo.git")
        .branch("main")
        .path("repo")
        .branch_lock(true)
        .build()?;
    sdk.add_repository(&mut prod_config, prod_repo)?;
    
    // Save the production configuration
    sdk.save_config(&prod_config, "prod-mirror.toml")?;
    
    Ok(())
}
```

## Validating Configurations

This workflow demonstrates how to validate mirror.toml configurations to ensure they are correct.

### Using the CLI

```bash
# Validate the default configuration
mirror-cli validate

# Validate a specific configuration
mirror-cli --config custom-mirror.toml validate
```

### Using the SDK

```rust
use mirror_sdk::{MirrorSdk, MirrorError};

fn validate_configs() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load and validate the default configuration
    let default_config = sdk.load_config("mirror.toml")?;
    match sdk.validate_config(&default_config) {
        Ok(_) => println!("Default configuration is valid"),
        Err(err) => println!("Default configuration is invalid: {}", err),
    }
    
    // Load and validate a custom configuration
    let custom_config = sdk.load_config("custom-mirror.toml")?;
    match sdk.validate_config(&custom_config) {
        Ok(_) => println!("Custom configuration is valid"),
        Err(err) => println!("Custom configuration is invalid: {}", err),
    }
    
    Ok(())
}
```

## Programmatically Cloning Repositories

This workflow demonstrates how to use the SDK to programmatically clone repositories defined in a mirror.toml file.

```rust
use mirror_sdk::{MirrorSdk, MirrorError};
use std::process::Command;

fn clone_repositories() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load the configuration
    let config = sdk.load_config("mirror.toml")?;
    
    // Clone each repository
    for repo in &config.repositories {
        println!("Cloning {} to {}", repo.origin, repo.path);
        
        // Create the directory if it doesn't exist
        std::fs::create_dir_all(&repo.path)?;
        
        // Clone the repository
        let output = Command::new("git")
            .args(&["clone", "--branch", &repo.branch, &repo.origin, &repo.path])
            .output()
            .expect("Failed to execute git clone command");
        
        if output.status.success() {
            println!("Successfully cloned {}", repo.path);
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            println!("Failed to clone {}: {}", repo.path, error);
        }
    }
    
    Ok(())
}
```

## Creating a Custom Mirror CLI Tool

This example demonstrates how to create a custom CLI tool that extends the functionality of the mirror-cli.

```rust
use mirror_sdk::{MirrorSdk, MirrorError};
use std::path::PathBuf;
use std::process::Command;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "Custom Mirror CLI Tool")]
struct Cli {
    /// Path to the mirror.toml file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Clone all repositories defined in the mirror.toml file
    CloneAll,
    
    /// Pull updates for all repositories defined in the mirror.toml file
    PullAll,
    
    /// Clone or pull repositories with a specific tag
    Sync {
        /// Filter repositories by tag
        #[arg(short, long)]
        tag: String,
    },
}

fn main() -> Result<(), MirrorError> {
    let cli = Cli::parse();
    let sdk = MirrorSdk::new();
    
    // Get config path
    let config_path = match cli.config {
        Some(path) => path,
        None => sdk.get_config_path()?,
    };
    
    // Load the configuration
    let config = sdk.load_config(&config_path)?;
    
    match cli.command {
        Commands::CloneAll => {
            for repo in &config.repositories {
                if !PathBuf::from(&repo.path).exists() {
                    println!("Cloning {} to {}", repo.origin, repo.path);
                    std::fs::create_dir_all(&repo.path)?;
                    
                    let output = Command::new("git")
                        .args(&["clone", "--branch", &repo.branch, &repo.origin, &repo.path])
                        .output()
                        .expect("Failed to execute git clone command");
                    
                    if output.status.success() {
                        println!("Successfully cloned {}", repo.path);
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        println!("Failed to clone {}: {}", repo.path, error);
                    }
                } else {
                    println!("Repository {} already exists, skipping", repo.path);
                }
            }
        },
        Commands::PullAll => {
            for repo in &config.repositories {
                let path = PathBuf::from(&repo.path);
                if path.exists() {
                    println!("Pulling updates for {}", repo.path);
                    
                    let output = Command::new("git")
                        .current_dir(&path)
                        .args(&["pull", "origin", &repo.branch])
                        .output()
                        .expect("Failed to execute git pull command");
                    
                    if output.status.success() {
                        println!("Successfully pulled updates for {}", repo.path);
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        println!("Failed to pull updates for {}: {}", repo.path, error);
                    }
                } else {
                    println!("Repository {} does not exist, skipping", repo.path);
                }
            }
        },
        Commands::Sync { tag } => {
            let repos = sdk.find_repositories_by_tag(&config, &tag);
            
            for repo in repos {
                let path = PathBuf::from(&repo.path);
                if !path.exists() {
                    println!("Cloning {} to {}", repo.origin, repo.path);
                    std::fs::create_dir_all(&repo.path)?;
                    
                    let output = Command::new("git")
                        .args(&["clone", "--branch", &repo.branch, &repo.origin, &repo.path])
                        .output()
                        .expect("Failed to execute git clone command");
                    
                    if output.status.success() {
                        println!("Successfully cloned {}", repo.path);
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        println!("Failed to clone {}: {}", repo.path, error);
                    }
                } else {
                    println!("Pulling updates for {}", repo.path);
                    
                    let output = Command::new("git")
                        .current_dir(&path)
                        .args(&["pull", "origin", &repo.branch])
                        .output()
                        .expect("Failed to execute git pull command");
                    
                    if output.status.success() {
                        println!("Successfully pulled updates for {}", repo.path);
                    } else {
                        let error = String::from_utf8_lossy(&output.stderr);
                        println!("Failed to pull updates for {}: {}", repo.path, error);
                    }
                }
            }
        },
    }
    
    Ok(())
}