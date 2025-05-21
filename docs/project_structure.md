# MCTL Project Structure

This document outlines the recommended project structure for implementing the MCTL (Mirror Control) system in Rust, based on the architecture defined in [architecture.md](architecture.md).

## Directory Structure

```
mctl/
├── Cargo.toml                 # Project manifest
├── .gitignore                 # Git ignore file
├── README.md                  # Project documentation
├── docs/                      # Documentation
│   ├── architecture.md        # Architecture documentation
│   └── project_structure.md   # This file
├── src/                       # Source code
│   ├── main.rs                # Entry point
│   ├── cli/                   # CLI interface module
│   │   ├── mod.rs             # Module definition
│   │   ├── args.rs            # Command-line argument parsing
│   │   └── commands.rs        # Command implementations
│   ├── config/                # Configuration management module
│   │   ├── mod.rs             # Module definition
│   │   ├── models.rs          # Configuration data structures
│   │   ├── parser.rs          # TOML parsing and validation
│   │   └── writer.rs          # Configuration file writing
│   ├── repo/                  # Repository engine module
│   │   ├── mod.rs             # Module definition
│   │   ├── sync.rs            # Repository synchronization
│   │   ├── save.rs            # Repository save operations
│   │   └── update.rs          # Repository update operations
│   ├── status/                # Status monitor module
│   │   ├── mod.rs             # Module definition
│   │   ├── checker.rs         # Status checking logic
│   │   └── formatter.rs       # Status output formatting
│   ├── git/                   # Git interface module
│   │   ├── mod.rs             # Module definition
│   │   ├── commands.rs        # Git command execution
│   │   ├── models.rs          # Git data structures
│   │   └── utils.rs           # Git utility functions
│   ├── security/              # Security layer module
│   │   ├── mod.rs             # Module definition
│   │   ├── credentials.rs     # Credential management
│   │   └── validation.rs      # Input validation
│   └── error/                 # Error handling module
│       ├── mod.rs             # Module definition
│       ├── types.rs           # Error type definitions
│       └── handler.rs         # Error handling logic
└── tests/                     # Integration tests
    ├── cli_tests.rs           # CLI integration tests
    ├── config_tests.rs        # Configuration tests
    ├── repo_tests.rs          # Repository operation tests
    ├── fixtures/              # Test fixtures
    │   └── sample_repos/      # Sample repositories for testing
    └── utils/                 # Test utilities
        ├── mod.rs             # Module definition
        ├── git_mock.rs        # Git operation mocking
        └── temp_dir.rs        # Temporary directory management
```

## Key Files

### Project Configuration

#### `Cargo.toml`

```toml
[package]
name = "mctl"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Mirror Control (MCTL) - A tool for efficient git repository synchronization and mirroring"
license = "MIT"
repository = "https://github.com/yourusername/mctl"
readme = "README.md"

[dependencies]
clap = { version = "4.4", features = ["derive"] }  # Command line argument parsing
toml = "0.8"                                       # TOML parsing
serde = { version = "1.0", features = ["derive"] } # Serialization/deserialization
thiserror = "1.0"                                  # Error handling
anyhow = "1.0"                                     # Error propagation
git2 = "0.18"                                      # Git operations
dirs = "5.0"                                       # Directory handling
log = "0.4"                                        # Logging
env_logger = "0.10"                                # Environment-based logger
tempfile = "3.8"                                   # Temporary file handling
regex = "1.10"                                     # Regular expressions
chrono = "0.4"                                     # Date and time handling
indicatif = "0.17"                                 # Progress indicators
console = "0.15"                                   # Terminal utilities
secrecy = "0.8"                                    # Secure credential handling

[dev-dependencies]
assert_cmd = "2.0"                                 # Command testing
predicates = "3.0"                                 # Test assertions
mockall = "0.11"                                   # Mocking framework
proptest = "1.3"                                   # Property-based testing
test-case = "3.3"                                  # Test case macros
```

### Source Code

#### `src/main.rs`

Entry point for the application:

```rust
mod cli;
mod config;
mod repo;
mod status;
mod git;
mod security;
mod error;

use cli::Cli;
use error::handler::ErrorHandler;

fn main() {
    // Initialize logging
    env_logger::init();
    
    // Create CLI instance
    let cli = Cli::new();
    
    // Parse arguments and execute command
    match cli.parse_args() {
        Ok(command) => {
            if let Err(err) = cli.execute(command) {
                let error_handler = ErrorHandler::new();
                eprintln!("{}", error_handler.handle_error(&err));
                std::process::exit(1);
            }
        }
        Err(err) => {
            let error_handler = ErrorHandler::new();
            eprintln!("{}", error_handler.handle_error(&err));
            std::process::exit(1);
        }
    }
}
```

