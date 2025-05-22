use clap::{Parser, Subcommand};
use colored::Colorize;
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};
use std::env;
use std::path::PathBuf;
use std::process;

/// Command-line interface for managing mirror.toml configuration files
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the mirror.toml file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new empty mirror.toml file
    Init {
        /// Force creation even if file exists
        #[arg(short, long)]
        force: bool,
    },
    /// Add a new repository to the configuration
    Add {
        /// Git repository origin URL
        #[arg(short, long, required = true)]
        origin: String,
        
        /// Git branch to use
        #[arg(short, long, default_value = "main")]
        branch: String,
        
        /// Local filesystem path where the repository should be cloned
        #[arg(short, long, required = true)]
        path: String,
        
        /// Optional unique identifier for the repository
        #[arg(short, long)]
        id: Option<String>,
        
        /// Whether the branch is locked (cannot be changed)
        #[arg(long)]
        branch_lock: bool,
        
        /// Optional tags for categorizing repositories
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,
    },
    /// Remove a repository from the configuration
    Remove {
        /// Repository path to remove
        #[arg(short, long, conflicts_with = "id")]
        path: Option<String>,
        
        /// Repository ID to remove
        #[arg(short, long, conflicts_with = "path")]
        id: Option<String>,
    },
    /// List all repositories in the configuration
    List {
        /// Filter repositories by tag
        #[arg(short, long)]
        tag: Option<String>,
    },
    /// Update a repository's properties
    Update {
        /// Repository path to update
        #[arg(short, long, required = true)]
        path: String,
        
        /// New Git repository origin URL
        #[arg(short, long)]
        origin: Option<String>,
        
        /// New Git branch to use
        #[arg(short, long)]
        branch: Option<String>,
        
        /// New local filesystem path
        #[arg(long)]
        new_path: Option<String>,
        
        /// New unique identifier
        #[arg(short, long)]
        id: Option<String>,
        
        /// Whether the branch is locked (cannot be changed)
        #[arg(long)]
        branch_lock: Option<bool>,
        
        /// Tags to add (comma-separated)
        #[arg(long, value_delimiter = ',')]
        add_tags: Vec<String>,
        
        /// Tags to remove (comma-separated)
        #[arg(long, value_delimiter = ',')]
        remove_tags: Vec<String>,
    },
    /// Validate the mirror.toml file
    Validate,
}

fn main() {
    let cli = Cli::parse();
    
    // Get config path from CLI, environment variable, or default
    let config_path = match get_config_path(cli.config) {
        Ok(path) => path,
        Err(err) => {
            eprintln!("{} {}", "Error:".bright_red(), err);
            process::exit(1);
        }
    };
    
    let sdk = MirrorSdk::new();
    
    match &cli.command {
        Commands::Init { force } => {
            match init_command(&sdk, &config_path, *force) {
                Ok(_) => println!("{} Created new mirror.toml file at {}", "Success:".bright_green(), config_path.display()),
                Err(err) => {
                    eprintln!("{} {}", "Error:".bright_red(), err);
                    process::exit(1);
                }
            }
        },
        Commands::Add { origin, branch, path, id, branch_lock, tags } => {
            match add_command(&sdk, &config_path, origin, branch, path, id, *branch_lock, tags) {
                Ok(_) => println!("{} Added repository {} to {}", "Success:".bright_green(), path, config_path.display()),
                Err(err) => {
                    eprintln!("{} {}", "Error:".bright_red(), err);
                    process::exit(1);
                }
            }
        },
        Commands::Remove { path, id } => {
            match remove_command(&sdk, &config_path, path, id) {
                Ok(_) => {
                    if let Some(p) = path {
                        println!("{} Removed repository with path {} from {}", "Success:".bright_green(), p, config_path.display());
                    } else if let Some(i) = id {
                        println!("{} Removed repository with ID {} from {}", "Success:".bright_green(), i, config_path.display());
                    }
                },
                Err(err) => {
                    eprintln!("{} {}", "Error:".bright_red(), err);
                    process::exit(1);
                }
            }
        },
        Commands::List { tag } => {
            match list_command(&sdk, &config_path, tag) {
                Ok(_) => {},
                Err(err) => {
                    eprintln!("{} {}", "Error:".bright_red(), err);
                    process::exit(1);
                }
            }
        },
        Commands::Update { path, origin, branch, new_path, id, branch_lock, add_tags, remove_tags } => {
            match update_command(&sdk, &config_path, path, origin, branch, new_path, id, branch_lock, add_tags, remove_tags) {
                Ok(_) => println!("{} Updated repository {} in {}", "Success:".bright_green(), path, config_path.display()),
                Err(err) => {
                    eprintln!("{} {}", "Error:".bright_red(), err);
                    process::exit(1);
                }
            }
        },
        Commands::Validate => {
            match validate_command(&sdk, &config_path) {
                Ok(_) => println!("{} Configuration is valid", "Success:".bright_green()),
                Err(err) => {
                    eprintln!("{} {}", "Error:".bright_red(), err);
                    process::exit(1);
                }
            }
        },
    }
}

