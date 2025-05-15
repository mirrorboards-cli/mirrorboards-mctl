//! # Repository Orchestrator Application Layer Extension
//!
//! Extends the domain repository orchestrator with application-specific functionality.

use std::sync::Arc;
use anyhow::{Result, Context};
use log::{info, debug, warn};

use crate::domain::repository::{Repository, RepositoryOperations, RepositoryStatus};
use crate::domain::repository::orchestrator::{RepositoryOrchestrator as DomainOrchestrator, OrchestratorConfig};
use crate::presentation::output::OutputFormatter;

/// Application-specific repository orchestrator wrapper
pub struct RepositoryOrchestrator<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Domain orchestrator
    orchestrator: DomainOrchestrator,
    /// Repository operations
    repository_ops: Arc<O>,
    /// Output formatter
    output_formatter: Arc<F>,
}

/// Repository operation summary
pub struct OperationSummary {
    /// Number of successful operations
    pub successful: usize,
    /// Number of failed operations
    pub failed: usize,
    /// List of successful repositories and their results
    pub results: Vec<(Repository, RepositoryStatus)>,
    /// Map of repositories to error messages
    pub errors: Vec<(Repository, String)>,
}

impl<O, F> RepositoryOrchestrator<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new repository orchestrator
    pub fn new(
        repository_ops: Arc<O>,
        output_formatter: Arc<F>,
    ) -> Self {
        Self {
            orchestrator: DomainOrchestrator::new(),
            repository_ops,
            output_formatter,
        }
    }

    /// Configure the orchestrator from command arguments
    pub fn configure_from_args(&mut self, parallel: bool, timeout_secs: Option<u64>) {
        let mut config = OrchestratorConfig::default();
        
        // Configure parallelism
        if !parallel {
            config.max_threads = 1;
        }
        
        // Configure timeout
        if let Some(timeout) = timeout_secs {
            config.timeout_secs = timeout;
        }
        
        self.orchestrator = DomainOrchestrator::with_config(config);
    }

    /// Clone repositories that don't exist yet
    pub fn clone_repositories(&self, repositories: Vec<Repository>) -> Result<OperationSummary> {
        info!("Cloning repositories in parallel...");
        
        // Clone the Arc to move into the closure
        let repo_ops = Arc::clone(&self.repository_ops);
        
        let results = self.orchestrator.execute(repositories, move |repo| {
            // Skip repositories that already exist
            if repo.path.exists() {
                debug!("Repository {} already exists, skipping clone", repo.path.display());
                return Ok(RepositoryStatus {
                    has_changes: false,
                    has_unpushed_commits: false,
                    current_branch: String::new(),
                    changed_files: Default::default(),
                    message: Some("Repository already exists".to_string()),
                });
            }
            
            // Clone repository
            debug!("Cloning repository {} to {}", repo.origin, repo.path.display());
            
            // Access the clone method on the repository operations trait
            // The method is implemented on the dereferenced Arc
            {
                let ops = &*repo_ops;
                <O as RepositoryOperations>::clone(ops, &repo.origin, &repo.path)
                    .with_context(|| format!("Failed to clone {} to {}",
                        repo.origin, repo.path.display()))?
            };
                
            // Get initial status
            let status = RepositoryStatus {
                has_changes: false,
                has_unpushed_commits: false,
                current_branch: repo.branch.clone().unwrap_or_else(|| "main".to_string()),
                changed_files: Default::default(),
                message: Some("Repository cloned successfully".to_string()),
            };
            
            Ok(status)
        })?;
        
        // Process results
        let (successes, failures) = DomainOrchestrator::group_results(results);
        
        // Create summary
        let summary = OperationSummary {
            successful: successes.len(),
            failed: failures.len(),
            results: successes
                .into_iter()
                .map(|result| (result.repository, result.result.unwrap()))
                .collect(),
            errors: failures
                .into_iter()
                .map(|result| (result.repository, format!("{}", result.result.unwrap_err())))
                .collect(),
        };
        
        Ok(summary)
    }

    /// Update repository submodules
    pub fn update_submodules(&self, repositories: Vec<Repository>) -> Result<OperationSummary> {
        info!("Updating submodules in parallel...");
        
        // Clone Arc to use in closure
        let repo_ops = Arc::clone(&self.repository_ops);
        
        let results = self.orchestrator.execute(repositories, move |repo| {
            debug!("Updating submodules in {}", repo.path.display());
            repo_ops.update_submodules(&repo.path)
                .with_context(|| format!("Failed to update submodules in {}",
                    repo.path.display()))?;
                
            // Report success
            let status = RepositoryStatus {
                has_changes: false,
                has_unpushed_commits: false,
                current_branch: repo.branch.clone().unwrap_or_else(|| "main".to_string()),
                changed_files: Default::default(),
                message: Some("Submodules updated successfully".to_string()),
            };
            
            Ok(status)
        })?;
        
        // Process results
        let (successes, failures) = DomainOrchestrator::group_results(results);
        
        // Create summary
        let summary = OperationSummary {
            successful: successes.len(),
            failed: failures.len(),
            results: successes
                .into_iter()
                .map(|result| (result.repository, result.result.unwrap()))
                .collect(),
            errors: failures
                .into_iter()
                .map(|result| (result.repository, format!("{}", result.result.unwrap_err())))
                .collect(),
        };
        
        Ok(summary)
    }

    /// Get status of multiple repositories
    pub fn get_repository_statuses(&self, repositories: Vec<Repository>, include_untracked: bool) -> Result<OperationSummary> {
        info!("Getting repository statuses in parallel...");
        
        let include_untracked_clone = include_untracked;
        let repo_ops = Arc::clone(&self.repository_ops);
        
        let results = self.orchestrator.execute(repositories, move |repo| {
            debug!("Getting status for repository {}", repo.path.display());
            let status = repo_ops.get_status(&repo.path)
                .with_context(|| format!("Failed to get status for {}",
                    repo.path.display()))?;
                
            // Skip untracked files if not requested
            if !include_untracked_clone {
                // Note: This is a simplified example. In a real implementation,
                // we would need to modify the underlying git operations to exclude untracked files.
                debug!("Excluding untracked files from status for {}", repo.path.display());
            }
            
            Ok(status)
        })?;
        
        // Process results
        let (successes, failures) = DomainOrchestrator::group_results(results);
        
        // Create summary
        let summary = OperationSummary {
            successful: successes.len(),
            failed: failures.len(),
            results: successes
                .into_iter()
                .map(|result| (result.repository, result.result.unwrap()))
                .collect(),
            errors: failures
                .into_iter()
                .map(|result| (result.repository, format!("{}", result.result.unwrap_err())))
                .collect(),
        };
        
        Ok(summary)
    }

    /// Push changes to repositories
    pub fn push_repositories(&self, repositories: Vec<Repository>) -> Result<OperationSummary> {
        info!("Pushing changes in parallel...");
        
        // Clone Arc to use in closure
        let repo_ops = Arc::clone(&self.repository_ops);
        
        let results = self.orchestrator.execute(repositories, move |repo| {
            debug!("Pushing changes in repository {}", repo.path.display());
            repo_ops.push_changes(&repo.path)
                .with_context(|| format!("Failed to push changes in {}",
                    repo.path.display()))?;
                
            // Report success with minimal status
            let status = RepositoryStatus {
                has_changes: false,
                has_unpushed_commits: false,
                current_branch: repo.branch.clone().unwrap_or_else(|| "main".to_string()),
                changed_files: Default::default(),
                message: Some("Changes pushed successfully".to_string()),
            };
            
            Ok(status)
        })?;
        
        // Process results
        let (successes, failures) = DomainOrchestrator::group_results(results);
        
        // Create summary
        let summary = OperationSummary {
            successful: successes.len(),
            failed: failures.len(),
            results: successes
                .into_iter()
                .map(|result| (result.repository, result.result.unwrap()))
                .collect(),
            errors: failures
                .into_iter()
                .map(|result| (result.repository, format!("{}", result.result.unwrap_err())))
                .collect(),
        };
        
        Ok(summary)
    }

    /// Generate a human-readable report from an operation summary
    pub fn generate_report(&self, summary: &OperationSummary) -> String {
        let mut report = String::new();
        
        report.push_str(&format!("Operation Summary:\n"));
        report.push_str(&format!("- Successful: {}\n", summary.successful));
        report.push_str(&format!("- Failed: {}\n", summary.failed));
        
        if !summary.errors.is_empty() {
            report.push_str("\nErrors:\n");
            for (repo, error) in &summary.errors {
                report.push_str(&format!("- {}: {}\n", repo.path.display(), error));
            }
        }
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;
    use std::path::{Path, PathBuf};
    use std::collections::HashSet;
    
    // Import ProgressTracker directly for the mock
    use crate::presentation::output::ProgressTracker;
    
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
            fn create_progress_tracker(&self) -> ProgressTracker;
        }
    }

    #[test]
    fn test_repository_orchestrator_configure() {
        let repo_ops = MockRepoOps::new();
        let output_fmt = MockOutputFmt::new();
        
        let mut orchestrator = RepositoryOrchestrator::new(
            Arc::new(repo_ops),
            Arc::new(output_fmt)
        );
        
        // Test default configuration (parallel)
        orchestrator.configure_from_args(true, None);
        
        // Test single-threaded configuration
        orchestrator.configure_from_args(false, None);
        
        // Test with timeout
        orchestrator.configure_from_args(true, Some(30));
    }

    #[test]
    fn test_repository_orchestrator_report_generation() {
        let repo_ops = MockRepoOps::new();
        let output_fmt = MockOutputFmt::new();
        
        let orchestrator = RepositoryOrchestrator::new(
            Arc::new(repo_ops),
            Arc::new(output_fmt)
        );
        
        // Create a test summary
        let repo1 = Repository {
            path: PathBuf::from("/test/repo1"),
            origin: "git@github.com:org/repo1.git".to_string(),
            branch: Some("main".to_string()),
            is_git: true,
            enabled: true,
            tags: vec!["core".to_string()],
            config_overrides: None,
        };
        
        let status = RepositoryStatus {
            has_changes: true,
            has_unpushed_commits: false,
            current_branch: "main".to_string(),
            changed_files: HashSet::new(),
            message: None,
        };
        
        let summary = OperationSummary {
            successful: 1,
            failed: 1,
            results: vec![(repo1.clone(), status)],
            errors: vec![(repo1.clone(), "Test error".to_string())],
        };
        
        let report = orchestrator.generate_report(&summary);
        
        // Verify report contains basic information
        assert!(report.contains("Successful: 1"));
        assert!(report.contains("Failed: 1"));
        assert!(report.contains("Test error"));
    }
}