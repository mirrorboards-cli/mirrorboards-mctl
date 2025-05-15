# MCTL Architecture Design

## 1. Overview

This document outlines the architecture design for the MCTL (Mirror Control) tool rewrite. The new architecture addresses the limitations identified in the current implementation while introducing improved features and capabilities.

### 1.1 Current Implementation Analysis

The current MCTL implementation has the following characteristics:

- Command-based architecture (sync, status, save)
- TOML configuration with basic repository definitions
- Git operations via system Git commands with SSH authentication
- Limited error handling and minimal separation of concerns
- No logging system, parallel processing, or test coverage

### 1.2 Requirements for New Architecture

1. Maintain military-grade quality and reliability
2. Ensure proper Git authentication via SSH using default Git SSH keys
3. Improved separation of concerns and modularity
4. Better error handling and logging
5. Support for parallel processing of repositories
6. Strong test coverage
7. Extensible command structure for future additions

## 2. Architecture Pattern

### 2.1 Layered Architecture

The MCTL rewrite will follow a layered architecture pattern with clear separation of concerns:

```
┌───────────────────────────────────────────────────────────┐
│                    Presentation Layer                      │
│  CLI Interface, Command Parsing, User Output Formatting    │
└─────────────────────────────┬─────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────┐
│                    Application Layer                       │
│  Command Implementations, Orchestration, Business Logic    │
└─────────────────────────────┬─────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────┐
│                      Domain Layer                          │
│  Core Entities, Repository Interfaces, Domain Services     │
└─────────────────────────────┬─────────────────────────────┘
                              │
                              ▼
┌───────────────────────────────────────────────────────────┐
│                   Infrastructure Layer                     │
│  Git Operations, Filesystem Access, Configuration          │
└───────────────────────────────────────────────────────────┘
```

### 2.2 Key Architecture Principles

1. **Clean Separation of Concerns**: Each layer has distinct responsibilities
2. **Dependency Inversion**: Higher layers depend on abstractions, not concrete implementations
3. **Single Responsibility**: Each module has one reason to change
4. **Open/Closed Principle**: Open for extension, closed for modification
5. **Interface Segregation**: Multiple specific interfaces are better than one general-purpose interface

## 3. Core Modules and Responsibilities

### 3.1 Presentation Layer

#### 3.1.1 CLI Module
- Command-line interface parsing using clap
- Command registration and routing
- Help text and documentation

#### 3.1.2 Output Module
- Formatted console output
- Color and styling
- Progress reporting
- Error presentation

### 3.2 Application Layer

#### 3.2.1 Command Module
- Command implementations
- Command registration system
- Command lifecycle management

#### 3.2.2 Orchestrator Module
- Parallel execution coordination
- Task scheduling and prioritization
- Cancellation and timeout handling

### 3.3 Domain Layer

#### 3.3.1 Repository Module
- Repository entities and aggregates
- Repository operations interfaces
- Domain events

#### 3.3.2 Configuration Module
- Configuration entities
- Configuration validation rules
- Default settings

### 3.4 Infrastructure Layer

#### 3.4.1 Git Module
- Git repository operations
- SSH authentication handling
- Git command execution

#### 3.4.2 FileSystem Module
- File and directory operations
- Path resolution
- File locking mechanisms

#### 3.4.3 Config Provider Module
- Configuration file loading
- Configuration parsing
- Environment variable integration

#### 3.4.4 Logging Module
- Structured logging
- Log levels and filtering
- Log rotation and storage

## 4. Interface Definitions and Dependencies

### 4.1 Key Interfaces

#### 4.1.1 Repository Operations Interface

```rust
pub trait RepositoryOperations {
    // Core operations
    fn clone(&self, url: &str, path: &Path) -> Result<()>;
    fn update_submodules(&self, path: &Path) -> Result<()>;
    fn has_changes(&self, path: &Path) -> Result<bool>;
    fn commit_changes(&self, path: &Path, message: &str) -> Result<()>;
    fn push_changes(&self, path: &Path) -> Result<()>;
    
    // Information retrieval
    fn get_status(&self, path: &Path) -> Result<RepositoryStatus>;
    fn get_remote_url(&self, path: &Path) -> Result<String>;
}
```

