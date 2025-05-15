//! # Command Line Interface Module
//!
//! This module defines the command-line interface structure using clap.
//! It handles CLI argument parsing and command identification for MCTL.
//!
//! The CLI layer:
//! - Defines all available commands and arguments
//! - Provides detailed help and usage examples
//! - Validates input parameters
//! - Supports machine-readable output formats

use std::path::PathBuf;
use std::str::FromStr;
use clap::{Parser, Subcommand, Args, ValueEnum, builder::PossibleValue};
use crate::presentation::output::{OutputFormat, ColorMode};

/// MCTL - Multiple repository management tool
#[derive(Parser, Debug)]
#[command(
    name = "mctl",
    author,
    version,
    about = "Military-grade multiple repository management tool",
    long_about = "MCTL helps manage multiple Git repositories with support for SSH authentication and centralized configuration. It provides a unified interface for synchronizing, checking status, and committing changes across multiple repositories.",
    after_help = "EXAMPLES:
    # Synchronize all repositories
    mctl sync

    # Check status of repositories with changes only
    mctl status --changes-only

    # Commit and push changes with a message
    mctl save --message \"Update documentation\" --push

    # Initialize a new configuration
    mctl init --output ./mctl.toml

    # Use verbose output
    mctl -vv status",
    term_width = 100
)]
pub struct Cli {
    /// Configuration file path
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "Path to configuration file (default: ./mirror.toml, ~/.config/mctl/config.toml)",
        long_help = "Path to the TOML configuration file that defines repositories and global settings. \
                     If not specified, MCTL will look for './mirror.toml' in the current directory, \
                     then '~/.config/mctl/config.toml' in the user's home directory."
    )]
    pub config_path: Option<PathBuf>,
    
    /// Verbose output mode
    #[arg(
        short,
        long,
        action = clap::ArgAction::Count,
        help = "Increase verbosity level (can be used multiple times)",
        long_help = "Set verbosity level for output and logging:\n\
                     -v: Info level (shows major operations)\n\
                     -vv: Debug level (shows detailed operations)\n\
                     -vvv: Trace level (shows all operations including external commands)"
    )]
    pub verbose: u8,
    
    /// Output format
    #[arg(
        short,
        long,
        value_enum,
        help = "Output format (text, json, compact)",
        long_help = "Specify the output format:\n\
                     text: Human-readable text with colors (default)\n\
                     json: Machine-readable JSON format\n\
                     compact: Compact single-line format for scripts"
    )]
    pub format: Option<OutputFormat>,
    
    /// Color mode
    #[arg(
        long,
        value_enum,
        help = "Color output mode (auto, always, never)",
        long_help = "Control when to use colored output:\n\
                     auto: Use colors when output is to a terminal (default)\n\
                     always: Always use colors\n\
                     never: Never use colors"
    )]
    pub color: Option<ColorMode>,
    
    /// Command to execute
    #[command(subcommand)]
    pub command: Commands,
    
    /// Additional arguments passed to the command
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        help = "Additional arguments passed to the command"
    )]
    pub args: Vec<String>,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Synchronize repositories (clone/pull)
    #[command(
        about = "Synchronize repositories by cloning or pulling",
        long_about = "Synchronize repositories by cloning them if they don't exist or pulling the latest changes if they do. \
                      This command ensures all repositories are up-to-date with their remote origins.",
        after_help = "EXAMPLES:
    # Sync all repositories
    mctl sync

    # Sync specific repositories
    mctl sync repo1 repo2

    # Sync repositories with a specific tag
    mctl sync --tag frontend

    # Sync repositories with recursive submodule update
    mctl sync --recursive

    # Sync repositories in parallel
    mctl sync --parallel",
    )]
    Sync(SyncArgs),
    
    /// Check status of repositories
    #[command(
        about = "Check status of repositories",
        long_about = "Check the status of repositories and report any uncommitted changes or unpushed commits. \
                      This provides a quick overview of which repositories need attention.",
        after_help = "EXAMPLES:
    # Check status of all repositories
    mctl status

    # Show only repositories with changes
    mctl status --changes-only

    # Check status including untracked files
    mctl status --include-untracked

    # Check status of specific repositories
    mctl status repo1 repo2",
    )]
    Status(StatusArgs),
    
    /// Save changes (commit/push)
    #[command(
        about = "Save changes to repositories (commit/push)",
        long_about = "Save changes to repositories by committing and optionally pushing them. \
                      This is useful for quickly saving work across multiple repositories.",
        after_help = "EXAMPLES:
    # Commit changes with default message
    mctl save

    # Commit with custom message
    mctl save --message \"Update documentation\"

    # Commit and push changes
    mctl save --push

    # Commit changes in specific repositories
    mctl save repo1 repo2 --message \"Fix bug\"",
    )]
    Save(SaveArgs),
    
    /// Initialize a new configuration
    #[command(
        about = "Initialize a new configuration file",
        long_about = "Create a new configuration file with default settings. \
                      This is the first step to set up MCTL for your project.",
        after_help = "EXAMPLES:
    # Create a new configuration file in current directory
    mctl init

    # Create a configuration file with a custom path
    mctl init --output ~/projects/mctl.toml

    # Force overwrite an existing configuration
    mctl init --force

    # Set default SSH key for repositories
    mctl init --ssh-key ~/.ssh/id_rsa",
    )]
    Init(InitArgs),
}

