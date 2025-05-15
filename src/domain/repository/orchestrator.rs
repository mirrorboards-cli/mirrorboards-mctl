//! # Repository Orchestrator
//!
//! This module provides parallel processing capabilities for repository operations.
//! It allows executing operations across multiple repositories concurrently while tracking
//! progress and collecting results.

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};
use std::sync::mpsc;

use anyhow::{Result, Context, anyhow};
use thiserror::Error;

use crate::domain::repository::{Repository, RepositoryOperations};
use crate::domain::error::{RepositoryError, CommandError};

/// Error type for repository orchestration operations
#[derive(Debug, Error)]
pub enum OrchestratorError {
    /// Error when a parallel operation times out
    #[error("Operation timed out after {timeout_secs} seconds")]
    OperationTimeout {
        /// Timeout in seconds
        timeout_secs: u64,
    },
    
    /// Error when a thread panics during parallel execution
    #[error("Thread panic during parallel execution: {message}")]
    ThreadPanic {
        /// Panic message
        message: String,
    },
    
    /// Error when all operations fail
    #[error("All operations failed: {message}")]
    AllOperationsFailed {
        /// Summary message
        message: String,
    },
}

/// Result of an operation on a repository
#[derive(Debug)]
pub struct OperationResult<T> {
    /// Repository reference
    pub repository: Repository,
    /// Operation result
    pub result: Result<T>,
    /// Duration of the operation
    pub duration: Duration,
}

/// Progress information for repository operations
#[derive(Debug, Clone)]
pub struct ProgressInfo {
    /// Total number of repositories
    pub total: usize,
    /// Number of completed operations
    pub completed: usize,
    /// Number of successful operations
    pub successful: usize,
    /// Number of failed operations
    pub failed: usize,
    /// Total elapsed time
    pub elapsed: Duration,
}

/// Progress callback function type
pub type ProgressCallback = Box<dyn Fn(&ProgressInfo) + Send + Sync>;

/// Configuration for the repository orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum number of threads to use
    pub max_threads: usize,
    /// Operation timeout in seconds (0 means no timeout)
    pub timeout_secs: u64,
    /// Whether to continue on errors
    pub continue_on_error: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_threads: num_cpus::get(),
            timeout_secs: 0, // No timeout by default
            continue_on_error: true,
        }
    }
}

/// Repository orchestrator for coordinating parallel repository operations
pub struct RepositoryOrchestrator {
    /// Configuration
    config: OrchestratorConfig,
    /// Progress callback
    progress_callback: Option<ProgressCallback>,
}

impl RepositoryOrchestrator {
    /// Create a new repository orchestrator with default configuration
    pub fn new() -> Self {
        Self {
            config: OrchestratorConfig::default(),
            progress_callback: None,
        }
    }