#### 4.1.2 Configuration Provider Interface

```rust
pub trait ConfigProvider {
    fn load_config(&self) -> Result<Config>;
    fn find_config_file(&self) -> Result<PathBuf>;
    fn get_default_config(&self) -> Config;
}
```

#### 4.1.3 Logger Interface

```rust
pub trait Logger {
    fn debug(&self, message: &str, context: Option<&LogContext>);
    fn info(&self, message: &str, context: Option<&LogContext>);
    fn warn(&self, message: &str, context: Option<&LogContext>);
    fn error(&self, message: &str, context: Option<&LogContext>);
    fn with_context(&self, context: LogContext) -> Box<dyn Logger>;
}
```

#### 4.1.4 Command Interface

```rust
pub trait Command {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, args: &[String]) -> Result<()>;
}
```

### 4.2 Dependency Graph

```
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│  CLI Module   │────▶│ Command Module │────▶│ Repository    │
└───────────────┘     └───────────────┘     │ Operations    │
                             │              └───────────────┘
                             │                      ▲
                             ▼                      │
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│ Output Module │◀────│ Orchestrator  │────▶│ Git Module    │
└───────────────┘     │ Module        │     └───────────────┘
                      └───────────────┘
                             │
                             │
                             ▼
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│ Config        │◀────│ Configuration │────▶│ FileSystem    │
│ Provider      │     │ Module        │     │ Module        │
└───────────────┘     └───────────────┘     └───────────────┘
                             │
                             │
                             ▼
                      ┌───────────────┐
                      │ Logging       │
                      │ Module        │
                      └───────────────┘
```

## 5. Data Flow Diagrams

### 5.1 Configuration Loading Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ CLI Module   │────▶│ Config       │────▶│ FileSystem   │
└──────────────┘     │ Provider     │     │ Module       │
                     └──────────────┘     └──────────────┘
                            │                     │
                            ▼                     ▼
                     ┌──────────────┐     ┌──────────────┐
                     │ Configuration│◀────│ TOML File    │
                     │ Module       │     │              │
                     └──────────────┘     └──────────────┘
                            │
                            ▼
                     ┌──────────────┐
                     │ Command      │
                     │ Module       │
                     └──────────────┘
```

### 5.2 Repository Sync Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ User         │────▶│ CLI Module   │────▶│ Sync Command │
└──────────────┘     └──────────────┘     └──────────────┘
                                                 │
                                                 ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Output       │◀────│ Orchestrator │◀────│ Configuration│
│ Module       │     │ Module       │     │ Module       │
└──────────────┘     └──────────────┘     └──────────────┘
                            │
                            ▼
                     ┌──────────────┐     ┌──────────────┐
                     │ Repository   │────▶│ Git Module   │
                     │ Operations   │     │              │
                     └──────────────┘     └──────────────┘
                                                 │
                                                 ▼
                                          ┌──────────────┐
                                          │ SSH Auth     │
                                          │ (system)     │
                                          └──────────────┘
```

## 6. Error Handling Strategy

### 6.1 Error Architecture

The error handling strategy will follow a tiered approach:

1. **Domain-Specific Errors**: Define error types for each domain concern using thiserror
2. **Error Context**: Add context to errors as they propagate up the layers using anyhow
3. **Error Reporting**: Present errors to users in a clear, actionable format
4. **Error Logging**: Log detailed error information for debugging

### 6.2 Error Type Hierarchy

```
Error
├── ConfigError
│   ├── ConfigFileNotFound
│   ├── ConfigParseError
│   └── ConfigValidationError
├── GitError
│   ├── CloneError
│   ├── PushError
│   ├── CommitError
│   └── AuthenticationError
├── RepositoryError
│   ├── RepositoryNotFound
│   ├── RepositoryAlreadyExists
│   └── SubmoduleError
└── CommandError
    ├── InvalidArgumentError
    ├── CommandExecutionError
    └── CommandTimeoutError
```

### 6.3 Error Handling Principles