/// Arguments for the sync command
#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Repository names or tags to sync (default: all)
    #[arg(
        value_name = "REPOS",
        help = "Repository names or tags to sync (default: all)",
        long_help = "Specify repository names or tags to sync. If not provided, all repositories will be synchronized. \
                     Partial matches are supported for both names and tags."
    )]
    pub repos: Vec<String>,
    
    /// Filter repositories by tag
    #[arg(
        short,
        long,
        value_name = "TAG",
        help = "Filter repositories by tag"
    )]
    pub tag: Option<String>,
    
    /// Recursively clone submodules
    #[arg(
        short,
        long,
        help = "Recursively clone submodules",
        long_help = "Update submodules recursively after cloning or pulling repositories. \
                     This ensures that any dependencies referenced via Git submodules are also synchronized."
    )]
    pub recursive: bool,
    
    /// Clone depth
    #[arg(
        long,
        value_name = "DEPTH",
        help = "Limit clone depth (number of commits)",
        long_help = "Limit the clone depth to the specified number of commits. \
                     This creates a shallow clone which can be useful for large repositories \
                     when you don't need the full history."
    )]
    pub depth: Option<u32>,
    
    /// Parallel execution
    #[arg(
        short,
        long,
        help = "Execute operations in parallel",
        long_help = "Execute repository operations in parallel for faster performance. \
                     This can significantly speed up operations when dealing with many repositories."
    )]
    pub parallel: bool,
    
    /// Fail fast
    #[arg(
        long,
        help = "Stop on first error",
        long_help = "Stop synchronization on the first error encountered instead of continuing with other repositories. \
                     By default, MCTL attempts to process all repositories even if some fail."
    )]
    pub fail_fast: bool,
}

/// Arguments for the status command
#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Repository names or tags to check (default: all)
    #[arg(
        value_name = "REPOS",
        help = "Repository names or tags to check (default: all)",
        long_help = "Specify repository names or tags to check status for. If not provided, all repositories will be checked. \
                     Partial matches are supported for both names and tags."
    )]
    pub repos: Vec<String>,
    
    /// Filter repositories by tag
    #[arg(
        short,
        long,
        value_name = "TAG",
        help = "Filter repositories by tag"
    )]
    pub tag: Option<String>,
    
    /// Show only repositories with changes
    #[arg(
        short,
        long,
        help = "Show only repositories with changes",
        long_help = "Only display repositories that have uncommitted changes or unpushed commits. \
                     This helps focus on repositories that need attention."
    )]
    pub changes_only: bool,
    
    /// Include untracked files
    #[arg(
        short = 'u',
        long,
        help = "Include untracked files in status",
        long_help = "Include untracked files when checking repository status. \
                     By default, only tracked files with modifications are considered."
    )]
    pub include_untracked: bool,
    
    /// Show detailed status
    #[arg(
        short = 'd',
        long,
        help = "Show detailed status information",
        long_help = "Show detailed status information including specific files changed. \
                     By default, only a summary is shown."
    )]
    pub detailed: bool,
}