    /// Create a new repository orchestrator with the given configuration
    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self {
            config,
            progress_callback: None,
        }
    }

    /// Set a progress callback function
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ProgressInfo) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Execute an operation on multiple repositories in parallel
    ///
    /// # Arguments
    ///
    /// * `repositories` - List of repositories to process
    /// * `operation` - Function that performs the operation on a single repository
    ///
    /// # Returns
    ///
    /// Vector of operation results, one per repository
    pub fn execute<F, T>(&self, repositories: Vec<Repository>, operation: F) -> Result<Vec<OperationResult<T>>>
    where
        F: Fn(&Repository) -> Result<T> + Send + Sync + 'static,
        T: Send + 'static,
    {
        if repositories.is_empty() {
            return Ok(Vec::new());
        }

        let total_repos = repositories.len();
        let thread_count = std::cmp::min(self.config.max_threads, total_repos);
        
        // Shared state for tracking progress
        let completed = Arc::new(AtomicUsize::new(0));
        let successful = Arc::new(AtomicUsize::new(0));
        let failed = Arc::new(AtomicUsize::new(0));
        let start_time = Instant::now();
        
        // Channel for collecting results
        let (tx, rx) = mpsc::channel();
        
        // Create thread pool
        let pool = self.create_thread_pool(thread_count);
        let operation = Arc::new(operation);
        
        // Submit work to thread pool
        for repo in repositories {
            let tx = tx.clone();
            let operation = operation.clone();
            let completed = completed.clone();
            let successful = successful.clone();
            let failed = failed.clone();
            
            pool.execute(move || {
                let repo_start_time = Instant::now();
                let repo_clone = repo.clone();
                
                // Execute the operation and capture the result
                let result = operation(&repo)
                    .context(format!("Failed to execute operation on repository at {}", repo.path.display()));
                
                // Update counters
                completed.fetch_add(1, Ordering::SeqCst);
                if result.is_ok() {
                    successful.fetch_add(1, Ordering::SeqCst);
                } else {
                    failed.fetch_add(1, Ordering::SeqCst);
                }
                
                // Send result back via channel
                let _ = tx.send(OperationResult {
                    repository: repo_clone,
                    result,
                    duration: repo_start_time.elapsed(),
                });
            });
        }

        // Drop original sender so rx.recv() will eventually return Err when all threads are done
        drop(tx);
        
        // Collect results and report progress
        let mut results = Vec::with_capacity(total_repos);
        let timeout = if self.config.timeout_secs > 0 {
            Some(Duration::from_secs(self.config.timeout_secs))
        } else {
            None
        };
        
        while results.len() < total_repos {
            // Check for timeout
            if let Some(timeout_duration) = timeout {
                if start_time.elapsed() > timeout_duration {
                    // Stop all threads and return timeout error
                    pool.shutdown();
                    return Err(anyhow::Error::new(OrchestratorError::OperationTimeout {
                        timeout_secs: self.config.timeout_secs,
                    }));
                }
            }
            
            // Wait for next result with timeout
            let recv_result = if let Some(timeout_duration) = timeout {
                // Use a shorter timeout for polling to allow periodic timeout checks
                let poll_timeout = std::cmp::min(Duration::from_secs(1), timeout_duration);
                rx.recv_timeout(poll_timeout)
            } else {
                // No timeout, wait indefinitely
                rx.recv().map_err(|_| mpsc::RecvTimeoutError::Disconnected)
            };
            
            match recv_result {
                Ok(result) => {
                    results.push(result);
                    
                    // Report progress if callback is set
                    if let Some(ref callback) = self.progress_callback {
                        let progress = ProgressInfo {
                            total: total_repos,
                            completed: completed.load(Ordering::SeqCst),
                            successful: successful.load(Ordering::SeqCst),
                            failed: failed.load(Ordering::SeqCst),
                            elapsed: start_time.elapsed(),
                        };
                        callback(&progress);
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Just a polling timeout, continue
                    continue;
                },
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // All senders dropped, but we didn't get enough results
                    // This shouldn't happen in normal operation
                    if results.len() < total_repos {
                        return Err(anyhow::Error::new(OrchestratorError::ThreadPanic {
                            message: format!(
                                "Thread communication failed. Expected {} results, got {}.",
                                total_repos, results.len()
                            ),
                        }));
                    }
                    break;
                }
            }
        }
        
        // Check if all operations failed and we're not allowed to continue on error
        if !self.config.continue_on_error && failed.load(Ordering::SeqCst) == total_repos {
            return Err(anyhow::Error::new(OrchestratorError::AllOperationsFailed {
                message: format!("All {} operations failed", total_repos),
            }));
        }
        
        Ok(results)
    }
    
    /// Create a thread pool with the given number of threads
    fn create_thread_pool(&self, thread_count: usize) -> ThreadPool {
        ThreadPool::new(thread_count)
    }
    
    /// Group operation results by status (success/failure)
    pub fn group_results<T>(results: Vec<OperationResult<T>>) -> (Vec<OperationResult<T>>, Vec<OperationResult<T>>) {
        let mut successes = Vec::new();
        let mut failures = Vec::new();
        
        for result in results {
            if result.result.is_ok() {
                successes.push(result);
            } else {
                failures.push(result);
            }
        }
        
        (successes, failures)
    }
    
    /// Execute an operation on multiple repositories and collect the successful results
    pub fn execute_and_collect<F, T>(&self, repositories: Vec<Repository>, operation: F) -> Result<Vec<T>>
    where
        F: Fn(&Repository) -> Result<T> + Send + Sync + 'static,
        T: Send + 'static,
    {
        let results = self.execute(repositories, operation)?;
        let (successes, failures) = Self::group_results(results);
        
        let successful_results: Vec<T> = successes
            .into_iter()
            .filter_map(|op_result| op_result.result.ok())
            .collect();
            
        if !failures.is_empty() && !self.config.continue_on_error {
            return Err(anyhow::Error::new(OrchestratorError::AllOperationsFailed {
                message: format!("{} operations failed", failures.len()),
            }));
        }
        
        Ok(successful_results)
    }
}

// Basic Thread Pool Implementation
// This is a simple worker thread pool implementation specifically for this orchestrator
struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Message>>,
}

impl ThreadPool {
    fn new(size: usize) -> ThreadPool {
        assert!(size > 0, "Thread pool size must be greater than 0");

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        if let Some(sender) = &self.sender {
            sender.send(Message::NewJob(job)).expect("Thread pool worker has disconnected");
        }
    }