1. **Fail Early, Fail Loudly**: Validate inputs at boundaries
2. **Don't Lose Context**: Preserve error chain and add context at each layer
3. **Actionable Errors**: Provide clear instructions on how to resolve errors
4. **Graceful Degradation**: Continue processing where possible, fail safely where not

## 7. Authentication Mechanism

### 7.1 SSH Authentication for Git Operations

The MCTL tool will use the system's Git command with SSH authentication, leveraging the user's existing SSH keys and configuration.

#### 7.1.1 SSH Authentication Flow

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Git Module   │────▶│ System Git   │────▶│ SSH Agent    │
└──────────────┘     │ Command      │     │              │
                     └──────────────┘     └──────────────┘
                                                 │
                                                 ▼
                                          ┌──────────────┐
                                          │ User's SSH   │
                                          │ Keys         │
                                          └──────────────┘
```

#### 7.1.2 SSH Authentication Implementation

```rust
pub struct GitSshHandler {
    // Configuration for SSH behavior
    ssh_key_path: Option<PathBuf>,
    known_hosts_path: Option<PathBuf>,
}

impl GitSshHandler {
    // Create a new handler with default SSH paths
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| String::from("."));
        Self {
            ssh_key_path: Some(PathBuf::from(format!("{}/.ssh/id_rsa", home))),
            known_hosts_path: Some(PathBuf::from(format!("{}/.ssh/known_hosts", home))),
        }
    }

    // Set up the environment for Git command execution with SSH
    pub fn prepare_environment(&self) -> Result<HashMap<String, String>> {
        let mut env_vars = HashMap::new();
        
        // Use GIT_SSH_COMMAND to configure SSH behavior
        let mut ssh_command = String::from("ssh");
        
        if let Some(key_path) = &self.ssh_key_path {
            if key_path.exists() {
                ssh_command.push_str(&format!(" -i {}", key_path.display()));
            }
        }
        
        if let Some(known_hosts) = &self.known_hosts_path {
            if known_hosts.exists() {
                ssh_command.push_str(&format!(" -o UserKnownHostsFile={}", known_hosts.display()));
            }
        }
        
        env_vars.insert("GIT_SSH_COMMAND".to_string(), ssh_command);
        
        Ok(env_vars)
    }
}
```

### 7.2 Authentication Error Handling

- Detect authentication failures in Git command output
- Provide clear error messages about SSH key issues
- Include troubleshooting guidance in error messages

## 8. Parallel Processing

### 8.1 Parallel Repository Processing

The MCTL tool will support parallel processing of repositories using a thread pool approach:

```rust
pub struct RepositoryOrchestrator {
    thread_pool: ThreadPool,
    max_concurrent_tasks: usize,
}

impl RepositoryOrchestrator {
    pub fn new(max_concurrent_tasks: usize) -> Self {
        Self {
            thread_pool: ThreadPool::new(max_concurrent_tasks),
            max_concurrent_tasks,
        }
    }

    pub fn process_repositories<F>(&self, repositories: Vec<Repository>, operation: F) -> Vec<Result<()>>
    where
        F: Fn(&Repository) -> Result<()> + Send + Sync + Clone + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let operation = Arc::new(operation);

        for repo in repositories {
            let tx = tx.clone();
            let operation = operation.clone();
            let repo_clone = repo.clone();

            self.thread_pool.execute(move || {
                let result = operation(&repo_clone);
                tx.send((repo_clone, result)).expect("Channel send failed");
            });
        }

        // Collect results
        drop(tx); // Drop the original sender
        let mut results = Vec::new();
        while let Ok((repo, result)) = rx.recv() {
            // Process result and report progress
            results.push(result);
        }

        results
    }
}
```

### 8.2 Progress Reporting

A cross-thread progress reporting mechanism will track and display progress:

```rust
pub struct ProgressReporter {
    total: AtomicUsize,
    completed: AtomicUsize,
    errors: AtomicUsize,
}

impl ProgressReporter {
    pub fn new(total: usize) -> Self {
        Self {
            total: AtomicUsize::new(total),
            completed: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
        }
    }

