//! # Command Orchestrator
//!
//! This module provides a command orchestrator for creating and executing commands.

use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use log::{info, debug, error};

use crate::domain::repository::RepositoryOperations;
use crate::domain::configuration::Config;
use crate::application::commands::Command;
use crate::application::commands::CommandFactory;
use crate::application::commands::factory::DefaultCommandFactory;
use crate::presentation::output::OutputFormatter;

/// Command orchestrator for creating and executing commands
pub struct CommandOrchestrator<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Application configuration
    config: Config,
    /// Repository operations implementation
    repository_ops: Arc<O>,
    /// Output formatter
    output_formatter: Arc<F>,
    /// Command factory
    command_factory: Box<dyn CommandFactory>,
}

impl<O, F> CommandOrchestrator<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new command orchestrator
    pub fn new(
        config: Config,
        repository_ops: Arc<O>,
        output_formatter: Arc<F>,
    ) -> Self {
        let factory = DefaultCommandFactory::new(
            Arc::clone(&repository_ops),
            Arc::clone(&output_formatter)
        );

        Self {
            config,
            repository_ops,
            output_formatter,
            command_factory: Box::new(factory),
        }
    }

    /// Execute a command by name with arguments
    pub fn execute_command(&self, command_name: String, args: Vec<String>) -> Result<()> {
        info!("Executing command: {} with args: {:?}", command_name, args);

        // Combine command name and args into a single vec for the factory
        let mut full_args = vec![command_name];
        full_args.extend(args);

        // Create the command using the factory
        let command = self.command_factory.create_command(&self.config, &full_args)
            .with_context(|| format!("Failed to create command from args: {:?}", full_args))?;

        debug!("Created command: {}", command.name());

        // Execute the command
        command.execute()
            .with_context(|| format!("Failed to execute {} command", command.name()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use std::path::{Path, PathBuf};
    use crate::domain::repository::{Repository, RepositoryStatus};
    use std::collections::HashSet;

    // Mock command factory
    mock! {
        pub CommandFactoryMock {}
        impl CommandFactory for CommandFactoryMock {
            fn create_command(&self, config: &Config, args: &[String]) -> Result<Box<dyn Command>>;
        }
    }

    // Mock command
    mock! {
        pub CommandMock {}
        impl Command for CommandMock {
            fn execute(&self) -> Result<()>;
            fn name(&self) -> &'static str;
            fn description(&self) -> &'static str;
        }
    }

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

    // Mock output formatter
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
        }
    }

    #[test]
    fn test_command_orchestrator_execution_flow() {
        // Create mock factory
        let mut mock_factory = MockCommandFactoryMock::new();
        
        // Setup expectations - factory should create a command, and command should execute
        mock_factory.expect_create_command()
            .times(1)  // Called exactly once
            .returning(|_, _| {
                // Create a command that expects to be executed
                let mut cmd = MockCommandMock::new();
                cmd.expect_name().returning(|| "test").times(1);
                cmd.expect_execute().returning(|| Ok(())).times(1);
                Ok(Box::new(cmd) as Box<dyn Command>)
            });
        
        // Create mock dependencies
        let repo_ops = Arc::new(MockRepoOps::new());
        let output_fmt = Arc::new(MockOutputFmt::new());
        
        // Create a test config
        let config = Config {
            repositories: vec![],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        // Create orchestrator with mock factory
        let mut orchestrator = CommandOrchestrator::<MockRepoOps, MockOutputFmt>::new(
            config,
            repo_ops,
            output_fmt
        );
        
        // Replace factory with our mock
        orchestrator.command_factory = Box::new(mock_factory);
        
        // Execute a command
        let result = orchestrator.execute_command("test".to_string(), vec![]);
        assert!(result.is_ok());
    }
}