    fn shutdown(&self) {
        // Send terminate message to all workers
        if let Some(sender) = &self.sender {
            for _ in &self.workers {
                // Ignore errors as workers might already be shutting down
                let _ = sender.send(Message::Terminate);
            }
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Send terminate message to all workers
        if let Some(sender) = &self.sender {
            for _ in &self.workers {
                let _ = sender.send(Message::Terminate);
            }
        }
        // Take ownership of sender and drop it to close the channel
        self.sender.take();

        // Wait for all workers to finish
        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                // Ignore errors from join as some threads might have panicked
                let _ = thread.join();
            }
        }
    }
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = {
                let lock_result = receiver.lock();
                
                // If we can't lock the mutex (e.g., another thread panicked while holding it),
                // we should terminate this worker
                let receiver_guard = match lock_result {
                    Ok(guard) => guard,
                    Err(_) => break,
                };
                
                // Receive the next message
                match receiver_guard.recv() {
                    Ok(message) => message,
                    Err(_) => break,
                }
            };

            match message {
                Message::NewJob(job) => {
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::thread;
    use std::sync::Mutex;
    
    // Mock repository for testing
    fn create_test_repo(id: usize) -> Repository {
        Repository {
            path: PathBuf::from(format!("/test/repo{}", id)),
            origin: format!("git@example.com:org/repo{}.git", id),
            branch: Some(format!("main")),
            is_git: true,
            enabled: true,
            tags: vec![],
            config_overrides: None,
        }
    }
    
    #[test]
    fn test_empty_repositories() {
        let orchestrator = RepositoryOrchestrator::new();
        let results = orchestrator.execute::<_, ()>(vec![], |_| Ok(()));
        
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 0);
    }
    
    #[test]
    fn test_successful_operations() {
        let orchestrator = RepositoryOrchestrator::new();
        let repos: Vec<_> = (0..5).map(create_test_repo).collect();
        
        let results = orchestrator.execute(repos.clone(), |repo| {
            Ok(repo.path.to_string_lossy().to_string())
        });
        
        assert!(results.is_ok());
        let results = results.unwrap();
        assert_eq!(results.len(), 5);
        
        for (i, result) in results.iter().enumerate() {
            assert!(result.result.is_ok());
            assert_eq!(
                result.result.as_ref().unwrap(),
                &repos[i].path.to_string_lossy().to_string()
            );
        }
    }
    
    #[test]
    fn test_failed_operations() {
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            continue_on_error: true,
            ..OrchestratorConfig::default()
        });
        
        let repos: Vec<_> = (0..5).map(create_test_repo).collect();
        
        let results = orchestrator.execute::<_, ()>(repos, |repo| {
            // Fail for even-numbered repos
            if repo.path.to_string_lossy().contains("repo0") || 
               repo.path.to_string_lossy().contains("repo2") || 
               repo.path.to_string_lossy().contains("repo4") {
                Err(anyhow!("Simulated failure"))
            } else {
                Ok(())
            }
        });
        
        assert!(results.is_ok());
        let results = results.unwrap();
        
        let (successes, failures) = RepositoryOrchestrator::group_results(results);
        assert_eq!(successes.len(), 2); // Odd-numbered repos succeed
        assert_eq!(failures.len(), 3);  // Even-numbered repos fail
    }
    
    #[test]
    fn test_all_operations_failed() {
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            continue_on_error: false, // Don't continue on error
            ..OrchestratorConfig::default()
        });
        
        let repos: Vec<_> = (0..3).map(create_test_repo).collect();
        
        let results = orchestrator.execute_and_collect::<_, ()>(repos, |_| {
            Err(anyhow!("Simulated failure"))
        });
        
        assert!(results.is_err());
        let error = results.err().unwrap();
        assert!(error.downcast_ref::<OrchestratorError>().is_some());
    }
    
    #[test]
    fn test_progress_callback() {
        let progress_data = Arc::new(Mutex::new(Vec::<ProgressInfo>::new()));
        let progress_data_clone = progress_data.clone();
        
        let callback = move |info: &ProgressInfo| {
            let mut data = progress_data_clone.lock().unwrap();
            data.push(info.clone());
        };
        
        let orchestrator = RepositoryOrchestrator::new()
            .with_progress_callback(callback);
            
        let repos: Vec<_> = (0..5).map(create_test_repo).collect();
        
        let _ = orchestrator.execute(repos, |repo| {
            // Simulate work
            thread::sleep(Duration::from_millis(50));
            Ok(repo.path.to_string_lossy().to_string())
        });
        
        let progress_history = progress_data.lock().unwrap();
        
        // We should have at least one progress update per repository
        assert!(progress_history.len() >= 1);
        
        // The last update should show all repos completed
        let last_progress = progress_history.last().unwrap();
        assert_eq!(last_progress.total, 5);
        assert_eq!(last_progress.completed, 5);
        assert_eq!(last_progress.successful, 5);
        assert_eq!(last_progress.failed, 0);
    }
    
    #[test]
    fn test_operation_timeout() {
        // Create an orchestrator with a very short timeout
        let orchestrator = RepositoryOrchestrator::with_config(OrchestratorConfig {
            timeout_secs: 1, // 1 second timeout
            ..OrchestratorConfig::default()
        });
        
        let repos: Vec<_> = (0..3).map(create_test_repo).collect();
        
        let results = orchestrator.execute::<_, ()>(repos, |_| {
            // Sleep longer than the timeout
            thread::sleep(Duration::from_secs(2));
            Ok(())
        });
        
        assert!(results.is_err());
        let error = results.err().unwrap();
        let timeout_error = error.downcast_ref::<OrchestratorError>();
        assert!(timeout_error.is_some());
        match timeout_error.unwrap() {
            OrchestratorError::OperationTimeout { timeout_secs } => {
                assert_eq!(*timeout_secs, 1);
            },
            _ => panic!("Expected OperationTimeout error"),
        }
    }
}