/// Arguments for the save command
#[derive(Args, Debug)]
pub struct SaveArgs {
    /// Repository names or tags to save (default: all)
    #[arg(
        value_name = "REPOS",
        help = "Repository names or tags to save (default: all)",
        long_help = "Specify repository names or tags to save changes for. If not provided, all repositories with changes will be saved. \
                     Partial matches are supported for both names and tags."
    )]
    pub repos: Vec<String>,
    
    /// Filter repositories by tag
    #[arg(
        short,
        long,
        value_name = "TAG",
        help = "Filter repositories by tag"
    )]
    pub tag: Option<String>,
    
    /// Commit message
    #[arg(
        short,
        long,
        default_value = "Auto-commit by MCTL",
        help = "Commit message",
        long_help = "Message to use for the commit. If not specified, a default message will be used."
    )]
    pub message: String,
    
    /// Push changes after commit
    #[arg(
        short,
        long,
        help = "Push changes after commit",
        long_help = "Push committed changes to the remote repository after committing. \
                     This ensures your changes are immediately available on the remote."
    )]
    pub push: bool,
    
    /// Sign commits
    #[arg(
        short,
        long,
        help = "Sign commits with GPG",
        long_help = "Sign commits with GPG using your default signing key. \
                     This adds a verification signature to your commits."
    )]
    pub sign: bool,
    
    /// Include all files (tracked and untracked)
    #[arg(
        short = 'a',
        long,
        help = "Include all files (tracked and untracked)",
        long_help = "Include all files (tracked and untracked) in the commit. \
                     By default, only tracked files with modifications are committed."
    )]
    pub all: bool,
    
    /// Skip empty commits
    #[arg(
        long,
        help = "Skip repositories with no changes",
        long_help = "Skip repositories that have no changes to commit. \
                     By default, MCTL will attempt to commit in all specified repositories."
    )]
    pub skip_empty: bool,
}

/// Arguments for the init command
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Output file path for new configuration
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "./mirror.toml",
        help = "Output file path for new configuration",
        long_help = "Path where the new configuration file should be created. \
                     If not specified, it will be created in the current directory as 'mirror.toml'."
    )]
    pub output: PathBuf,
    
    /// Force overwrite if file exists
    #[arg(
        short,
        long,
        help = "Force overwrite if file exists",
        long_help = "Overwrite the configuration file if it already exists. \
                     By default, MCTL will not overwrite existing configuration files."
    )]
    pub force: bool,
    
    /// Default SSH key path
    #[arg(
        long,
        value_name = "PATH",
        help = "Default SSH key path",
        long_help = "Path to the SSH key to use for repository authentication by default. \
                     This can be overridden per repository in the configuration."
    )]
    pub ssh_key: Option<PathBuf>,
    
    /// Interactive mode
    #[arg(
        short,
        long,
        help = "Use interactive mode for setup",
        long_help = "Use interactive mode to configure repositories and settings through a guided process. \
                     This makes initial setup easier, especially for new users."
    )]
    pub interactive: bool,
    
    /// Template configuration
    #[arg(
        short,
        long,
        value_name = "TEMPLATE",
        help = "Use a template for configuration",
        long_help = "Use a predefined template for the configuration. Available templates: \
                     'basic', 'monorepo', 'multi-team'."
    )]
    pub template: Option<String>,
}

