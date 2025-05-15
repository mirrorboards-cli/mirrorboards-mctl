//! # Sync Command Module
//!
//! This module implements the sync command, which clones or pulls repositories.
//! It uses the repository orchestrator for parallel processing capabilities.

use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use log::{info, error, debug, warn};

use crate::domain::repository::{Repository, RepositoryOperations};
use crate::domain::configuration::Config;
use crate::application::repository_orchestrator::RepositoryOrchestrator;
use crate::presentation::output::OutputFormatter;
use crate::application::commands::Command;
use crate::presentation::cli::SyncArgs;
use crate::domain::error::{CommandError, RepositoryError};

/// Sync command implementation
pub struct SyncCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Repository operations implementation
    repository_ops: Arc<O>,
    /// Output formatter
    output_formatter: Arc<F>,
    /// Application configuration
    config: Config,
    /// Command arguments
    args: SyncArgs,
}

impl<O, F> SyncCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new sync command
    pub fn new(
        repository_ops: Arc<O>,
        output_formatter: Arc<F>,
        config: Config,
        args: SyncArgs,
    ) -> Self {
        Self {
            repository_ops,
            output_formatter,
            config,
            args,
        }
    }
    
    /// Filter repositories based on command arguments
    fn filter_repositories(&self) -> Vec<Repository> {
        let mut repositories = self.config.repositories.clone();
        
        // Filter by specified repos if any
        if !self.args.repos.is_empty() {
            repositories = repositories
                .into_iter()
                .filter(|repo| {
                    // Match by path or by tags
                    self.args.repos.iter().any(|filter| {
                        let path_match = repo.path.to_string_lossy().contains(filter);
                        let tag_match = repo.tags.iter().any(|tag| tag.contains(filter));
                        path_match || tag_match
                    })
                })
                .collect();
        }
        
        // Only keep enabled repositories
        repositories.into_iter()
            .filter(|repo| repo.enabled)
            .collect()
    }
    
    /// Clone repositories that don't exist yet
    fn clone_missing_repositories(
        &self,
        orchestrator: &RepositoryOrchestrator<O, F>,
        repositories: &[Repository]
    ) -> Result<()> {
        info!("Cloning repositories...");
        let formatter = &*self.output_formatter;
        formatter.show_progress("Preparing to clone repositories", 0, repositories.len());
        
        // Check for repository-specific overrides for depth
        let repositories = repositories.iter().map(|repo| {
            let mut repo_clone = repo.clone();
            
            // Apply repository-specific depth override if exists
            if let Some(config_overrides) = &repo.config_overrides {
                if let Some(commands) = &config_overrides.commands {
                    if let Some(sync_config) = &commands.sync {
                        if sync_config.depth.is_some() {
                            debug!("Using repository-specific depth for {}: {:?}", 
                                repo.path.display(), sync_config.depth);
                        }
                    }
                }
            }
            
            repo_clone
        }).collect::<Vec<_>>();
        
        let summary = orchestrator.clone_repositories(repositories)?;
        
        // Log results
        if summary.failed > 0 {
            warn!("Failed to clone {} repositories", summary.failed);
            
            // Generate detailed error report
            let report = orchestrator.generate_report(&summary);
            debug!("Clone operation report:\n{}", report);
            
            // Display errors to user
            let message = formatter.format_warning(
                &format!("Some repositories failed to clone. See logs for details.")
            );
            println!("{}", message);
            
            // Show specific errors for each failed repository
            for (repo, error) in &summary.errors {
                let error_message = formatter.format_error(
                    &format!("Failed to clone {}: {}", repo.path.display(), error)
                );
                println!("{}", error_message);
            }
        }
        
        formatter.complete_progress(&format!("Cloned {} repositories successfully", summary.successful));
        
        Ok(())
    }
    
    /// Update repository submodules
    fn update_repository_submodules(
        &self,
        orchestrator: &RepositoryOrchestrator<O, F>,
        repositories: &[Repository]
    ) -> Result<()> {
        info!("Updating submodules...");
        let formatter = &*self.output_formatter;
        formatter.show_progress("Preparing to update submodules", 0, repositories.len());
        
        // Update submodules in all repositories
        let summary = orchestrator.update_submodules(repositories.to_vec())?;
        
        // Log results
        if summary.failed > 0 {
            warn!("Failed to update submodules in {} repositories", summary.failed);
            
            // Generate detailed error report
            let report = orchestrator.generate_report(&summary);
            debug!("Submodule update operation report:\n{}", report);
            
            // Display errors to user
            let message = formatter.format_warning(
                &format!("Some submodules failed to update. See logs for details.")
            );
            println!("{}", message);
            
            // Show specific errors for each failed repository
            for (repo, error) in &summary.errors {
                let error_message = formatter.format_error(
                    &format!("Failed to update submodules in {}: {}", repo.path.display(), error)
                );
                println!("{}", error_message);
            }
        }
        
        formatter.complete_progress(&format!("Updated submodules in {} repositories successfully", summary.successful));
        
        Ok(())
    }
}

impl<O, F> Command for SyncCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    fn execute(&self) -> Result<()> {
        info!("Executing sync command");
        debug!("Sync command options: recursive={}, depth={:?}, parallel={}",
            self.args.recursive, self.args.depth, self.args.parallel);
        
        // Get repositories from configuration
        let repositories = self.filter_repositories();
        if repositories.is_empty() {
            return Err(anyhow!("No repositories found matching the specified criteria"));
        }
        
