//! Command definitions for MCTL
//!
//! This module defines the command enum and related types for the MCTL CLI.

/// Command enum representing the available commands in MCTL
#[derive(Debug, Clone)]
pub enum Command {
    /// Add a git repository to mirror.toml
    Add {
        /// Git URL of the repository to add
        git_url: String,
        /// Local path where the repository will be cloned
        path: String,
        /// Specific branch to track (optional)
        branch: Option<String>,
    },

    /// Clone all repositories defined in mirror.toml
    Sync {
        /// Custom path to the configuration file
        config_path: Option<String>,
        /// Custom destination directory for cloned repositories
        dest: Option<String>,
        /// Skip pulling updates for existing repositories
        no_pull: bool,
        /// Force pull even if it might cause conflicts
        force: bool,
        /// Clone or pull multiple repositories in parallel
        parallel: Option<usize>,
    },

    /// Check status of all repositories defined in mirror.toml
    Status {
        /// Custom path to the configuration file
        config_path: Option<String>,
        /// Enable verbose output
        verbose: bool,
    },

    /// Commit and push changes in all repositories
    Save {
        /// Custom commit message
        message: Option<String>,
    },

    /// Update repositories with latest changes from remote sources
    Update {
        /// Custom path to the configuration file
        config_path: Option<String>,
        /// Enable verbose output
        verbose: bool,
        /// Force update even when there might be conflicts
        force: bool,
        /// Show what would be updated without making changes
        dry_run: bool,
        /// Update only the specified repository
        repo: Option<String>,
    },
}

impl Command {
    /// Returns the name of the command as a string
    pub fn name(&self) -> &'static str {
        match self {
            Command::Add { .. } => "add",
            Command::Sync { .. } => "sync",
            Command::Status { .. } => "status",
            Command::Save { .. } => "save",
            Command::Update { .. } => "update",
        }
    }

    /// Returns a description of the command
    pub fn description(&self) -> &'static str {
        match self {
            Command::Add { .. } => "Add a git repository to mirror.toml",
            Command::Sync { .. } => "Clone all repositories defined in mirror.toml",
            Command::Status { .. } => "Check status of all repositories defined in mirror.toml",
            Command::Save { .. } => "Commit and push changes in all repositories",
            Command::Update { .. } => "Update repositories with latest changes from remote sources",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_name() {
        let cmd = Command::Add {
            git_url: "git@github.com:example/repo.git".to_string(),
            path: "example-repo".to_string(),
            branch: None,
        };
        assert_eq!(cmd.name(), "add");

        let cmd = Command::Sync {
            config_path: None,
            dest: None,
            no_pull: false,
            force: false,
            parallel: None,
        };
        assert_eq!(cmd.name(), "sync");
    }

    #[test]
    fn test_command_description() {
        let cmd = Command::Status {
            config_path: None,
            verbose: false,
        };
        assert_eq!(
            cmd.description(),
            "Check status of all repositories defined in mirror.toml"
        );
    }
}
