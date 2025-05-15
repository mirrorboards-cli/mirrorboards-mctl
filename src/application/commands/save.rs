//! # Save Command Module
//!
//! This module implements the save command, which commits and optionally pushes changes.
//! It supports automatic commit message generation based on repository status.

use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use log::{info, error, debug, warn};
use std::collections::HashSet;

use crate::domain::repository::{Repository, RepositoryOperations, RepositoryStatus};
use crate::domain::configuration::Config;
use crate::application::repository_orchestrator::RepositoryOrchestrator;
use crate::presentation::output::OutputFormatter;
use crate::application::commands::Command;
use crate::presentation::cli::SaveArgs;

/// Save command implementation
pub struct SaveCommand<O, F>
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
    args: SaveArgs,
}

impl<O, F> SaveCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new save command
    pub fn new(
        repository_ops: Arc<O>,
        output_formatter: Arc<F>,
        config: Config,
        args: SaveArgs,
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
    
    /// Find repositories with changes
    fn find_repositories_with_changes(
        &self,
        orchestrator: &RepositoryOrchestrator<O, F>,
        repositories: &[Repository]
    ) -> Result<Vec<(Repository, RepositoryStatus)>> {
        info!("Checking for repositories with changes...");
        let formatter = &*self.output_formatter;
        formatter.show_progress("Checking repository statuses", 0, repositories.len());
        
        // Get status for all repositories
        let summary = orchestrator.get_repository_statuses(repositories.to_vec(), true)?;
        
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
        
        // Filter out repositories with no changes
        let repositories_with_changes = summary.results
            .into_iter()
            .filter(|(_, status)| status.has_changes)
            .collect::<Vec<_>>();
        
        info!("Found {} repositories with changes", repositories_with_changes.len());
        
        Ok(repositories_with_changes)
    }
    
    /// Generate a descriptive commit message
    fn generate_commit_message(&self, repository: &Repository, status: &RepositoryStatus) -> String {
        // If a custom message is provided, use it
        if self.args.message != "Auto-commit by MCTL" {
            return self.args.message.clone();
        }
        
        // Otherwise, generate a descriptive message based on changed files
        let mut message = String::from("Update");
        
        // Add repository name to commit message
        if let Some(repo_name) = repository.path.file_name() {
            message.push_str(&format!(" {}", repo_name.to_string_lossy()));
        }
        
        // If there are changed files, list them (up to a reasonable limit)
        if !status.changed_files.is_empty() {
            message.push_str(":\n\n");
            
            let file_list = status.changed_files.iter()
                .take(10)  // Limit to 10 files to avoid massive commit messages
                .map(|file| format!("- {}", file))
                .collect::<Vec<_>>()
                .join("\n");
            
            message.push_str(&file_list);
            
            if status.changed_files.len() > 10 {
                message.push_str(&format!("\n\n... and {} more files", status.changed_files.len() - 10));
            }
        }
        
        message
    }
    
    /// Commit changes to repositories
    fn commit_changes(
        &self,
        orchestrator: &RepositoryOrchestrator<O, F>,
        repositories_with_status: &[(Repository, RepositoryStatus)]
    ) -> Result<Vec<Repository>> {
        if repositories_with_status.is_empty() {
            info!("No repositories with changes found. Nothing to commit.");
            return Ok(Vec::new());
        }
        
        info!("Committing changes in {} repositories...", repositories_with_status.len());
        let formatter = &*self.output_formatter;
        formatter.show_progress("Committing changes", 0, repositories_with_status.len());
        
        let mut committed_repos = Vec::new();
        let mut failed_count = 0;
        
        // Process each repository with changes
        for (idx, (repo, status)) in repositories_with_status.iter().enumerate() {
            formatter.show_progress("Committing changes", idx, repositories_with_status.len());
            
            // Generate or use provided commit message
            let commit_message = self.generate_commit_message(repo, status);
            
            // Commit changes
            match self.repository_ops.commit_changes(&repo.path, &commit_message) {
                Ok(_) => {
                    debug!("Successfully committed changes in {}", repo.path.display());
                    committed_repos.push(repo.clone());
                }
                Err(err) => {
                    error!("Failed to commit changes in {}: {}", repo.path.display(), err);
                    failed_count += 1;
                    
                    let error_message = formatter.format_error(
                        &format!("Failed to commit changes in {}: {}", repo.path.display(), err)
                    );
                    println!("{}", error_message);
                }
            }
        }
        
        // Complete progress and show summary
        formatter.complete_progress(&format!("Committed changes in {} repositories", committed_repos.len()));
        
        if failed_count > 0 {
            let warning = formatter.format_warning(
                &format!("Failed to commit changes in {} repositories", failed_count)
            );
            println!("{}", warning);
        }
        
        Ok(committed_repos)
    }
    
    /// Push changes to remote
    fn push_changes(
        &self,
        orchestrator: &RepositoryOrchestrator<O, F>,
        repositories: &[Repository]
    ) -> Result<()> {
        if repositories.is_empty() {
            info!("No repositories committed. Nothing to push.");
            return Ok(());
        }
        
        info!("Pushing changes in {} repositories...", repositories.len());
        let formatter = &*self.output_formatter;
        formatter.show_progress("Pushing changes", 0, repositories.len());
        
        // Push changes in all committed repositories
        let summary = orchestrator.push_repositories(repositories.to_vec())?;
        
        // Log results
        if summary.failed > 0 {
            warn!("Failed to push changes in {} repositories", summary.failed);
            
            // Generate detailed error report
            let report = orchestrator.generate_report(&summary);
            debug!("Push operation report:\n{}", report);
            
            // Display errors to user
            let message = formatter.format_warning(
                &format!("Some changes couldn't be pushed. See logs for details.")
            );
            println!("{}", message);
            
            // Show specific errors for each failed repository
            for (repo, error) in &summary.errors {
                let error_message = formatter.format_error(
                    &format!("Failed to push changes in {}: {}", repo.path.display(), error)
                );
                println!("{}", error_message);
            }
        }
        
        formatter.complete_progress(&format!("Pushed changes in {} repositories", summary.successful));
        
        Ok(())
    }
}

