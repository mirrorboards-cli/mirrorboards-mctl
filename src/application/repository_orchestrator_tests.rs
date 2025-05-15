//! # Application Repository Orchestrator Tests
//!
//! Integration tests for the application-level repository orchestrator module.

#[cfg(test)]
mod tests {
    use super::super::repository_orchestrator::{RepositoryOrchestrator, RepositoryOperationSummary};
    use crate::domain::repository::{Repository, RepositoryOperations, RepositoryStatus};
    use crate::presentation::output::OutputFormatter;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use mockall::predicate::*;
    use mockall::*;
    use anyhow::{Result, anyhow};
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

    // Helper to create test repositories
    fn create_test_repos(count: usize) -> Vec<Repository> {
        (0..count).map(|i| Repository {
            path: PathBuf::from(format!("/test/repo{}", i)),
            origin: format!("git@example.com:org/repo{}.git", i),
            branch: Some(format!("main")),
            is_git: true,
            enabled: true,
            tags: vec![],
            config_overrides: None,
        }).collect()
    }

    #[test]
    fn test_parallel_clone() {
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Track which repos were cloned
        let cloned_repos = Arc::new(Mutex::new(Vec::new()));
        let cloned_repos_capture = Arc::clone(&cloned_repos);
        
        // Set up expectations for repository operations
        mock_repo_ops.expect_clone()
            .times(3)
            .returning(move |url, path| {
                let mut repos = cloned_repos_capture.lock().unwrap();
                repos.push((url.to_string(), path.to_string_lossy().to_string()));
                Ok(())
            });
        
        // Set up expectations for output formatter
        mock_output.expect_show_progress()
            .returning(|_, _, _| ());
            
        // Create the orchestrator with 2 parallel threads
        let orchestrator = RepositoryOrchestrator::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output)
        ).with_max_parallel(2);
        
        let repos = create_test_repos(3);
        let result = orchestrator.clone_repositories(repos);
        
        // Verify the operation succeeded
        assert!(result.is_ok());
        let summary = result.unwrap();
        
        // Verify all repos were processed
        assert_eq!(summary.successful, 3);
        assert_eq!(summary.failed, 0);
        
        // Verify the correct repos were cloned
        let cloned = cloned_repos.lock().unwrap();
        assert_eq!(cloned.len(), 3);
        
        // Verify URLs and paths match
        for i in 0..3 {
            let (url, path) = &cloned[i];
            assert!(url.contains(&format!("repo{}.git", i)));
            assert!(path.contains(&format!("/repo{}", i)));
        }
    }

    #[test]
    fn test_progress_reporting() {
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Count progress updates
        let progress_updates = Arc::new(Mutex::new(Vec::new()));
        let progress_updates_capture = Arc::clone(&progress_updates);
        
        // Set up repo operations expectations
        mock_repo_ops.expect_clone()
            .times(5)
            .returning(|_, _| {
                // Simulate work
                std::thread::sleep(Duration::from_millis(50));
                Ok(())
            });
        
        // Set up output formatter expectations
        mock_output.expect_show_progress()
            .returning(move |msg, current, total| {
                let mut updates = progress_updates_capture.lock().unwrap();
                updates.push((msg.to_string(), current, total));
            });
        
        // Create orchestrator and execute
        let orchestrator = RepositoryOrchestrator::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output)
        );
        
        let repos = create_test_repos(5);
        let _ = orchestrator.clone_repositories(repos);
        
        // Verify progress was reported
        let updates = progress_updates.lock().unwrap();
        assert!(!updates.is_empty());
        
        // Verify the first update shows progress starting
        let first = &updates[0];
        assert!(first.1 < first.2);
        
        // Verify the last update shows all work completed
        let last = updates.last().unwrap();
        assert_eq!(last.1, last.2);
    }

    #[test]
    fn test_check_status() {
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Create test status responses
        let status1 = RepositoryStatus {
            has_changes: true,
            has_unpushed_commits: false,
            current_branch: "main".to_string(),
            changed_files: HashSet::from(["file1.txt".to_string()]),
            message: None,
        };
        
        let status2 = RepositoryStatus {
            has_changes: false,
            has_unpushed_commits: true,
            current_branch: "feature".to_string(),
            changed_files: HashSet::new(),
            message: None,
        };
        
        // Set up expectations for repository operations
        mock_repo_ops.expect_get_status()
            .times(2)
            .returning(move |path| {
                if path.to_string_lossy().contains("repo0") {
                    Ok(status1.clone())
                } else {
                    Ok(status2.clone())
                }
            });
            
        // Set up expectations for output formatter
        mock_output.expect_show_progress()
            .returning(|_, _, _| ());
            
        let orchestrator = RepositoryOrchestrator::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output)
        );
        
        let repos = create_test_repos(2);
        let result = orchestrator.check_status(repos, true);
        
        assert!(result.is_ok());
        let summary = result.unwrap();
        
        // Verify all repos were processed
        assert_eq!(summary.successful, 2);
        assert_eq!(summary.failed, 0);
        
        // Check status results
        assert_eq!(summary.results.len(), 2);
        
        // Verify the statuses
        let has_main_branch = summary.results.iter().any(|(_, status)| status.current_branch == "main");
        let has_feature_branch = summary.results.iter().any(|(_, status)| status.current_branch == "feature");
        
        assert!(has_main_branch);
        assert!(has_feature_branch);
    }

    #[test]
    fn test_error_handling() {
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Set up expectations: some operations fail
        mock_repo_ops.expect_clone()
            .times(3)
            .returning(|url, _| {
                if url.contains("repo1") {
                    Err(anyhow!("Mock failure for repo1"))
                } else {
                    Ok(())
                }
            });
            
        // Set up expectations for output formatter
        mock_output.expect_show_progress()
            .returning(|_, _, _| ());
            
        let orchestrator = RepositoryOrchestrator::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output)
        ).with_continue_on_error(true);
        
        let repos = create_test_repos(3);
        let result = orchestrator.clone_repositories(repos);
        
        // Operation should succeed even with partial failures
        assert!(result.is_ok());
        let summary = result.unwrap();
        
        // Verify correct success/failure counts
        assert_eq!(summary.successful, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.errors.len(), 1);
        
        // Verify error details
        let (repo, error) = &summary.errors[0];
        assert!(repo.path.to_string_lossy().contains("repo1"));
        assert!(error.to_string().contains("Mock failure for repo1"));
        
        // Test report generation
        let report = orchestrator.generate_report(&summary);
        assert!(report.contains("Operation Summary"));
        assert!(report.contains("Total repositories: 3"));
        assert!(report.contains("Successful: 2"));
        assert!(report.contains("Failed: 1"));
        assert!(report.contains("Mock failure for repo1"));
    }

    #[test]
    fn test_save_changes() {
        let mut mock_repo_ops = MockRepoOps::new();
        let mut mock_output = MockOutputFmt::new();
        
        // Set up expectations for repository operations
        mock_repo_ops.expect_has_changes()
            .times(2)
            .returning(|path| {
                // Only first repo has changes
                Ok(path.to_string_lossy().contains("repo0"))
            });
            
        mock_repo_ops.expect_commit_changes()
            .times(1)
            .returning(|_, _| Ok(()));
            
        mock_repo_ops.expect_push_changes()
            .times(1)
            .returning(|_| Ok(()));
            
        // Set up expectations for output formatter
        mock_output.expect_show_progress()
            .returning(|_, _, _| ());
            
        let orchestrator = RepositoryOrchestrator::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output)
        );
        
        let repos = create_test_repos(2);
        let result = orchestrator.save_changes(repos, "Test commit message", true);
        
        assert!(result.is_ok());
        let summary = result.unwrap();
        
        // Both operations should succeed, even though only one actually committed
        assert_eq!(summary.successful, 2);
        assert_eq!(summary.failed, 0);
    }

    #[test]
    fn test_configuration() {
        let mock_repo_ops = MockRepoOps::new();
        let mock_output = MockOutputFmt::new();
        
        // Create orchestrator with custom settings
        let orchestrator = RepositoryOrchestrator::new(
            Arc::new(mock_repo_ops),
            Arc::new(mock_output)
        )
        .with_max_parallel(42)
        .with_timeout(120)
        .with_continue_on_error(false);
        
        // Use reflection to examine the private fields
        let field_value = unsafe {
            // This is only for testing - we're using unsafe code to access private fields
            let max_parallel_ptr = 
                &orchestrator as *const _ as *const usize;
            
            // The max_parallel field is at offset 2 (after repository_ops and output_formatter)
            *max_parallel_ptr.add(2)
        };
        
        // Verify max_parallel was set to 42
        assert_eq!(field_value, 42);
    }
}