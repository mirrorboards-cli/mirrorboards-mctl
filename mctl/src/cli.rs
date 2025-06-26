use clap::{Parser, Subcommand};

/// mctl - Mirror Configuration Management Tool
/// 
/// A CLI tool for managing mirror.toml configuration files that define
/// collections of git repositories for large-scale IT projects.
#[derive(Parser)]
#[command(name = "mctl")]
#[command(about = "Mirror Configuration Management Tool")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = "mctl contributors")]
#[command(long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Configuration file path (defaults to mirror.toml)
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<String>,
    
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new mirror.toml configuration file
    Init {
        /// Force overwrite existing file
        #[arg(short, long)]
        force: bool,
    },
    
    /// Add a repository to the configuration
    Add {
        /// Git URL (SSH or HTTPS format)
        git_url: String,
        
        /// Custom local path (defaults to extracted org/repo from URL)
        #[arg(short, long)]
        path: Option<String>,
        
        /// Branch to track (defaults to "main")
        #[arg(short, long)]
        branch: Option<String>,
        
        /// Skip pushing to this repository
        #[arg(long)]
        skip_push: bool,
    },
    
    /// List all repositories in the configuration
    List {
        /// Output in JSON format
        #[arg(long)]
        json: bool,
    },
    
    /// Remove a repository by its hash ID
    Remove {
        /// Repository hash ID (supports partial matching)
        hash: String,
        
        /// Skip confirmation prompt
        #[arg(short, long)]
        force: bool,
    },
    
    /// Show detailed information about a repository
    Show {
        /// Repository hash ID (supports partial matching)
        hash: String,
    },
    
    /// Validate the configuration file
    Validate {
        /// Show detailed validation information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Synchronize repositories by cloning missing ones and updating existing ones
    Sync {
        /// Perform dry run without making changes
        #[arg(long)]
        dry_run: bool,
        
        /// Update existing repositories (pull latest changes)
        #[arg(long)]
        pull: bool,
        
        /// Force re-clone even if repository exists
        #[arg(long)]
        force: bool,
    },
}

impl Cli {
    /// Get the configuration file path, using default if not specified
    pub fn config_path(&self) -> String {
        self.config
            .clone()
            .unwrap_or_else(|| "mirror.toml".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_verify() {
        // Verify that the CLI can be built without errors
        Cli::command().debug_assert();
    }
    
    #[test]
    fn test_cli_parsing() {
        // Test basic command parsing
        let cli = Cli::try_parse_from(&["mctl", "init"]).unwrap();
        assert!(matches!(cli.command, Commands::Init { .. }));
        
        let cli = Cli::try_parse_from(&["mctl", "add", "git@github.com:org/repo.git"]).unwrap();
        if let Commands::Add { git_url, .. } = cli.command {
            assert_eq!(git_url, "git@github.com:org/repo.git");
        } else {
            panic!("Expected Add command");
        }
        
        let cli = Cli::try_parse_from(&["mctl", "list", "--json"]).unwrap();
        if let Commands::List { json } = cli.command {
            assert!(json);
        } else {
            panic!("Expected List command");
        }
        
        let cli = Cli::try_parse_from(&["mctl", "sync", "--dry-run", "--pull", "--force"]).unwrap();
        if let Commands::Sync { dry_run, pull, force } = cli.command {
            assert!(dry_run);
            assert!(pull);
            assert!(force);
        } else {
            panic!("Expected Sync command");
        }
    }
    
    #[test]
    fn test_global_options() {
        let cli = Cli::try_parse_from(&["mctl", "--config", "custom.toml", "--verbose", "init"]).unwrap();
        assert_eq!(cli.config_path(), "custom.toml");
        assert!(cli.verbose);
    }
}