impl<O, F> Command for SaveCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    fn execute(&self) -> Result<()> {
        info!("Executing save command");
        debug!("Save command options: message='{}', push={}, sign={}",
            self.args.message, self.args.push, self.args.sign);
        
        // Get repositories from configuration
        let repositories = self.filter_repositories();
        if repositories.is_empty() {
            return Err(anyhow!("No repositories found matching the specified criteria"));
        }
        
        let formatter = &*self.output_formatter;
        let info_message = formatter.format_info(
            &format!("Processing {} repositories for changes", repositories.len())
        );
        println!("{}", info_message);
        
        // Create repository orchestrator
        let mut orchestrator = RepositoryOrchestrator::new(
            Arc::clone(&self.repository_ops),
            Arc::clone(&self.output_formatter)
        );
        
        // Configure orchestrator with parallel execution
        orchestrator.configure_from_args(true, None);
        
        // Find repositories with changes
        let repos_with_changes = self.find_repositories_with_changes(&orchestrator, &repositories)
            .context("Failed to find repositories with changes")?;
        
        if repos_with_changes.is_empty() {
            println!("{}", formatter.format_success("No changes found in any repositories."));
            return Ok(());
        }
        
        // Commit changes
        let committed_repos = self.commit_changes(&orchestrator, &repos_with_changes)
            .context("Failed to commit changes")?;
        
        // Push changes if requested
        if self.args.push && !committed_repos.is_empty() {
            self.push_changes(&orchestrator, &committed_repos)
                .context("Failed to push changes")?;
        }
        
        // Show success message with summary
        let message = formatter.format_success(
            &format!("Successfully saved changes in {} repositories", committed_repos.len())
        );
        println!("{}", message);
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "save"
    }
    
    fn description(&self) -> &'static str {
        "Save changes (commit/push)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use mockall::predicate::*;
    use mockall::*;
    
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
        let args = SaveArgs {
            repos: vec!["repo1".to_string()],
            message: "Test commit".to_string(),
            push: false,
            sign: false,
        };
        
        // Create command with mocks
        let command = SaveCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config.clone(),
            args
        );
        
        let filtered = command.filter_repositories();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].path, PathBuf::from("/test/repo1"));
    }
    
    // Create test repository for use in tests
    fn create_test_repo() -> Repository {
        Repository {
            path: PathBuf::from("/test/my-project"),
            origin: "git@github.com:org/my-project.git".to_string(),
            branch: Some("main".to_string()),
            is_git: true,
            enabled: true,
            tags: vec!["core".to_string()],
            config_overrides: None,
        }
    }
    
    // Create test status for use in tests
    fn create_test_status() -> RepositoryStatus {
        let mut changed_files = HashSet::new();
        changed_files.insert("src/main.rs".to_string());
        changed_files.insert("Cargo.toml".to_string());
        
        RepositoryStatus {
            has_changes: true,
            has_unpushed_commits: false,
            current_branch: "main".to_string(),
            changed_files,
            message: None,
        }
    }
    
    #[test]
    fn test_commit_message_generation_auto() {
        // Create mocks
        let mock_repo_ops = MockRepoOps::new();
        let mock_output = MockOutputFmt::new();
        
        // Get test data
        let repo = create_test_repo();
        let status = create_test_status();
        
        // Test with default auto-commit message
        let args = SaveArgs {
            repos: vec![],
            message: "Auto-commit by MCTL".to_string(), // Default message
            push: true,
            sign: false,
        };
        
        let config = Config {
            repositories: vec![repo.clone()],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        let command = SaveCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config,
            args
        );
        
        let message = command.generate_commit_message(&repo, &status);
        assert!(message.contains("Update my-project"));
        assert!(message.contains("src/main.rs"));
        assert!(message.contains("Cargo.toml"));
    }
    
    #[test]
    fn test_commit_message_generation_custom() {
        // Create mocks
        let mock_repo_ops = MockRepoOps::new();
        let mock_output = MockOutputFmt::new();
        
        // Get test data
        let repo = create_test_repo();
        let status = create_test_status();
        
        // Test with custom message
        let args = SaveArgs {
            repos: vec![],
            message: "Custom commit message".to_string(),
            push: true,
            sign: false,
        };
        
        let config = Config {
            repositories: vec![repo.clone()],
            global: Default::default(),
            auth: Default::default(),
            logging: Default::default(),
            commands: Default::default(),
        };
        
        let command = SaveCommand::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output),
            config,
            args
        );
        
        let message = command.generate_commit_message(&repo, &status);
        assert_eq!(message, "Custom commit message");
    }
}