/// Implementation for parsing string arguments into structs
impl SyncArgs {
    pub fn from_args(args: &[String]) -> Result<Self, String> {
        let mut repos = Vec::new();
        let mut recursive = false;
        let mut depth = None;
        let mut parallel = false;
        let mut fail_fast = false;
        let mut tag = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-r" | "--recursive" => recursive = true,
                "--depth" => {
                    if i + 1 < args.len() {
                        depth = Some(args[i + 1].parse().map_err(|_| "Invalid depth value".to_string())?);
                        i += 1;
                    } else {
                        return Err("Missing value for --depth".to_string());
                    }
                },
                "-p" | "--parallel" => parallel = true,
                "--fail-fast" => fail_fast = true,
                "-t" | "--tag" => {
                    if i + 1 < args.len() {
                        tag = Some(args[i + 1].clone());
                        i += 1;
                    } else {
                        return Err("Missing value for --tag".to_string());
                    }
                },
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg));
                },
                _ => repos.push(args[i].clone()),
            }
            i += 1;
        }
        
        Ok(Self {
            repos,
            tag,
            recursive,
            depth,
            parallel,
            fail_fast,
        })
    }
}

impl StatusArgs {
    pub fn from_args(args: &[String]) -> Result<Self, String> {
        let mut repos = Vec::new();
        let mut changes_only = false;
        let mut include_untracked = false;
        let mut detailed = false;
        let mut tag = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-c" | "--changes-only" => changes_only = true,
                "-u" | "--include-untracked" => include_untracked = true,
                "-d" | "--detailed" => detailed = true,
                "-t" | "--tag" => {
                    if i + 1 < args.len() {
                        tag = Some(args[i + 1].clone());
                        i += 1;
                    } else {
                        return Err("Missing value for --tag".to_string());
                    }
                },
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg));
                },
                _ => repos.push(args[i].clone()),
            }
            i += 1;
        }
        
        Ok(Self {
            repos,
            tag,
            changes_only,
            include_untracked,
            detailed,
        })
    }
}

impl SaveArgs {
    pub fn from_args(args: &[String]) -> Result<Self, String> {
        let mut repos = Vec::new();
        let mut message = "Auto-commit by MCTL".to_string();
        let mut push = false;
        let mut sign = false;
        let mut all = false;
        let mut skip_empty = false;
        let mut tag = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-m" | "--message" => {
                    if i + 1 < args.len() {
                        message = args[i + 1].clone();
                        i += 1;
                    } else {
                        return Err("Missing value for --message".to_string());
                    }
                },
                "-p" | "--push" => push = true,
                "-s" | "--sign" => sign = true,
                "-a" | "--all" => all = true,
                "--skip-empty" => skip_empty = true,
                "-t" | "--tag" => {
                    if i + 1 < args.len() {
                        tag = Some(args[i + 1].clone());
                        i += 1;
                    } else {
                        return Err("Missing value for --tag".to_string());
                    }
                },
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg));
                },
                _ => repos.push(args[i].clone()),
            }
            i += 1;
        }
        
        Ok(Self {
            repos,
            tag,
            message,
            push,
            sign,
            all,
            skip_empty,
        })
    }
}

impl InitArgs {
    pub fn from_args(args: &[String]) -> Result<Self, String> {
        let mut output = PathBuf::from("./mirror.toml");
        let mut force = false;
        let mut ssh_key = None;
        let mut interactive = false;
        let mut template = None;
        
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "-o" | "--output" => {
                    if i + 1 < args.len() {
                        output = PathBuf::from(&args[i + 1]);
                        i += 1;
                    } else {
                        return Err("Missing value for --output".to_string());
                    }
                },
                "-f" | "--force" => force = true,
                "--ssh-key" => {
                    if i + 1 < args.len() {
                        ssh_key = Some(PathBuf::from(&args[i + 1]));
                        i += 1;
                    } else {
                        return Err("Missing value for --ssh-key".to_string());
                    }
                },
                "-i" | "--interactive" => interactive = true,
                "-t" | "--template" => {
                    if i + 1 < args.len() {
                        template = Some(args[i + 1].clone());
                        i += 1;
                    } else {
                        return Err("Missing value for --template".to_string());
                    }
                },
                arg if arg.starts_with('-') => {
                    return Err(format!("Unknown option: {}", arg));
                },
                _ => return Err(format!("Unexpected argument: {}", args[i])),
            }
            i += 1;
        }
        
        Ok(Self {
            output,
            force,
            ssh_key,
            interactive,
            template,
        })
    }
}