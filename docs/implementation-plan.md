# MCTL Technical Implementation Plan

## 1. Overview

This document outlines the detailed technical implementation plan for the MCTL tool rewrite based on the completed architecture design. The plan follows a bottom-up approach, starting with the foundational infrastructure components and progressively building up through the domain, application, and presentation layers.

## 2. Implementation Phases

### 2.1 Phase 1: Foundation and Infrastructure Layer (4 weeks)

**Description:** Establish the core infrastructure components that will serve as the foundation for the entire application.

#### Milestone 1.1: Development Environment Setup (3 days)
- Task 1.1.1: Set up Rust project structure with Cargo (0.5 days)
- Task 1.1.2: Configure development tools (rustfmt, clippy) (0.5 days)
- Task 1.1.3: Set up test infrastructure with mock support (1 day)
- Task 1.1.4: Configure CI/CD pipeline for automated testing (1 day)

#### Milestone 1.2: Core Infrastructure Components (2 weeks)
- Task 1.2.1: Implement Git Module with SSH authentication (4 days)
  - GitSshHandler implementation
  - Git command execution service
  - Unit tests for Git operations
- Task 1.2.2: Implement FileSystem Module (3 days)
  - File and directory operations
  - Path resolution utilities
  - Unit tests for filesystem operations
- Task 1.2.3: Implement ConfigProvider Module (3 days)
  - TOML configuration parsing
  - Configuration file discovery
  - Unit tests for configuration loading
- Task 1.2.4: Implement Logging Module (4 days)
  - Structured logging system
  - Log levels and filtering
  - Multiple log targets (console, file)
  - Unit tests for logging functionality

#### Milestone 1.3: Infrastructure Integration Testing (1 week)
- Task 1.3.1: Integration tests for Git and FileSystem modules (2 days)
- Task 1.3.2: Integration tests for ConfigProvider and Logging (2 days)
- Task 1.3.3: End-to-end tests for infrastructure layer (3 days)

### 2.2 Phase 2: Domain Layer Implementation (3 weeks)

**Description:** Implement the core domain models, interfaces, and error handling framework.

#### Milestone 2.1: Domain Models and Interfaces (1.5 weeks)
- Task 2.1.1: Implement Repository entity and value objects (3 days)
- Task 2.1.2: Implement Configuration domain models (3 days)
- Task 2.1.3: Define and implement domain interfaces (RepositoryOperations, ConfigProvider, Logger) (3 days)

#### Milestone 2.2: Error Handling Framework (1 week)
- Task 2.2.1: Define domain-specific error types using thiserror (2 days)
- Task 2.2.2: Implement error context propagation with anyhow (2 days)
- Task 2.2.3: Create error conversion between layers (1 day)
- Task 2.2.4: Unit tests for error handling (1 day)

#### Milestone 2.3: Domain Layer Testing (0.5 weeks)
- Task 2.3.1: Unit tests for domain models (1 day)
- Task 2.3.2: Unit tests for domain interfaces (1 day)
- Task 2.3.3: Integration tests for domain layer (0.5 days)

### 2.3 Phase 3: Application Layer Implementation (4 weeks)

**Description:** Implement the business logic, command implementations, and orchestration components.

#### Milestone 3.1: Orchestrator Module (1.5 weeks)
- Task 3.1.1: Implement RepositoryOrchestrator with thread pool (4 days)
- Task 3.1.2: Create ProgressReporter for parallel operations (2 days)
- Task 3.1.3: Implement task scheduling and cancellation (1 day)
- Task 3.1.4: Unit tests for orchestration components (1 day)

#### Milestone 3.2: Command Module (1 week)
- Task 3.2.1: Implement CommandRegistry (1 day)
- Task 3.2.2: Create Command interface and base implementation (1 day)
- Task 3.2.3: Implement command argument parsing (1 day)
- Task 3.2.4: Unit tests for Command module (2 days)

#### Milestone 3.3: Core Command Implementations (1.5 weeks)
- Task 3.3.1: Implement SyncCommand (2 days)
  - Repository cloning and updating
  - Submodule handling
  - Error handling and reporting
- Task 3.3.2: Implement StatusCommand (2 days)
  - Repository status checking
  - Status reporting
- Task 3.3.3: Implement SaveCommand (2 days)
  - Commit and push operations
  - Error handling for conflicts