    pub fn increment_completed(&self) {
        self.completed.fetch_add(1, Ordering::SeqCst);
        self.report_progress();
    }

    pub fn increment_errors(&self) {
        self.errors.fetch_add(1, Ordering::SeqCst);
        self.report_progress();
    }

    pub fn report_progress(&self) {
        let completed = self.completed.load(Ordering::SeqCst);
        let total = self.total.load(Ordering::SeqCst);
        let errors = self.errors.load(Ordering::SeqCst);

        // Update progress bar or print progress
        println!("Progress: {}/{} complete, {} errors", completed, total, errors);
    }
}
```

## 9. Testing Strategy

### 9.1 Unit Testing

Each module will have comprehensive unit tests with the following characteristics:

- High test coverage (target: >90%)
- Mock dependencies using trait-based interfaces
- Test all error paths and edge cases
- Table-driven tests for complex logic

Example unit test:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::*;

    mock! {
        GitOperations {}
        impl RepositoryOperations for GitOperations {
            fn clone(&self, url: &str, path: &Path) -> Result<()>;
            fn update_submodules(&self, path: &Path) -> Result<()>;
            fn has_changes(&self, path: &Path) -> Result<bool>;
            fn commit_changes(&self, path: &Path, message: &str) -> Result<()>;
            fn push_changes(&self, path: &Path) -> Result<()>;
            fn get_status(&self, path: &Path) -> Result<RepositoryStatus>;
            fn get_remote_url(&self, path: &Path) -> Result<String>;
        }
    }

    #[test]
    fn test_sync_command_execution() {
        let mut mock_git = MockGitOperations::new();
        
        // Set up expectations
        mock_git.expect_clone()
            .with(eq("git@github.com:org/repo.git"), any())
            .times(1)
            .returning(|_, _| Ok(()));
            
        mock_git.expect_update_submodules()
            .times(1)
            .returning(|_| Ok(()));
        
        // Create command with mock
        let command = SyncCommand::new(mock_git);
        
        // Execute command
        let result = command.execute(&["sync"]);
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### 9.2 Integration Testing

Integration tests will verify the interaction between real components:

- Test real components working together
- Mock external systems (Git) when appropriate
- Test end-to-end command workflows
- Verify error handling across component boundaries

### 9.3 Test Doubles Strategy

The testing approach will use several types of test doubles:

- **Mocks**: For verifying interactions and method calls
- **Stubs**: For providing canned answers to calls
- **Fakes**: Simplified implementations of interfaces (e.g., in-memory repository)
- **Spies**: For recording method calls for later verification

## 10. Logging System

### 10.1 Structured Logging

The logging system will use structured logging to enable better filtering and analysis:

```rust
pub struct StructuredLogger {
    level: LogLevel,
    target: Box<dyn LogTarget>,
}

impl Logger for StructuredLogger {
    fn debug(&self, message: &str, context: Option<&LogContext>) {
        if self.level <= LogLevel::Debug {
            self.log(LogLevel::Debug, message, context);
        }
    }
    
    // Other log level methods...
    
    fn with_context(&self, context: LogContext) -> Box<dyn Logger> {
        // Create a new logger with this context
        Box::new(ContextualLogger {
            inner: self,
            context,
        })
    }
    
    fn log(&self, level: LogLevel, message: &str, context: Option<&LogContext>) {
        let log_entry = LogEntry {
            timestamp: chrono::Utc::now(),
            level,
            message: message.to_string(),
            context: context.cloned(),
        };
        
        self.target.write(&log_entry);
    }
}
```

### 10.2 Log Targets

Multiple log targets will be supported:

- Console output (with color)
- File output (with rotation)
- System log integration (syslog/journald)

## 11. Configuration Management

### 11.1 Enhanced Configuration Format

The configuration system will be enhanced to support:

- Multiple repository groups
- Per-repository configuration options
- Environment variable overrides
- Global default settings

Example enhanced TOML configuration:

```toml
# Global settings
[settings]
parallel_jobs = 4
ssh_key = "~/.ssh/id_rsa"
default_branch = "main"

# Repository groups
[groups.core]
description = "Core system repositories"