/// Get the configuration file path from CLI args, environment variable, or default
fn get_config_path(cli_path: Option<PathBuf>) -> Result<PathBuf, String> {
    // Priority: CLI arg > Environment variable > Default
    if let Some(path) = cli_path {
        return Ok(path);
    }
    
    if let Ok(path) = env::var("MIRROR_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    
    let sdk = MirrorSdk::new();
    match sdk.get_config_path() {
        Ok(path) => Ok(path),
        Err(_) => Ok(PathBuf::from("mirror.toml")),
    }
}

/// Initialize a new mirror.toml file
fn init_command(sdk: &MirrorSdk, config_path: &PathBuf, force: bool) -> Result<(), MirrorError> {
    sdk.init_config(config_path, force)?;
    Ok(())
}

/// Add a repository to the configuration
fn add_command(
    sdk: &MirrorSdk,
    config_path: &PathBuf,
    origin: &str,
    branch: &str,
    path: &str,
    id: &Option<String>,
    branch_lock: bool,
    tags: &Vec<String>,
) -> Result<(), MirrorError> {
    // Load existing config or create new one if it doesn't exist
    let mut config = match sdk.load_config(config_path) {
        Ok(config) => config,
        Err(MirrorError::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("{} Configuration file not found, creating new one", "Info:".bright_blue());
            sdk.new_config()
        },
        Err(e) => return Err(e),
    };
    
    // Build repository
    let mut builder = RepositoryBuilder::new()
        .origin(origin)
        .branch(branch)
        .path(path)
        .branch_lock(branch_lock);
    
    // Add ID if provided
    if let Some(id_value) = id {
        builder = builder.id(id_value);
    }
    
    // Add tags if provided
    for tag in tags {
        builder = builder.tag(tag);
    }
    
    let repo = builder.build()?;
    
    // Add repository to config
    sdk.add_repository(&mut config, repo)?;
    
    // Save config
    sdk.save_config(&config, config_path)?;
    
    Ok(())
}

/// Remove a repository from the configuration
fn remove_command(
    sdk: &MirrorSdk,
    config_path: &PathBuf,
    path: &Option<String>,
    id: &Option<String>,
) -> Result<(), MirrorError> {
    // Load existing config
    let mut config = sdk.load_config(config_path)?;
    
    // Remove repository by path or ID
    if let Some(path_value) = path {
        sdk.remove_repository_by_path(&mut config, path_value)?;
    } else if let Some(id_value) = id {
        sdk.remove_repository_by_id(&mut config, id_value)?;
    } else {
        return Err(MirrorError::InvalidConfiguration(
            "Either path or ID must be provided".to_string(),
        ));
    }
    
    // Save config
    sdk.save_config(&config, config_path)?;
    
    Ok(())
}

/// List repositories in the configuration
fn list_command(
    sdk: &MirrorSdk,
    config_path: &PathBuf,
    tag: &Option<String>,
) -> Result<(), MirrorError> {
    // Load existing config
    let config = sdk.load_config(config_path)?;
    
    // Filter repositories by tag if provided
    let repositories = if let Some(tag_value) = tag {
        println!("{} Repositories with tag '{}':", "Listing:".bright_blue(), tag_value);
        sdk.find_repositories_by_tag(&config, tag_value)
            .into_iter()
            .collect::<Vec<_>>()
    } else {
        println!("{} All repositories:", "Listing:".bright_blue());
        config.repositories.iter().collect::<Vec<_>>()
    };
    
    // Print repositories
    if repositories.is_empty() {
        println!("  No repositories found");
    } else {
        for (i, repo) in repositories.iter().enumerate() {
            println!("{} {}", (i + 1).to_string().bright_yellow(), repo.path.bright_green());
            if let Some(id) = &repo.id {
                println!("  {}: {}", "ID".bright_blue(), id);
            }
            println!("  {}: {}", "Origin".bright_blue(), repo.origin);
            println!("  {}: {}", "Branch".bright_blue(), repo.branch);
            if repo.branch_lock {
                println!("  {}: {}", "Branch Lock".bright_blue(), "true".bright_red());
            }
            if !repo.tags.is_empty() {
                println!("  {}: {}", "Tags".bright_blue(), repo.tags.join(", "));
            }
            if i < repositories.len() - 1 {
                println!();
            }
        }
    }
    
    Ok(())
}

/// Update a repository in the configuration
fn update_command(
    sdk: &MirrorSdk,
    config_path: &PathBuf,
    path: &str,
    origin: &Option<String>,
    branch: &Option<String>,
    new_path: &Option<String>,
    id: &Option<String>,
    branch_lock: &Option<bool>,
    add_tags: &Vec<String>,
    remove_tags: &Vec<String>,
) -> Result<(), MirrorError> {
    // Load existing config
    let mut config = sdk.load_config(config_path)?;
    
    // Find repository by path
    let repo = match sdk.find_repository_by_path(&config, path) {
        Some(repo) => repo.clone(),
        None => return Err(MirrorError::RepositoryNotFound(path.to_string())),
    };
    
    // Build updated repository
    let mut builder = RepositoryBuilder::new()
        .origin(repo.origin)
        .branch(repo.branch)
        .path(new_path.as_ref().unwrap_or(&repo.path))
        .branch_lock(branch_lock.unwrap_or(repo.branch_lock));
    
    // Set ID
    if let Some(id_value) = id {
        builder = builder.id(id_value);
    } else if let Some(id_value) = repo.id {
        builder = builder.id(id_value);
    }
    
    // Update origin if provided
    if let Some(origin_value) = origin {
        builder = builder.origin(origin_value);
    }
    
    // Update branch if provided
    if let Some(branch_value) = branch {
        builder = builder.branch(branch_value);
    }
    
    // Add existing tags that are not in remove_tags
    for tag in &repo.tags {
        if !remove_tags.contains(tag) {
            builder = builder.tag(tag);
        }
    }
    
    // Add new tags
    for tag in add_tags {
        if !repo.tags.contains(tag) {
            builder = builder.tag(tag);
        }
    }
    
    let updated_repo = builder.build()?;
    
    // Remove old repository and add updated one
    sdk.remove_repository_by_path(&mut config, path)?;
    sdk.add_repository(&mut config, updated_repo)?;
    
    // Save config
    sdk.save_config(&config, config_path)?;
    
    Ok(())
}

/// Validate the configuration
fn validate_command(
    sdk: &MirrorSdk,
    config_path: &PathBuf,
) -> Result<(), MirrorError> {
    // Load existing config
    let config = sdk.load_config(config_path)?;
    
    // Validate config
    match sdk.validate_config(&config) {
        Ok(_) => Ok(()),
        Err(err) => Err(MirrorError::Validation(err)),
    }
}