- Task 3.3.4: Implement additional commands (ListCommand, UpdateCommand, InitCommand) (2 days)

### 2.4 Phase 4: Presentation Layer Implementation (2 weeks)

**Description:** Implement the user interface, command parsing, and output formatting.

#### Milestone 4.1: CLI Module (1 week)
- Task 4.1.1: Implement command-line parsing with clap (2 days)
- Task 4.1.2: Create command routing system (2 days)
- Task 4.1.3: Implement help text and documentation (1 day)
- Task 4.1.4: Unit tests for CLI module (2 days)

#### Milestone 4.2: Output Module (1 week)
- Task 4.2.1: Implement formatted console output (2 days)
- Task 4.2.2: Create progress display with spinners/bars (2 days)
- Task 4.2.3: Implement error presentation formatting (2 days)
- Task 4.2.4: Unit tests for output components (1 day)

### 2.5 Phase 5: Integration, Testing, and Refinement (3 weeks)

**Description:** Comprehensive testing, optimization, and finalization.

#### Milestone 5.1: Integration Testing (1.5 weeks)
- Task 5.1.1: End-to-end tests for sync workflow (3 days)
- Task 5.1.2: End-to-end tests for status workflow (2 days)
- Task 5.1.3: End-to-end tests for save workflow (2 days)
- Task 5.1.4: Testing of command-line interface (1 day)

#### Milestone 5.2: Performance Optimization (1 week)
- Task 5.2.1: Profile and optimize Git operations (2 days)
- Task 5.2.2: Optimize parallel processing (2 days)
- Task 5.2.3: Benchmark and tune critical workflows (1 day)

#### Milestone 5.3: Documentation and Finalization (0.5 weeks)
- Task 5.3.1: Create user documentation (1 day)
- Task 5.3.2: Finalize API documentation (1 day)
- Task 5.3.3: Prepare release artifacts (0.5 days)

## 3. Dependencies Between Components

### 3.1 Component Dependency Graph

```
Infrastructure Layer
├── Git Module
├── FileSystem Module
├── ConfigProvider Module
└── Logging Module
    │
    ▼
Domain Layer
├── Repository Module  (depends on Git Module)
├── Configuration Module (depends on ConfigProvider Module)
└── Error Module (used by all components)
    │
    ▼
Application Layer
├── Command Module (depends on Repository and Configuration Modules)
├── Orchestrator Module (depends on Repository Module)
└── Progress Reporter (depends on Logging Module)
    │
    ▼
Presentation Layer
├── CLI Module (depends on Command Module)
└── Output Module (depends on Progress Reporter)
```

### 3.2 Suggested Implementation Order

1. **Infrastructure Layer**
   - Start with Git Module (highest technical risk)
   - Implement FileSystem Module in parallel
   - Implement ConfigProvider Module
   - Implement Logging Module

2. **Domain Layer**
   - Implement Repository and Configuration domain models
   - Define domain interfaces
   - Implement error handling framework

3. **Application Layer**
   - Implement Orchestrator Module
   - Implement Command Module
   - Implement core commands

4. **Presentation Layer**
   - Implement CLI Module
   - Implement Output Module

This order allows for early implementation and testing of the core functionality without waiting for the entire system to be built.

## 4. Detailed Task List with Effort Estimates

