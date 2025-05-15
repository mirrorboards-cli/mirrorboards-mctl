//! # Status Command Module
//!
//! This module implements the status command, which checks the status of repositories.
//! It provides information about uncommitted changes and unpushed commits.

use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use log::{info, error, debug, warn};

use crate::domain::repository::{Repository, RepositoryOperations, RepositoryStatus};
use crate::domain::configuration::Config;
use crate::application::repository_orchestrator::RepositoryOrchestrator;
use crate::presentation::output::OutputFormatter;
use crate::application::commands::Command;
use crate::presentation::cli::StatusArgs;

/// Status command implementation
pub struct StatusCommand<O, F>
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
    args: StatusArgs,
}

impl<O, F> StatusCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new status command
    pub fn new(
        repository_ops: Arc<O>,
        output_formatter: Arc<F>,
        config: Config,
        args: StatusArgs,
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
    
    /// Check repository statuses
    fn check_repository_statuses(
        &self,
        orchestrator: &RepositoryOrchestrator<O, F>,
        repositories: &[Repository]
    ) -> Result<Vec<(Repository, RepositoryStatus)>> {
        info!("Checking repository statuses...");
        let formatter = &*self.output_formatter;
        formatter.show_progress("Checking repository statuses", 0, repositories.len());
        
        // Get status for all repositories
        let summary = orchestrator.get_repository_statuses(repositories.to_vec(), self.args.include_untracked)?;
        
        // Log results
        if summary.failed > 0 {
            warn!("Failed to check status for {} repositories", summary.failed);
            
            // Generate detailed error report
            let report = orchestrator.generate_report(&summary);
            debug!("Status check operation report:\n{}", report);
            
            // Display errors to user
            let message = formatter.format_warning(
                &format!("Some repositories couldn't be checked. See logs for details.")
            );
            println!("{}", message);
            
            // Show specific errors for each failed repository
            for (repo, error) in &summary.errors {
                let error_message = formatter.format_error(
                    &format!("Failed to check status for {}: {}", repo.path.display(), error)
                );
                println!("{}", error_message);
            }
        }
        
        formatter.complete_progress(&format!("Checked status of {} repositories", summary.successful));
        
        // Return successful repository statuses
        Ok(summary.results)
    }
    
    /// Format and display repository statuses
    fn display_repository_statuses(&self, statuses: &[(Repository, RepositoryStatus)]) -> Result<()> {
        let formatter = &*self.output_formatter;
        let mut changed_count = 0;
        let mut clean_count = 0;
        
        // Sort repositories by path for consistent output
        let mut sorted_statuses = statuses.to_vec();
        sorted_statuses.sort_by(|(repo1, _), (repo2, _)| repo1.path.cmp(&repo2.path));
        
        for (repo, status) in &sorted_statuses {
            // Skip repositories with no changes if changes_only flag is set
            if self.args.changes_only && !status.has_changes && !status.has_unpushed_commits {
                clean_count += 1;
                continue;
            }
            
            // Format and display repository status
            let status_str = formatter.format_status(status);
            println!("Repository: {}\n{}\n", repo.path.display(), status_str);
            
            if status.has_changes || status.has_unpushed_commits {
                changed_count += 1;
            } else {
                clean_count += 1;
            }
        }
        
        // Print summary
        if self.args.changes_only {
            if changed_count == 0 {
                println!("{}", formatter.format_success("All repositories are clean."));
            } else {
                println!("{}", formatter.format_info(
                    &format!("{} repositories have changes, {} repositories are clean.",
                        changed_count, clean_count)
                ));
            }
        } else {
            println!("{}", formatter.format_info(
                &format!("Status: {} repositories checked, {} with changes, {} clean.",
                    sorted_statuses.len(), changed_count, clean_count)
            ));
        }
        
        Ok(())
    }
}

impl<O, F> Command for StatusCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    fn execute(&self) -> Result<()> {
        info!("Executing status command");
        debug!("Status command options: changes_only={}, include_untracked={}",
            self.args.changes_only, self.args.include_untracked);
        
        // Get repositories from configuration
        let repositories = self.filter_repositories();
        if repositories.is_empty() {
            return Err(anyhow!("No repositories found matching the specified criteria"));
        }
        
        let formatter = &*self.output_formatter;
        let info_message = formatter.format_info(
            &format!("Checking status of {} repositories", repositories.len())
        );
        println!("{}", info_message);
        
        // Create repository orchestrator
        let mut orchestrator = RepositoryOrchestrator::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter)
        );
        
        // Configure orchestrator with parallel execution
        orchestrator.configure_from_args(true, None);
        
        // Check repository statuses
        let statuses = self.check_repository_statuses(&orchestrator, &repositories)
            .context("Failed to check repository statuses")?;
        
        // Display the results
        self.display_repository_statuses(&statuses)
            .context("Failed to display repository statuses")?;
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "status"
    }
    
    fn description(&self) -> &'static str {
        "Check status of repositories"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use mockall::predicate::*;
    use mockall::*;
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
        let args = StatusArgs {
            repos: vec!["repo1".to_string()],
            changes_only: false,
            include_untracked: false,
        };
        
        // Create command with mocks
        let command = StatusCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config.clone(),
            args
        );
        
        let filtered = command.filter_repositories();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].path, PathBuf::from("/test/repo1"));
        
        // Test filtering by tag
        let args = StatusArgs {
            repos: vec!["plugin".to_string()],
            changes_only: false,
            include_untracked: false,
        };
        
        let command = StatusCommand::new(
            Arc::new(MockRepoOps::new()),
            Arc::new(MockOutputFmt::new()),
            config.clone(),
            args
        );
        
        let filtered = command.filter_repositories();
        assert_eq!(filtered.len(), 1); // Only repo2 because repo3 is disabled
        assert_eq!(filtered[0].path, PathBuf::from("/test/repo2"));
    }
    
    #[test]
    fn test_status_command_execution() {
        // Create mocks
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Set up mock expectations for status checks
        mock_repo_ops.expect_get_status()
            .times(2)
            .returning(|path| {
                // Return different statuses for different repositories
                if path == Path::new("/test/repo1") {
                    Ok(RepositoryStatus {
                        has_changes: true,
                        has_unpushed_commits: false,
                        current_branch: "main".to_string(),
                        changed_files: HashSet::new(),
                        message: None,
                    })
                } else {
                    Ok(RepositoryStatus {
                        has_changes: false,
                        has_unpushed_commits: true,
                        current_branch: "feature".to_string(),
                        changed_files: HashSet::new(),
                        message: None,
                    })
                }
            });
        
        // Set up mock expectations for output formatting
        mock_output.expect_show_progress()
            .returning(|_, _, _| ());
            
        mock_output.expect_complete_progress()
            .returning(|_| ());
            
        mock_output.expect_format_status()
            .returning(|status| {
                format!("Branch: {}, Has changes: {}, Has unpushed: {}", 
                    status.current_branch, status.has_changes, status.has_unpushed_commits)
            });
            
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
        let args = StatusArgs {
            repos: vec![],
            changes_only: false,
            include_untracked: true,
        };
        
        // Create command
        let command = StatusCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config,
            args
        );
        
        // Execute command
        let result = command.execute();
        assert!(result.is_ok()); // Should succeed with our mock setup
    }
}