[groups.plugins]
description = "Plugin repositories"

# Repositories
[[repositories]]
path = "src/core"
origin = "git@github.com:org/core.git"
group = "core"
branch = "develop"  # Override default branch

[[repositories]]
path = "src/plugin-auth"
origin = "git@github.com:org/plugin-auth.git"
group = "plugins"
```

## 12. Command Structure

### 12.1 Extensible Command Design

The command structure will be extensible to allow for easy addition of new commands:

```rust
// Command registry for dynamic command registration
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, command: Box<dyn Command>) {
        self.commands.insert(command.name().to_string(), command);
    }
    
    pub fn get(&self, name: &str) -> Option<&Box<dyn Command>> {
        self.commands.get(name)
    }
    
    pub fn all_commands(&self) -> Vec<&Box<dyn Command>> {
        self.commands.values().collect()
    }
}
```

### 12.2 Core Commands

The core commands will include:

- `sync`: Clone and update repositories
- `status`: Check status of repositories
- `save`: Commit and push changes
- `list`: List configured repositories
- `update`: Update existing repositories
- `init`: Initialize a new configuration

### 12.3 Command Implementation Pattern

Each command will follow a consistent implementation pattern:

```rust
pub struct SyncCommand {
    git_ops: Box<dyn RepositoryOperations>,
    config_provider: Box<dyn ConfigProvider>,
    orchestrator: RepositoryOrchestrator,
    logger: Box<dyn Logger>,
}

impl Command for SyncCommand {
    fn name(&self) -> &str {
        "sync"
    }
    
    fn description(&self) -> &str {
        "Synchronize repositories defined in the configuration"
    }
    
    fn execute(&self, args: &[String]) -> Result<()> {
        // Parse command-specific arguments
        let args = self.parse_args(args)?;
        
        // Load configuration
        let config = self.config_provider.load_config()?;
        
        // Log operation start
        self.logger.info(&format!("Starting sync operation for {} repositories", config.repositories.len()), None);
        
        // Process repositories in parallel
        let results = self.orchestrator.process_repositories(
            config.repositories,
            |repo| self.sync_repository(repo),
        );
        
        // Handle results and generate summary
        self.handle_results(results)
    }
}

impl SyncCommand {
    fn sync_repository(&self, repo: &Repository) -> Result<()> {
        // Implementation details...
    }
    
    fn parse_args(&self, args: &[String]) -> Result<SyncArgs> {
        // Parse arguments using clap
    }
    
    fn handle_results(&self, results: Vec<Result<()>>) -> Result<()> {
        // Process results and generate summary
    }
}
```

## 13. Architecture Validation

### 13.1 Quality Attributes Assessment

| Quality Attribute | How Addressed in Architecture |
|-------------------|-------------------------------|
| Reliability | Comprehensive error handling, testing, logging |
| Maintainability | Clean separation of concerns, interfaces, modularity |
| Performance | Parallel processing, efficient Git operations |
| Security | System SSH integration, careful credential handling |
| Testability | Interface-based design, dependency injection |
| Usability | Clear command structure, helpful error messages |
| Extensibility | Command registry, plugin system |

### 13.2 Requirements Traceability

| Requirement | Architecture Component |
|-------------|------------------------|
| Military-grade quality | Error handling, logging, testing |
| SSH authentication | GitSshHandler, system Git integration |
| Separation of concerns | Layered architecture |
| Error handling & logging | Error system, structured logging |
| Parallel processing | RepositoryOrchestrator, thread pool |
| Test coverage | Unit & integration testing strategy |
| Extensible commands | CommandRegistry, Command trait |

## 14. Migration Strategy

A phased approach will be used for migrating from the current implementation:

1. Create new core domain models and interfaces
2. Implement infrastructure layer components
3. Build application layer with command implementations
4. Develop presentation layer with CLI interface
5. Test components individually and together
6. Release with backwards-compatible configuration

## 15. Conclusion

This architecture design addresses the current limitations of the MCTL tool while providing a solid foundation for future enhancements. The modular, layered approach with clear interfaces enables better maintainability, testability, and extensibility.