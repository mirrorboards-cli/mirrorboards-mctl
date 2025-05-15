//! # Repository Orchestrator Tests
//!
//! Unit tests for the repository orchestrator module.

#[cfg(test)]
mod tests {
    use super::super::orchestrator::*;
    use crate::domain::repository::Repository;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // Helper function to create test repositories
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

    // We'll skip the direct thread pool test since ThreadPool is an internal implementation detail
    // Instead, we'll test the orchestrator's parallel execution functionality
    
    #[test]
    fn test_parallel_execution() {
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            max_threads: 4,
            timeout_secs: 0,
            continue_on_error: true,
        });
        
        let repos = create_test_repos(8);
        let concurrent_tasks = Arc::new(AtomicUsize::new(0));
        let max_concurrent = Arc::new(AtomicUsize::new(0));
        
        // We'll use counters to verify tasks are running in parallel
        // Each task will increment concurrent_tasks at start and decrement at end
        // We'll track the maximum number of concurrent tasks observed
        let concurrent_tasks_clone = Arc::clone(&concurrent_tasks);
        let max_concurrent_clone = Arc::clone(&max_concurrent);
        
        let results = orchestrator.execute(repos, move |repo| {
            let concurrent = concurrent_tasks_clone.fetch_add(1, Ordering::SeqCst) + 1;
            
            // Update max_concurrent if this is higher
            let mut current_max = max_concurrent_clone.load(Ordering::SeqCst);
            while concurrent > current_max {
                match max_concurrent_clone.compare_exchange(
                    current_max, concurrent, Ordering::SeqCst, Ordering::SeqCst
                ) {
                    Ok(_) => break,
                    Err(actual) => current_max = actual,
                }
            }
            
            // Simulate work
            thread::sleep(Duration::from_millis(50));
            
            // Mark task as done
            concurrent_tasks_clone.fetch_sub(1, Ordering::SeqCst);
            
            Ok(repo.path.to_string_lossy().to_string())
        }).unwrap();
        
        // Ensure all repositories were processed
        assert_eq!(results.len(), 8);
        
        // Verify tasks were actually running in parallel
        assert!(max_concurrent.load(Ordering::SeqCst) > 1);
    }
    
    #[test]
    fn test_progress_tracking() {
        let progress_updates = Arc::new(Mutex::new(Vec::new()));
        let progress_clone = Arc::clone(&progress_updates);
        
        // Create orchestrator with progress callback
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig::default())
            .with_progress_callback(move |info| {
                let mut updates = progress_clone.lock().unwrap();
                updates.push(ProgressInfo {
                    total: info.total,
                    completed: info.completed,
                    successful: info.successful,
                    failed: info.failed,
                    elapsed: info.elapsed,
                });
            });
        
        let repos = create_test_repos(5);
        
        let _ = orchestrator.execute(repos, |repo| {
            // Simulate work with varying durations
            let delay = match repo.path.to_string_lossy().as_ref() {
                s if s.contains("repo2") => 150, // Make one repo take longer
                _ => 50,
            };
            
            thread::sleep(Duration::from_millis(delay));
            
            Ok(repo.path.to_string_lossy().to_string())
        }).unwrap();
        
        // Verify progress was tracked
        let updates = progress_updates.lock().unwrap();
        
        // We should have some progress updates
        assert!(!updates.is_empty());
        
        // First update should show some work remaining
        assert!(updates[0].completed < updates[0].total);
        
        // Last update should show all work completed
        let last = updates.last().unwrap();
        assert_eq!(last.completed, last.total);
        assert_eq!(last.successful, last.total);
        assert_eq!(last.failed, 0);
    }
    
    #[test]
    fn test_error_handling() {
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            max_threads: 2,
            timeout_secs: 0,
            continue_on_error: true,
        });
        
        let repos = create_test_repos(4);
        
        // Simulate failures for even-numbered repositories
        let results = orchestrator.execute(repos, |repo| {
            let path_str = repo.path.to_string_lossy();
            
            // Fail for even-numbered repos
            if path_str.contains("repo0") || path_str.contains("repo2") {
                return Err(anyhow::anyhow!("Simulated failure for {}", path_str));
            }
            
            Ok(path_str.to_string())
        }).unwrap();
        
        // Group results by success/failure
        let (successes, failures) = RepositoryOrchestrator::group_results(results);
        
        // Verify the correct number of successes and failures
        assert_eq!(successes.len(), 2);
        assert_eq!(failures.len(), 2);
        
        // Verify specific repositories failed
        assert!(failures.iter().any(|r| r.repository.path.to_string_lossy().contains("repo0")));
        assert!(failures.iter().any(|r| r.repository.path.to_string_lossy().contains("repo2")));
    }
    
    #[test]
    fn test_timeout_handling() {
        // Create orchestrator with a very short timeout
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            max_threads: 2,
            timeout_secs: 1, // 1 second timeout
            continue_on_error: true,
        });
        
        let repos = create_test_repos(4);
        
        // Create a long-running task that should time out
        let result = orchestrator.execute(repos, |_| {
            // Sleep longer than the timeout
            thread::sleep(Duration::from_secs(3));
            Ok(())
        });
        
        // Verify the operation timed out
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_string = err.to_string();
        assert!(err_string.contains("timed out"));
    }
    
    #[test]
    fn test_all_operations_failed() {
        // Create orchestrator that should fail if all operations fail
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            max_threads: 2,
            timeout_secs: 0,
            continue_on_error: false, // Don't continue on error
        });
        
        let repos = create_test_repos(3);
        
        // Make all operations fail
        let result = orchestrator.execute(repos, |_| {
            Err(anyhow::anyhow!("Simulated failure"))
        });
        
        // Verify an error is returned
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_string = err.to_string();
        assert!(err_string.contains("All operations failed"));
    }
    
    #[test]
    fn test_execute_and_collect() {
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            max_threads: 2,
            timeout_secs: 0,
            continue_on_error: true,
        });
        
        let repos = create_test_repos(5);
        
        // Return the repository path length for each successful operation
        let result = orchestrator.execute_and_collect(repos, |repo| {
            let path_str = repo.path.to_string_lossy();
            
            // Fail for one specific repo
            if path_str.contains("repo3") {
                return Err(anyhow::anyhow!("Simulated failure"));
            }
            
            Ok(path_str.len())
        }).unwrap();
        
        // Verify we got the correct number of results
        assert_eq!(result.len(), 4); // 5 total - 1 failure
        
        // All path lengths should be the same in our test data
        let expected_len = "/test/repo0".len();
        for len in result {
            assert_eq!(len, expected_len);
        }
    }
}