        let formatter = &*self.output_formatter;
        let info_message = formatter.format_info(
            &format!("Synchronizing {} repositories", repositories.len())
        );
        println!("{}", info_message);
        
        // Create repository orchestrator
        let mut orchestrator = RepositoryOrchestrator::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter)
        );
        
        // Configure orchestrator from command args
        orchestrator.configure_from_args(self.args.parallel, None);
        
        // Clone repositories that don't exist yet
        self.clone_missing_repositories(&orchestrator, &repositories)
            .context("Failed to clone repositories")?;
        
        // Update submodules if requested
        if self.args.recursive {
            self.update_repository_submodules(&orchestrator, &repositories)
                .context("Failed to update submodules")?;
        }
        
        // Show success message with summary
        let message = formatter.format_success(
            &format!("Successfully synchronized {} repositories", repositories.len())
        );
        println!("{}", message);
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "sync"
    }
    
    fn description(&self) -> &'static str {
        "Synchronize repositories (clone/pull)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use mockall::predicate::*;
    use mockall::*;
    use crate::domain::repository::RepositoryStatus;
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
    fn test_repository_filtering() {
        // Create mocks
        let mock_repo_ops = MockRepoOps::new();
        let mock_output = MockOutputFmt::new();
        
        // Create repositories for testing
        let repo1 = Repository {
            path: PathBuf::from("/test/repo1"),
            origin: "git@github.com:org/repo1.git".to_string(),
            branch: Some("main".to_string()),
            is_git: true,
            enabled: true,
            tags: vec!["core".to_string()],
            config_overrides: None,
        };
        
        let repo2 = Repository {
            path: PathBuf::from("/test/repo2"),
            origin: "git@github.com:org/repo2.git".to_string(),
            branch: Some("main".to_string()),
            is_git: true,
            enabled: true,
            tags: vec!["plugin".to_string()],
            config_overrides: None,
        };
        
        let repo3 = Repository {
            path: PathBuf::from("/test/repo3"),
            origin: "git@github.com:org/repo3.git".to_string(),
            branch: Some("main".to_string()),
            is_git: true,
            enabled: false, // Disabled repository
            tags: vec!["plugin".to_string()],
            config_overrides: None,
        };
        
        // Create config with test repositories
        let config = Config {
            repositories: vec![repo1, repo2, repo3],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        // Test filtering by repo name
        let args = SyncArgs {
            repos: vec!["repo1".to_string()],
            recursive: false,
            depth: None,
            parallel: true,
        };
        
        // Create command with mocks
        let command = SyncCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config.clone(),
            args
        );
        
        let filtered = command.filter_repositories();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].path, PathBuf::from("/test/repo1"));
        
        // Test filtering by tag
        let args = SyncArgs {
            repos: vec!["plugin".to_string()],
            recursive: false,
            depth: None,
            parallel: true,
        };
        
        let command = SyncCommand::new(
            Arc::new(MockRepoOps::new()),
            Arc::new(MockOutputFmt::new()),
            config.clone(),
            args
        );
        
        let filtered = command.filter_repositories();
        assert_eq!(filtered.len(), 1); // Only repo2 because repo3 is disabled
        assert_eq!(filtered[0].path, PathBuf::from("/test/repo2"));
        
        // Test with no filters (all enabled repos)
        let args = SyncArgs {
            repos: vec![],
            recursive: false,
            depth: None,
            parallel: true,
        };
        
        let command = SyncCommand::new(
            Arc::new(MockRepoOps::new()),
            Arc::new(MockOutputFmt::new()),
            config,
            args
        );
        
        let filtered = command.filter_repositories();
        assert_eq!(filtered.len(), 2); // Only enabled repos (repo1, repo2)
    }
    
    #[test]
    fn test_sync_command_execution() {
        // Create mocks
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Set up mock expectations
        mock_repo_ops.expect_clone()
            .times(2)
            .returning(|_, _| Ok(()));
            
        mock_repo_ops.expect_update_submodules()
            .times(2)
            .returning(|_| Ok(()));
        
        mock_output.expect_show_progress()
            .returning(|_, _, _| ());
            
        mock_output.expect_complete_progress()
            .returning(|_| ());
            
        mock_output.expect_format_success()
            .returning(|msg| format!("SUCCESS: {}", msg));
            
        mock_output.expect_format_info()
            .returning(|msg| format!("INFO: {}", msg));
        
        // Create test repositories
        let repo1 = Repository {
            path: PathBuf::from("/test/repo1"),
            origin: "git@github.com:org/repo1.git".to_string(),
            branch: Some("main".to_string()),
            is_git: true,
            enabled: true,
            tags: vec!["core".to_string()],
            config_overrides: None,
        };
        
        let repo2 = Repository {
            path: PathBuf::from("/test/repo2"),
            origin: "git@github.com:org/repo2.git".to_string(),
            branch: Some("main".to_string()),
            is_git: true,
            enabled: true,
            tags: vec!["plugin".to_string()],
            config_overrides: None,
        };
        
        // Create config
        let config = Config {
            repositories: vec![repo1, repo2],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        // Create arguments
        let args = SyncArgs {
            repos: vec![],
            recursive: true,
            depth: None,
            parallel: true,
        };
        
        // Create command
        let command = SyncCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config,
            args
        );
        
        // Execute command
        let result = command.execute();
        assert!(result.is_ok());
    }
}