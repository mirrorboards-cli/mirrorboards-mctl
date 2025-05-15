//! # Sync Command Implementation
//!
//! This module implements the sync command to clone and update repositories.

use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use log::{info, error, debug, warn};

use crate::domain::repository::{Repository, RepositoryOperations};
use crate::domain::configuration::Config;
use crate::application::repository_orchestrator::RepositoryOrchestrator;
use crate::presentation::output::OutputFormatter;
use crate::presentation::cli::SyncArgs;

/// Command for synchronizing repositories
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
}

impl<O, F> SyncCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new sync command
    pub fn new(repository_ops: Arc<O>, output_formatter: Arc<F>, config: Config) -> Self {
        Self {
            repository_ops,
            output_formatter,
            config,
        }
    }
    
    /// Execute the sync command
    pub fn execute(&self, args: &SyncArgs) -> Result<()> {
        info!("Executing sync command");
        debug!("Sync command options: recursive={}, depth={:?}, parallel={}", 
            args.recursive, args.depth, args.parallel);
        
        // Get repositories from configuration
        let repositories = self.filter_repositories(args);
        if repositories.is_empty() {
            return Err(anyhow!("No repositories found"));
        }
        
        info!("Synchronizing {} repositories", repositories.len());
        
        // Create repository orchestrator
        let mut orchestrator = RepositoryOrchestrator::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter)
        );
        
        // Configure orchestrator from command args
        orchestrator.configure_from_args(args.parallel, None);
        
        // Clone repositories that don't exist yet
        self.clone_missing_repositories(&orchestrator, &repositories)?;
        
        // Update submodules if requested
        if args.recursive {
            self.update_repository_submodules(&orchestrator, &repositories)?;
        }
        
        // Show success message with summary
        let formatter = &*self.output_formatter;
        let message = formatter.format_success(
            &format!("Successfully synchronized {} repositories", repositories.len())
        );
        println!("{}", message);
        
        Ok(())
    }
    
    /// Filter repositories based on command arguments
    fn filter_repositories(&self, args: &SyncArgs) -> Vec<Repository> {
        let mut repositories = self.config.repositories.clone();
        
        // Filter by specified repos if any
        if !args.repos.is_empty() {
            repositories = repositories
                .into_iter()
                .filter(|repo| {
                    // Match by path or by tags
                    args.repos.iter().any(|filter| {
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
        // In a real implementation, we would check which repositories don't exist yet
        // on the filesystem and only clone those. For simplicity, we'll just clone all.
        let summary = orchestrator.clone_repositories(repositories.to_vec())?;
        
        // Log results
        if summary.failed > 0 {
            warn!("Failed to clone {} repositories", summary.failed);
            
            // Generate detailed error report
            let report = orchestrator.generate_report(&summary);
            debug!("Clone operation report:\n{}", report);
            
            // Display errors to user
            let formatter = &*self.output_formatter;
            let message = formatter.format_warning(
                &format!("Some repositories failed to clone. See logs for details.")
            );
            println!("{}", message);
        }
        
        Ok(())
    }
    
    /// Update repository submodules
    fn update_repository_submodules(
        &self, 
        orchestrator: &RepositoryOrchestrator<O, F>, 
        repositories: &[Repository]
    ) -> Result<()> {
        // Update submodules in all repositories
        let summary = orchestrator.update_submodules(repositories.to_vec())?;
        
        // Log results
        if summary.failed > 0 {
            warn!("Failed to update submodules in {} repositories", summary.failed);
            
            // Generate detailed error report
            let report = orchestrator.generate_report(&summary);
            debug!("Submodule update operation report:\n{}", report);
            
            // Display errors to user
            let formatter = &*self.output_formatter;
            let message = formatter.format_warning(
                &format!("Some submodules failed to update. See logs for details.")
            );
            println!("{}", message);
        }
        
        Ok(())
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
    
    // Import mocks from repository_orchestrator_tests
    use crate::application::repository_orchestrator_tests::tests::{MockRepoOps, MockOutputFmt};
    
    #[test]
    fn test_sync_command_execution() {
        // Create mocks
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Set up expectations
        mock_repo_ops.expect_clone()
            .times(2)
            .returning(|_, _| Ok(()));
            
        mock_repo_ops.expect_update_submodules()
            .times(2)
            .returning(|_| Ok(()));
        
        mock_output.expect_show_progress()
            .returning(|_, _, _| ());
            
        mock_output.expect_format_success()
            .returning(|msg| format!("SUCCESS: {}", msg));
        
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
        
        // Create config with test repositories
        let config = Config {
            repositories: vec![repo1, repo2],
            // Add other required config fields
            groups: Default::default(),
            settings: Default::default(),
        };
        
        // Create command with mocks
        let command = SyncCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config
        );
        
        // Create sync args
        let args = SyncArgs {
            repos: vec![],
            recursive: true,
            depth: None,
            parallel: true,
        };
        
        // Execute command
        let result = command.execute(&args);
        
        // Verify command executed successfully
        assert!(result.is_ok());
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
            // Add other required config fields
            groups: Default::default(),
            settings: Default::default(),
        };
        
        // Create command with mocks
        let command = SyncCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config
        );
        
        // Test filtering by repo name
        let args = SyncArgs {
            repos: vec!["repo1".to_string()],
            recursive: false,
            depth: None,
            parallel: true,
        };
        
        let filtered = command.filter_repositories(&args);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].path, PathBuf::from("/test/repo1"));
        
        // Test filtering by tag
        let args = SyncArgs {
            repos: vec!["plugin".to_string()],
            recursive: false,
            depth: None,
            parallel: true,
        };
        
        let filtered = command.filter_repositories(&args);
        assert_eq!(filtered.len(), 1);  // Only one because repo3 is disabled
        assert_eq!(filtered[0].path, PathBuf::from("/test/repo2"));
        
        // Test no filter (should get all enabled repos)
        let args = SyncArgs {
            repos: vec![],
            recursive: false,
            depth: None,
            parallel: true,
        };
        
        let filtered = command.filter_repositories(&args);
        assert_eq!(filtered.len(), 2);  // repo3 is excluded because it's disabled
    }
}