#### `src/cli/mod.rs`

CLI module definition:

```rust
mod args;
mod commands;

pub use args::Cli;
pub use commands::Command;
```

#### `src/cli/args.rs`

Command-line argument parsing:

```rust
use clap::{Parser, Subcommand};
use crate::error::types::CliError;
use super::commands::Command;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a git repository to mirror.toml
    Add {
        /// Git URL of the repository to add
        #[arg(long)]
        git_url: Option<String>,
        
        /// Local path where the repository will be cloned
        #[arg(long)]
        path: Option<String>,
        
        /// Specific branch to track (optional)
        #[arg(long)]
        branch: Option<String>,
        
        /// Positional arguments for git URL and path
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Clone all repositories defined in mirror.toml
    Sync {
        /// Custom path to the configuration file
        #[arg(long)]
        config: Option<String>,
        
        /// Alias for --config
        #[arg(long)]
        mirror: Option<String>,
        
        /// Custom destination directory for cloned repositories
        #[arg(long)]
        dest: Option<String>,
        
        /// Enable verbose output
        #[arg(long)]
        verbose: bool,
        
        /// Skip pulling updates for existing repositories
        #[arg(long)]
        no_pull: bool,
        
        /// Force pull even if it might cause conflicts
        #[arg(long)]
        force: bool,
        
        /// Clone or pull multiple repositories in parallel
        #[arg(long)]
        parallel: Option<usize>,
    },
    
    /// Check status of all repositories defined in mirror.toml
    Status {
        /// Custom path to the configuration file
        #[arg(long)]
        config: Option<String>,
        
        /// Enable verbose output
        #[arg(long)]
        verbose: bool,
    },
    
    /// Commit and push changes in all repositories
    Save {
        /// Custom commit message
        #[arg(long)]
        message: Option<String>,
        
        /// Positional argument for commit message
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    
    /// Update repositories with latest changes from remote sources
    Update {
        /// Custom path to the configuration file
        #[arg(long)]
        config: Option<String>,
        
        /// Enable verbose output
        #[arg(long)]
        verbose: bool,
        
        /// Force update even when there might be conflicts
        #[arg(long)]
        force: bool,
        
        /// Show what would be updated without making changes
        #[arg(long)]
        dry_run: bool,
        
        /// Update only the specified repository
        #[arg(long)]
        repo: Option<String>,
    },
}

impl Cli {
    pub fn new() -> Self {
        Self {
            command: None,
        }
    }
    
    pub fn parse_args(&self) -> Result<Command, CliError> {
        let cli = Self::parse();
        
        match cli.command {
            Some(Commands::Add { git_url, path, branch, args }) => {
                // Parse add command
                // ...
            },
            Some(Commands::Sync { config, mirror, dest, verbose, no_pull, force, parallel }) => {
                // Parse sync command
                // ...
            },
            Some(Commands::Status { config, verbose }) => {
                // Parse status command
                // ...
            },
            Some(Commands::Save { message, args }) => {
                // Parse save command
                // ...
            },
            Some(Commands::Update { config, verbose, force, dry_run, repo }) => {
                // Parse update command
                // ...
            },
            None => {
                Err(CliError::new(
                    crate::error::types::ErrorCode::MissingCommand,
                    "No command specified".to_string(),
                ))
            }
        }
    }
    
    pub fn execute(&self, command: Command) -> Result<(), CliError> {
        match command {
            Command::Add { git_url, path, branch } => {
                // Execute add command
                // ...
            },
            Command::Sync { config_path, dest, no_pull, force, parallel } => {
                // Execute sync command
                // ...
            },
            Command::Status { config_path, verbose } => {
                // Execute status command
                // ...
            },
            Command::Save { message } => {
                // Execute save command
                // ...
            },
            Command::Update { config_path, verbose, force, dry_run, repo } => {
                // Execute update command
                // ...
            },
        }
    }
}
```

#### `src/config/models.rs`

