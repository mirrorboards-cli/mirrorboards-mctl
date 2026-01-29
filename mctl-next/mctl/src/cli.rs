use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mctl")]
#[command(about = "Multi-repository management tool", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Path to mirror.toml config file
    #[arg(short, long, default_value = "mirror.toml")]
    pub config: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Clone or update repositories
    Sync {
        /// Sync only repositories in this workspace
        #[arg(short, long)]
        workspace: Option<String>,
    },

    /// List repositories
    List {
        /// Filter by workspace
        #[arg(short, long)]
        workspace: Option<String>,

        /// Group output by workspace
        #[arg(long)]
        by_workspace: bool,
    },

    /// Commit and push changes for a workspace
    Save {
        /// Workspace to save
        workspace: String,

        /// Custom commit message
        #[arg(short, long)]
        message: Option<String>,
    },

    /// Config repository operations
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },

    /// Snapshot operations
    Snapshot {
        #[command(subcommand)]
        command: SnapshotCommands,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Push mirror.toml to config repository
    Save {
        /// Commit message
        #[arg(short, long, default_value = "Update mirror.toml")]
        message: String,
    },

    /// Pull mirror.toml from config repository
    Pull,
}

#[derive(Subcommand)]
pub enum SnapshotCommands {
    /// Create a new snapshot
    Create {
        /// Snapshot name
        name: String,

        /// Optional description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List all snapshots
    List,

    /// Restore repositories to a snapshot state
    Restore {
        /// Snapshot name to restore
        name: String,
    },
}
