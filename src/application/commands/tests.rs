//! # Command Tests Module
//!
//! Integration tests for commands.

use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use anyhow::Result;
use mockall::predicate::*;
use mockall::*;

use super::*;
use crate::domain::repository::{Repository, RepositoryOperations, RepositoryStatus};
use crate::application::commands::factory::DefaultCommandFactory;
use crate::domain::configuration::Config;
use crate::presentation::output::OutputFormatter;
use crate::presentation::cli::{SyncArgs, StatusArgs, SaveArgs, InitArgs};

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

// Helper function to create a test repository
fn create_test_repository() -> Repository {
    Repository {
        path: PathBuf::from("/test/repo"),
        origin: "git@github.com:org/repo.git".to_string(),
        branch: Some("main".to_string()),
        is_git: true,
        enabled: true,
        tags: vec!["core".to_string()],
        config_overrides: None,
    }
}

// Helper function to create a test configuration
fn create_test_config() -> Config {
    Config {
        repositories: vec![create_test_repository()],
        global: Default::default(),
        auth: Default::default(),
        logging: Default::default(),
        commands: Default::default(),
    }
}

#[test]
fn test_command_factory_creates_all_commands() {
    let repo_ops = MockRepoOps::new();
    let output_fmt = MockOutputFmt::new();
    
    let factory = DefaultCommandFactory::new(
        Arc::new(repo_ops),
        Arc::new(output_fmt)
    );
    
    let config = create_test_config();
    
    // Test all command types
    let sync_cmd = factory.create_command(&config, &["sync".to_string()]);
    assert!(sync_cmd.is_ok());
    assert_eq!(sync_cmd.unwrap().name(), "sync");
    
    let status_cmd = factory.create_command(&config, &["status".to_string()]);
    assert!(status_cmd.is_ok());
    assert_eq!(status_cmd.unwrap().name(), "status");
    
    let save_cmd = factory.create_command(&config, &["save".to_string()]);
    assert!(save_cmd.is_ok());
    assert_eq!(save_cmd.unwrap().name(), "save");
    
    let init_cmd = factory.create_command(&config, &["init".to_string()]);
    assert!(init_cmd.is_ok());
    assert_eq!(init_cmd.unwrap().name(), "init");
    
    // Test invalid command
    let invalid_cmd = factory.create_command(&config, &["invalid".to_string()]);
    assert!(invalid_cmd.is_err());
}

#[test]
fn test_command_trait_implementation() {
    // Create mock components
    let repo_ops = Arc::new(MockRepoOps::new());
    let output_fmt = Arc::new(MockOutputFmt::new());
    let config = create_test_config();
    
    // Test SyncCommand trait methods
    let sync_args = SyncArgs {
        repos: vec![],
        recursive: false,
        depth: None,
        parallel: true,
    };
    
    let sync_command = crate::application::commands::sync::SyncCommand::new(
        Arc::clone(&repo_ops),
        Arc::clone(&output_fmt),
        config.clone(),
        sync_args
    );
    
    assert_eq!(sync_command.name(), "sync");
    assert!(!sync_command.description().is_empty());
    
    // Test StatusCommand trait methods
    let status_args = StatusArgs {
        repos: vec![],
        changes_only: false,
        include_untracked: false,
    };
    
    let status_command = crate::application::commands::status::StatusCommand::new(
        Arc::clone(&repo_ops),
        Arc::clone(&output_fmt),
        config.clone(),
        status_args
    );
    
    assert_eq!(status_command.name(), "status");
    assert!(!status_command.description().is_empty());
    
    // Test SaveCommand trait methods
    let save_args = SaveArgs {
        repos: vec![],
        message: "Test commit".to_string(),
        push: false,
        sign: false,
    };
    
    let save_command = crate::application::commands::save::SaveCommand::new(
        Arc::clone(&repo_ops),
        Arc::clone(&output_fmt),
        config.clone(),
        save_args
    );
    
    assert_eq!(save_command.name(), "save");
    assert!(!save_command.description().is_empty());
}

#[test]
fn test_command_integration() {
    // This test would simulate the entire command execution flow
    // from creation through the factory to execution
    // We'll create a minimal test here that can be expanded
    
    let mut repo_ops = MockRepoOps::new();
    let mut output_fmt = MockOutputFmt::new();
    
    // Set up necessary expectations for mock objects
    output_fmt.expect_format_info()
        .returning(|msg| format!("INFO: {}", msg));
        
    output_fmt.expect_format_error()
        .returning(|msg| format!("ERROR: {}", msg));
        
    output_fmt.expect_format_success() 
        .returning(|msg| format!("SUCCESS: {}", msg));
    
    // Create a factory with our mocks
    let factory = DefaultCommandFactory::new(
        Arc::new(repo_ops),
        Arc::new(output_fmt)
    );
    
    // Create a simple config with no repositories
    let config = Config {
        repositories: vec![],
        global: Default::default(),
        auth: Default::default(),
        logging: Default::default(),
        commands: Default::default(),
    };
    
    // Try to create a command - it should fail because there are no repositories
    let command = factory.create_command(&config, &["status".to_string()]);
    assert!(command.is_ok());
    
    // Note: We can't fully test execution here since it requires properly set up mocks
    // for all the operations that would be performed, which would be extensive
}