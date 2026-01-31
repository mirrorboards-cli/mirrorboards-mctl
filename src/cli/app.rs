//! CLI application definition using clap.

use clap::{Parser, Subcommand};

/// Mirror configuration management tool.
///
/// mctl helps manage multiple git repositories defined in a mirror.toml configuration file.
/// It supports workspaces for grouping repositories, version pinning (branch/rev/tag),
/// and includes for composing configurations from multiple files.
#[derive(Parser, Debug)]
#[command(name = "mctl")]
#[command(version)]
#[command(about = "Mirror configuration management tool", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, default_value = "mirror.toml", global = true)]
    pub config: String,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize a new mirror.toml configuration
    Init {
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },

    /// Add a repository to the configuration
    Add {
        /// Git URL of the repository
        git: String,

        /// Local path for the repository (optional, derived from URL if not provided)
        #[arg(short, long)]
        path: Option<String>,

        /// Branch to track
        #[arg(short, long)]
        branch: Option<String>,

        /// Specific revision (commit hash)
        #[arg(long)]
        rev: Option<String>,

        /// Specific tag
        #[arg(short, long)]
        tag: Option<String>,

        /// Workspaces to add the repository to
        #[arg(short, long, value_delimiter = ',')]
        workspace: Vec<String>,

        /// Skip push operations for this repository
        #[arg(long)]
        skip_push: bool,
    },

    /// List repositories in the configuration
    List {
        /// Filter by workspace
        workspace: Option<String>,

        /// Output format (table, json)
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Remove a repository from the configuration
    Remove {
        /// Path of the repository to remove
        path: String,

        /// Also delete the local directory
        #[arg(long)]
        delete: bool,
    },

    /// Show details of a repository
    Show {
        /// Path of the repository to show
        path: String,
    },

    /// Validate the configuration file
    Validate,

    /// Sync repositories (clone/pull)
    Sync {
        /// Filter by workspace
        workspace: Option<String>,

        /// Dry run - show what would be done
        #[arg(long)]
        dry_run: bool,

        /// Force sync even if there are local changes
        #[arg(short, long)]
        force: bool,
    },

    /// Show status of repositories
    Status {
        /// Filter by workspace
        workspace: Option<String>,

        /// Show detailed file changes
        #[arg(short, long)]
        detailed: bool,
    },

    /// Show diff of changes in repositories
    Diff {
        /// Filter by workspace
        workspace: Option<String>,

        /// Show only staged changes
        #[arg(long)]
        staged: bool,
    },

    /// Save (commit and push) changes in repositories
    Save {
        /// Filter by workspace
        workspace: Option<String>,

        /// Commit message
        #[arg(short, long, default_value = "Update")]
        message: String,

        /// Dry run - show what would be done
        #[arg(long)]
        dry_run: bool,
    },

    /// Create a snapshot of current repository states
    Snapshot {
        /// Filter by workspace
        workspace: Option<String>,

        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Remote config management
    #[command(subcommand)]
    Config(ConfigCommands),
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommands {
    /// Initialize remote config
    Init {
        /// Git URL for the remote config repository
        git: String,

        /// Branch to use
        #[arg(short, long, default_value = "main")]
        branch: String,

        /// Path to config file in the repository
        #[arg(short, long, default_value = "mirror.toml")]
        path: String,
    },

    /// Pull config from remote
    Pull,

    /// Push config to remote
    Push {
        /// Commit message
        #[arg(short, long, default_value = "Update mirror.toml")]
        message: String,
    },

    /// Show diff between local and remote config
    Diff,
}

impl Cli {
    pub fn parse_args() -> Self {
        Cli::parse()
    }
}