| Phase | Milestone | Task | Description | Effort (days) |
|-------|-----------|------|-------------|---------------|
| 1 | 1.1 | 1.1.1 | Set up Rust project structure | 0.5 |
| 1 | 1.1 | 1.1.2 | Configure development tools | 0.5 |
| 1 | 1.1 | 1.1.3 | Set up test infrastructure | 1 |
| 1 | 1.1 | 1.1.4 | Configure CI/CD pipeline | 1 |
| 1 | 1.2 | 1.2.1 | Implement Git Module with SSH authentication | 4 |
| 1 | 1.2 | 1.2.2 | Implement FileSystem Module | 3 |
| 1 | 1.2 | 1.2.3 | Implement ConfigProvider Module | 3 |
| 1 | 1.2 | 1.2.4 | Implement Logging Module | 4 |
| 1 | 1.3 | 1.3.1 | Integration tests for Git and FileSystem | 2 |
| 1 | 1.3 | 1.3.2 | Integration tests for ConfigProvider and Logging | 2 |
| 1 | 1.3 | 1.3.3 | End-to-end tests for infrastructure layer | 3 |
| 2 | 2.1 | 2.1.1 | Implement Repository entity | 3 |
| 2 | 2.1 | 2.1.2 | Implement Configuration models | 3 |
| 2 | 2.1 | 2.1.3 | Define domain interfaces | 3 |
| 2 | 2.2 | 2.2.1 | Define domain-specific error types | 2 |
| 2 | 2.2 | 2.2.2 | Implement error context propagation | 2 |
| 2 | 2.2 | 2.2.3 | Create error conversion between layers | 1 |
| 2 | 2.2 | 2.2.4 | Unit tests for error handling | 1 |
| 2 | 2.3 | 2.3.1 | Unit tests for domain models | 1 |
| 2 | 2.3 | 2.3.2 | Unit tests for domain interfaces | 1 |
| 2 | 2.3 | 2.3.3 | Integration tests for domain layer | 0.5 |
| 3 | 3.1 | 3.1.1 | Implement RepositoryOrchestrator | 4 |
| 3 | 3.1 | 3.1.2 | Create ProgressReporter | 2 |
| 3 | 3.1 | 3.1.3 | Implement task scheduling and cancellation | 1 |
| 3 | 3.1 | 3.1.4 | Unit tests for orchestration components | 1 |
| 3 | 3.2 | 3.2.1 | Implement CommandRegistry | 1 |
| 3 | 3.2 | 3.2.2 | Create Command interface | 1 |
| 3 | 3.2 | 3.2.3 | Implement command argument parsing | 1 |
| 3 | 3.2 | 3.2.4 | Unit tests for Command module | 2 |
| 3 | 3.3 | 3.3.1 | Implement SyncCommand | 2 |
| 3 | 3.3 | 3.3.2 | Implement StatusCommand | 2 |
| 3 | 3.3 | 3.3.3 | Implement SaveCommand | 2 |
| 3 | 3.3 | 3.3.4 | Implement additional commands | 2 |
| 4 | 4.1 | 4.1.1 | Implement command-line parsing | 2 |
| 4 | 4.1 | 4.1.2 | Create command routing system | 2 |
| 4 | 4.1 | 4.1.3 | Implement help text and documentation | 1 |
| 4 | 4.1 | 4.1.4 | Unit tests for CLI module | 2 |
| 4 | 4.2 | 4.2.1 | Implement formatted console output | 2 |
| 4 | 4.2 | 4.2.2 | Create progress display | 2 |
| 4 | 4.2 | 4.2.3 | Implement error presentation formatting | 2 |
| 4 | 4.2 | 4.2.4 | Unit tests for output components | 1 |
| 5 | 5.1 | 5.1.1 | End-to-end tests for sync workflow | 3 |
| 5 | 5.1 | 5.1.2 | End-to-end tests for status workflow | 2 |
| 5 | 5.1 | 5.1.3 | End-to-end tests for save workflow | 2 |
| 5 | 5.1 | 5.1.4 | Testing of command-line interface | 1 |
| 5 | 5.2 | 5.2.1 | Profile and optimize Git operations | 2 |
| 5 | 5.2 | 5.2.2 | Optimize parallel processing | 2 |
| 5 | 5.2 | 5.2.3 | Benchmark and tune critical workflows | 1 |
| 5 | 5.3 | 5.3.1 | Create user documentation | 1 |
| 5 | 5.3 | 5.3.2 | Finalize API documentation | 1 |
| 5 | 5.3 | 5.3.3 | Prepare release artifacts | 0.5 |

Total effort: ~80 developer days (approximately 16 weeks for one developer)

## 5. Critical Path Identification

The critical path represents the sequence of dependent tasks that determine the minimum project timeline:

1. **Git Module Implementation** (Task 1.2.1) - This is the foundation of all repository operations and has the highest technical risk
2. **Repository Domain Model** (Task 2.1.1) - Depends on Git Module and is required for all command implementations
3. **Repository Orchestrator** (Task 3.1.1) - Core to parallel processing functionality
4. **Command Implementation** (Tasks 3.3.1, 3.3.2, 3.3.3) - Required for application functionality
5. **CLI Interface** (Task 4.1.1, 4.1.2) - Required for user interaction
6. **Integration Testing and Refinement** (Phase 5) - Final validation and optimization

