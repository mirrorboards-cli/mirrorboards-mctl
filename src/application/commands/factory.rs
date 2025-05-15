//! # Command Factory Module
//!
//! This module provides a factory for creating command instances.

use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use log::{info, debug};

use crate::application::commands::{Command, CommandFactory};
use crate::application::commands::sync::SyncCommand;
use crate::application::commands::status::StatusCommand;
use crate::application::commands::save::SaveCommand;
use crate::application::commands::init::InitCommand;
use crate::domain::configuration::Config;
use crate::domain::repository::RepositoryOperations;
use crate::presentation::output::OutputFormatter;
use crate::presentation::cli::{SyncArgs, StatusArgs, SaveArgs, InitArgs};

/// Command factory implementation
pub struct DefaultCommandFactory<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Repository operations implementation
    repository_ops: Arc<O>,
    /// Output formatter
    output_formatter: Arc<F>,
}

impl<O, F> DefaultCommandFactory<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new command factory
    pub fn new(repository_ops: Arc<O>, output_formatter: Arc<F>) -> Self {
        Self {
            repository_ops,
            output_formatter,
        }
    }

    /// Create a sync command
    pub fn create_sync_command(&self, config: Config, args: SyncArgs) -> Box<dyn Command> {
        Box::new(SyncCommand::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter),
            config,
            args,
        ))
    }

    /// Create a status command
    pub fn create_status_command(&self, config: Config, args: StatusArgs) -> Box<dyn Command> {
        Box::new(StatusCommand::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter),
            config,
            args,
        ))
    }

    /// Create a save command
    pub fn create_save_command(&self, config: Config, args: SaveArgs) -> Box<dyn Command> {
        Box::new(SaveCommand::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter),
            config,
            args,
        ))
    }

    /// Create an init command
    pub fn create_init_command(&self, config: Config, args: InitArgs) -> Box<dyn Command> {
        Box::new(InitCommand::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter),
            config,
            args,
        ))
    }
}

impl<O, F> CommandFactory for DefaultCommandFactory<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    fn create_command(&self, config: &Config, args: &[String]) -> Result<Box<dyn Command>> {
        if args.is_empty() {
            return Err(anyhow!("No command specified"));
        }

        let command_name = &args[0];
        let command_args = &args[1..];

        match command_name.as_str() {
            "sync" => {
                debug!("Creating sync command with args: {:?}", command_args);
                let args = match SyncArgs::from_args(command_args) {
                    Ok(args) => args,
                    Err(e) => return Err(anyhow!("Failed to parse sync command arguments: {}", e)),
                };
                
                Ok(self.create_sync_command(config.clone(), args))
            },
            "status" => {
                debug!("Creating status command with args: {:?}", command_args);
                let args = match StatusArgs::from_args(command_args) {
                    Ok(args) => args,
                    Err(e) => return Err(anyhow!("Failed to parse status command arguments: {}", e)),
                };
                
                Ok(self.create_status_command(config.clone(), args))
            },
            "save" => {
                debug!("Creating save command with args: {:?}", command_args);
                let args = match SaveArgs::from_args(command_args) {
                    Ok(args) => args,
                    Err(e) => return Err(anyhow!("Failed to parse save command arguments: {}", e)),
                };
                
                Ok(self.create_save_command(config.clone(), args))
            },
            "init" => {
                debug!("Creating init command with args: {:?}", command_args);
                let args = match InitArgs::from_args(command_args) {
                    Ok(args) => args,
                    Err(e) => return Err(anyhow!("Failed to parse init command arguments: {}", e)),
                };
                
                Ok(self.create_init_command(config.clone(), args))
            },
            _ => Err(anyhow!("Unknown command: {}", command_name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use crate::domain::repository::{RepositoryStatus, Repository};
    use crate::presentation::output::{OutputFormatter, ProgressTracker};
    use std::path::{Path, PathBuf};
    use std::collections::HashSet;

    // Mock repository operations
    mock! {
        pub RepoOps {}
        impl RepositoryOperations for RepoOps {
            fn clone(&self, url: &str, path: &Path) -> Result<()>;
            fn update_submodules(&self, path: &Path) -> Result<()>;
            fn has_changes(&self, path: &Path) -> Result<bool>;
            fn commit_changes(&self, path: &Path, message: &str) -> Result<()>;
            fn push_changes(&self, path: &Path) -> Result<()>;
            fn get_status(&self, path: &Path) -> Result<RepositoryStatus>;
            fn get_remote_url(&self, path: &Path) -> Result<String>;
        }
    }

    // Mock output formatter with explicit mock structure
    mock! {
        pub OutputFmt {}
        impl OutputFormatter for OutputFmt {
            fn format_status(&self, status: &RepositoryStatus) -> String;
            fn format_error(&self, error: &str) -> String;
            fn format_success(&self, message: &str) -> String;
            fn format_info(&self, message: &str) -> String;
            fn format_warning(&self, message: &str) -> String;
            fn show_progress(&self, message: &str, current: usize, total: usize);
            fn complete_progress(&self, message: &str);
            fn create_progress_tracker(&self) -> crate::presentation::output::ProgressTracker;
        }
    }

    #[test]
    fn test_command_factory_create_sync() {
        let repo_ops = MockRepoOps::new();
        let output_fmt = MockOutputFmt::new();
        
        let factory = DefaultCommandFactory::new(
            Arc::new(repo_ops),
            Arc::new(output_fmt)
        );
        
        let config = Config {
            repositories: vec![],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        let command = factory.create_command(&config, &["sync".to_string()]);
        assert!(command.is_ok());
        assert_eq!(command.unwrap().name(), "sync");
    }

    #[test]
    fn test_command_factory_create_status() {
        let repo_ops = MockRepoOps::new();
        let output_fmt = MockOutputFmt::new();
        
        let factory = DefaultCommandFactory::new(
            Arc::new(repo_ops),
            Arc::new(output_fmt)
        );
        
        let config = Config {
            repositories: vec![],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        let command = factory.create_command(&config, &["status".to_string()]);
        assert!(command.is_ok());
        assert_eq!(command.unwrap().name(), "status");
    }

    #[test]
    fn test_command_factory_create_save() {
        let repo_ops = MockRepoOps::new();
        let output_fmt = MockOutputFmt::new();
        
        let factory = DefaultCommandFactory::new(
            Arc::new(repo_ops),
            Arc::new(output_fmt)
        );
        
        let config = Config {
            repositories: vec![],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        let command = factory.create_command(&config, &["save".to_string()]);
        assert!(command.is_ok());
        assert_eq!(command.unwrap().name(), "save");
    }

    #[test]
    fn test_command_factory_unknown_command() {
        let repo_ops = MockRepoOps::new();
        let output_fmt = MockOutputFmt::new();
        
        let factory = DefaultCommandFactory::new(
            Arc::new(repo_ops),
            Arc::new(output_fmt)
        );
        
        let config = Config {
            repositories: vec![],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        let command = factory.create_command(&config, &["unknown".to_string()]);
        assert!(command.is_err());
    }
}