Configuration data structures:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub repositories: Vec<Repository>,
    pub base_path: Option<String>,
    pub default_branch: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Repository {
    #[serde(rename = "git-url")]
    pub git_url: String,
    pub path: String,
    pub branch: Option<String>,
    pub name: Option<String>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
            base_path: None,
            default_branch: None,
        }
    }
    
    pub fn add_repository(&mut self, repo: Repository) -> Result<(), String> {
        // Validate repository
        if repo.git_url.is_empty() {
            return Err("Git URL cannot be empty".to_string());
        }
        
        if repo.path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }
        
        // Check for duplicates
        if self.repositories.iter().any(|r| r.path == repo.path) {
            // Update existing repository
            let index = self.repositories.iter().position(|r| r.path == repo.path).unwrap();
            self.repositories[index] = repo;
        } else {
            // Add new repository
            self.repositories.push(repo);
        }
        
        Ok(())
    }
}

impl Repository {
    pub fn new(git_url: String, path: String, branch: Option<String>) -> Self {
        Self {
            git_url,
            path,
            branch,
            name: None,
        }
    }
    
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
}
```

#### `src/error/types.rs`

Error type definitions:

```rust
use thiserror::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    // CLI errors
    InvalidArgument,
    MissingCommand,
    MissingRequiredOption,
    
    // Config errors
    ConfigNotFound,
    InvalidConfigFormat,
    ConfigWriteFailed,
    
    // Repository errors
    RepositoryNotFound,
    RepositoryAccessDenied,
    
    // Git errors
    GitCommandFailed,
    GitAuthenticationFailed,
    GitMergeConflict,
    
    // Security errors
    CredentialsNotFound,
    InvalidCredentials,
    CredentialStoreFailed,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::InvalidArgument => write!(f, "E001"),
            ErrorCode::MissingCommand => write!(f, "E002"),
            ErrorCode::MissingRequiredOption => write!(f, "E003"),
            ErrorCode::ConfigNotFound => write!(f, "E101"),
            ErrorCode::InvalidConfigFormat => write!(f, "E102"),
            ErrorCode::ConfigWriteFailed => write!(f, "E103"),
            ErrorCode::RepositoryNotFound => write!(f, "E201"),
            ErrorCode::RepositoryAccessDenied => write!(f, "E202"),
            ErrorCode::GitCommandFailed => write!(f, "E301"),
            ErrorCode::GitAuthenticationFailed => write!(f, "E302"),
            ErrorCode::GitMergeConflict => write!(f, "E303"),
            ErrorCode::CredentialsNotFound => write!(f, "E401"),
            ErrorCode::InvalidCredentials => write!(f, "E402"),
            ErrorCode::CredentialStoreFailed => write!(f, "E403"),
        }
    }
}

#[derive(Error, Debug)]
pub struct CliError {
    pub code: ErrorCode,
    pub message: String,
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub context: Option<String>,
}

impl CliError {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            source: None,
            context: None,
        }
    }
    
    pub fn with_source(mut self, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        self.source = Some(source);
        self
    }
    
    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        
        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }
        
        Ok(())
    }
}

// Similar error types for other modules
// ConfigError, RepoError, GitError, StatusError, SecurityError
```

## Testing Strategy

### Unit Tests

Each module should include unit tests in the same file or in a `tests` submodule:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_example() {
        // Test code here
    }
}
```

### Integration Tests

Integration tests should be placed in the `tests` directory:

```rust
// tests/config_tests.rs
use mctl::config::{Config, Repository};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_load_valid_config() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");
    
    fs::write(&config_path, r#"
        [[repositories]]
        git-url = "git@github.com:example/repo.git"
        path = "example-repo"
    "#).unwrap();
    
    let config = Config::load(&config_path).unwrap();
    assert_eq!(config.repositories.len(), 1);
    assert_eq!(config.repositories[0].git_url, "git@github.com:example/repo.git");
    assert_eq!(config.repositories[0].path, "example-repo");
}
```

## Implementation Guidelines

1. **Modular Design**: Follow the modular architecture defined in the architecture document
2. **Error Handling**: Use the `thiserror` and `anyhow` crates for robust error handling
3. **Configuration**: Use `serde` for TOML serialization/deserialization
4. **Git Operations**: Use the `git2` crate for git operations
5. **CLI Interface**: Use the `clap` crate for command-line argument parsing
6. **Testing**: Write comprehensive tests using the testing framework
7. **Documentation**: Document all public APIs with rustdoc comments
8. **Security**: Use the `secrecy` crate for handling sensitive information