Delays in any of these components will directly impact the overall project timeline. Additional resources should be allocated to these tasks if possible.

## 6. Testing Milestones

Testing is integrated throughout the development process:

### 6.1 Unit Testing (Continuous)
- Each component will have comprehensive unit tests with >90% code coverage
- Mock implementations will be used for dependencies
- Tests will cover both success and error paths

### 6.2 Integration Testing Milestones
- **Infrastructure Integration** (Milestone 1.3) - Week 4
- **Domain Layer Testing** (Milestone 2.3) - Week 7
- **Application Layer Testing** - After each command implementation
- **End-to-End Testing** (Milestone 5.1) - Weeks 14-15

### 6.3 Performance Testing
- **Phase 5** (Milestone 5.2) - Weeks 15-16
- Benchmark parallel processing with multiple repositories
- Optimize Git operations for speed and resource usage

## 7. Development Environment Setup Requirements

### 7.1 Tools and Dependencies
- Rust toolchain (latest stable)
- Cargo package manager
- Development tools:
  - rustfmt for code formatting
  - clippy for linting
  - cargo-tarpaulin for code coverage
  - mockall for mocking
  - criterion for benchmarking

### 7.2 External Dependencies
- Git command-line tool
- SSH key generation utilities
- Test repositories for integration testing

### 7.3 CI/CD Requirements
- Automated testing on each commit
- Coverage reporting
- Linting checks
- Build artifacts generation for multiple platforms

## 8. Git SSH Authentication Implementation

### 8.1 Authentication Strategy

The Git SSH authentication will leverage the system's Git command with proper SSH configuration:

1. **Use System Git Command**
   - Execute Git operations via system command
   - Leverage existing SSH agent and credentials

2. **SSH Key Configuration**
   - Use default SSH key path (~/.ssh/id_rsa) by default
   - Support custom key path via configuration
   - Validate key existence before operations

3. **Environment Configuration**
   - Set `GIT_SSH_COMMAND` environment variable
   - Configure SSH options (identity file, known hosts)
   - Support batch mode for non-interactive operation

### 8.2 Sample Implementation

```rust
pub struct GitSshHandler {
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
        let mut ssh_command = String::from("ssh -o BatchMode=yes");
        
        if let Some(key_path) = &self.ssh_key_path {
            if key_path.exists() {
                ssh_command.push_str(&format!(" -i {}", key_path.display()));
            } else {
                return Err(anyhow::anyhow!("SSH key not found at {}", key_path.display())
                    .context("Failed to prepare SSH environment"));
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

    // Detect authentication issues from Git output
    pub fn detect_auth_issues(&self, stderr: &str) -> Option<AuthError> {
        if stderr.contains("Permission denied (publickey)") {
            return Some(AuthError::PermissionDenied {
                message: "SSH authentication failed. Ensure your SSH key is correctly configured and added to the remote repository.".to_string(),
                key_path: self.ssh_key_path.clone(),
            });
        }
        
        if stderr.contains("Host key verification failed") {
            return Some(AuthError::HostVerificationFailed {
                message: "SSH host verification failed. The remote host key is not in your known_hosts file.".to_string(),
            });
        }
        
        None
    }
}
```

### 8.3 Implementation Recommendations

1. **Error Handling**
   - Detect SSH authentication failures from Git output
   - Provide clear error messages with troubleshooting guidance
   - Include key path information in error messages

2. **Testing Strategy**
   - Create mock Git responses for authentication scenarios
   - Test with various SSH key configurations
   - Verify behavior with missing or invalid keys

3. **Security Considerations**
   - Never store SSH private keys or passwords
   - Use the system's SSH agent for credential management
   - Support for SSH key passphrase via SSH agent

## 9. Summary and Conclusion

This implementation plan provides a comprehensive roadmap for developing the MCTL tool rewrite. By following a bottom-up approach with clear dependencies and milestones, the development team can:

1. Focus on the highest-risk components first (Git operations and SSH authentication)
2. Implement and test core functionality iteratively
3. Maintain a clear understanding of progress and dependencies
4. Ensure proper implementation of all architectural requirements

The total estimated effort is approximately 16 weeks for one developer, with potential for parallelization in some areas. The critical path focuses on the Git module, repository operations, and command implementations, which should be prioritized to ensure timely delivery.