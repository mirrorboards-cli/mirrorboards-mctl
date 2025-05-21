# MCTL Coder Mode Rules

Core Philosophy

1. Rust Excellence
   - Leverage Rust's safety features, ownership model, and type system to create robust, efficient code.
   - Follow idiomatic Rust patterns and community best practices for maintainable, performant implementations.

2. Git Operation Safety
   - Implement comprehensive error handling for all git operations to ensure repository integrity.
   - Design operations to be atomic and recoverable whenever possible.

3. Configuration-Driven Design
   - Externalize all configurable aspects through structured configuration files.
   - Validate configurations thoroughly before execution to prevent synchronization failures.

4. Secure Credential Management
   - Never hardcode credentials or tokens in source code.
   - Implement secure credential retrieval and storage mechanisms.

5. Modular Architecture
   - Maintain clear separation of concerns with well-defined module boundaries.
   - Design components to be independently testable and maintainable.

Methodology & Workflow

- Structured Development
  - Follow a clear, systematic approach from specification through implementation and testing.
- Incremental Implementation
  - Build features progressively, with thorough testing at each stage.
- Comprehensive Error Handling
  - Design robust error handling that provides clear, actionable feedback.
- Security-First Mindset
  - Consider security implications at every stage of development.

Rust-Specific Best Practices

1. Type Safety
   - Leverage Rust's type system to prevent errors at compile time.
   - Use custom types and enums to represent domain concepts clearly.
   - Implement appropriate trait bounds to enforce constraints.

2. Error Handling
   - Use Result<T, E> for operations that can fail, with custom error types.
   - Implement the std::error::Error trait for all error types.
   - Provide context with error chains using anyhow, thiserror, or similar libraries.
   - Avoid unwrap() and expect() in production code; use proper error propagation.

3. Memory Management
   - Respect Rust's ownership model to prevent memory leaks and data races.
   - Use references and borrowing appropriately to minimize copying.
   - Implement Clone, Copy, and other traits judiciously based on performance needs.

4. Concurrency
   - Use Rust's concurrency primitives (threads, async/await) appropriately for the task.
   - Leverage the type system to ensure thread safety.
   - Consider performance implications of synchronization mechanisms.

5. Code Organization
   - Structure code with clear module hierarchies.
   - Use Rust's visibility rules (pub, pub(crate), etc.) to enforce encapsulation.
   - Keep files under 500 lines and functions under 50 lines.

6. Documentation
   - Write comprehensive documentation comments (///) for public APIs.
   - Include examples in documentation where appropriate.
   - Document error conditions and handling strategies.

Git Operation Implementation

1. Command Execution
   - Use structured, type-safe wrappers around Command execution.
   - Validate command parameters before execution.
   - Implement timeouts for long-running operations.

2. Output Parsing
   - Parse git command output carefully, handling encoding issues.
   - Use structured types to represent parsed output.
   - Handle unexpected output formats gracefully.

3. Error Recovery
   - Implement strategies to recover from common git errors.
   - Design operations to be idempotent when possible.
   - Provide clear error messages that suggest remediation steps.

4. Operation Atomicity
   - Ensure operations are atomic or can be safely retried.
   - Implement proper cleanup for interrupted operations.
   - Use transactions or similar patterns for multi-step operations.

5. Repository State Validation
   - Verify repository state before and after operations.
   - Implement consistency checks for critical operations.
   - Detect and handle repository corruption scenarios.

Configuration Management

1. Schema Validation
   - Implement thorough validation for all configuration files.
   - Provide clear, actionable error messages for invalid configurations.
   - Use strong typing for configuration structures.

2. Default Values
   - Provide sensible defaults where appropriate.
   - Document all default values clearly.
   - Make defaults environment-aware when necessary.

3. Configuration Loading
   - Support multiple configuration sources (files, environment, CLI).
   - Implement clear precedence rules for configuration sources.
   - Handle missing or inaccessible configuration gracefully.

4. Sensitive Information
   - Never store sensitive information in plain text.
   - Support secure external credential sources.
   - Implement proper masking for logs and error messages.

Secure Credential Handling

1. Credential Sources
   - Support multiple secure credential sources (environment variables, credential stores, etc.).
   - Never hardcode credentials in source code or configuration files.
   - Implement proper fallback mechanisms for credential retrieval.

2. Credential Lifecycle
   - Load credentials only when needed.
   - Clear sensitive information from memory when no longer required.
   - Implement proper error handling for missing or invalid credentials.

3. Secure Storage
   - Use platform-appropriate secure storage mechanisms.
   - Support integration with git credential helpers.
   - Implement proper encryption for any stored credentials.

4. Access Control
   - Limit credential access to only the components that require it.
   - Use principle of least privilege for all operations.
   - Implement proper authentication for administrative operations.

Testing Strategies

1. Unit Testing
   - Write comprehensive unit tests for all components.
   - Use Rust's testing framework effectively.
   - Implement proper mocking for external dependencies.

2. Integration Testing
   - Test git operations against real repositories.
   - Implement test fixtures for common scenarios.
   - Test error conditions and recovery mechanisms.

3. Property-Based Testing
   - Use property-based testing for complex algorithms.
   - Define clear invariants for critical operations.
   - Test edge cases systematically.

4. Security Testing
   - Implement tests for security-critical components.
   - Verify proper handling of sensitive information.
   - Test access control mechanisms thoroughly.

Error Handling & Logging

1. Error Types
   - Define clear, specific error types for different failure categories.
   - Implement proper error conversion between subsystems.
   - Provide context-rich error messages.

2. Error Propagation
   - Use the ? operator for concise error propagation.
   - Add context to errors as they propagate up the call stack.
   - Maintain appropriate level of detail in error chains.

3. Logging Strategy
   - Implement structured logging with appropriate levels.
   - Include relevant context in log messages.
   - Ensure sensitive information is never logged.

4. User Feedback
   - Translate technical errors into user-friendly messages.
   - Provide actionable remediation steps when possible.
   - Implement proper error codes for programmatic handling.

Performance Considerations

1. Resource Usage
   - Optimize memory usage for large repositories.
   - Implement proper buffering for I/O operations.
   - Consider CPU usage for compute-intensive operations.

2. Concurrency
   - Use parallelism appropriately for independent operations.
   - Implement proper synchronization for shared resources.
   - Consider async I/O for network-bound operations.

3. Caching
   - Implement strategic caching for expensive operations.
   - Use appropriate cache invalidation strategies.
   - Consider memory usage implications of caching.

4. Profiling
   - Use Rust profiling tools to identify bottlenecks.
   - Optimize critical paths based on profiling data.
   - Document performance characteristics of key operations.

Code Quality & Maintenance

1. Code Style
   - Follow Rust style guidelines consistently.
   - Use rustfmt for consistent formatting.
   - Implement clippy linting with appropriate configuration.

2. Dependency Management
   - Choose dependencies carefully, considering maintenance status and security.
   - Keep dependencies updated regularly.
   - Minimize dependency footprint where possible.

3. Versioning
   - Follow semantic versioning principles.
   - Document breaking changes clearly.
   - Maintain backward compatibility where possible.

4. Refactoring
   - Refactor code regularly to maintain quality.
   - Use Rust's strong type system to make refactoring safer.
   - Write comprehensive tests before major refactoring.

Implementation Examples

1. Error Handling Pattern
   ```rust
   // Define custom error type
   #[derive(Debug, thiserror::Error)]
   pub enum GitOperationError {
       #[error("Failed to execute git command: {0}")]
       CommandExecution(#[from] std::io::Error),
       
       #[error("Git command failed with exit code {code}: {message}")]
       CommandFailed {
           code: i32,
           message: String,
       },
       
       #[error("Repository not found at {0}")]
       RepositoryNotFound(String),
       
       #[error("Authentication failed: {0}")]
       AuthenticationFailed(String),
   }

   // Use in functions
   pub fn fetch_repository(repo_path: &str) -> Result<(), GitOperationError> {
       if !Path::new(repo_path).exists() {
           return Err(GitOperationError::RepositoryNotFound(repo_path.to_string()));
       }
       
       let output = Command::new("git")
           .current_dir(repo_path)
           .arg("fetch")
           .output()
           .map_err(GitOperationError::CommandExecution)?;
       
       if !output.status.success() {
           let message = String::from_utf8_lossy(&output.stderr).to_string();
           return Err(GitOperationError::CommandFailed {
               code: output.status.code().unwrap_or(-1),
               message,
           });
       }
       
       Ok(())
   }
   ```

2. Secure Credential Handling
   ```rust
   // Define credential provider trait
   pub trait CredentialProvider {
       fn get_credentials(&self, url: &str) -> Result<Credentials, CredentialError>;
   }

   // Environment variable implementation
   pub struct EnvCredentialProvider;
   
   impl CredentialProvider for EnvCredentialProvider {
       fn get_credentials(&self, _url: &str) -> Result<Credentials, CredentialError> {
           let username = std::env::var("GIT_USERNAME")
               .map_err(|_| CredentialError::MissingCredential("GIT_USERNAME"))?;
           
           let password = std::env::var("GIT_PASSWORD")
               .map_err(|_| CredentialError::MissingCredential("GIT_PASSWORD"))?;
           
           Ok(Credentials { username, password })
       }
   }

   // Usage with dependency injection
   pub struct GitClient<C: CredentialProvider> {
       credential_provider: C,
   }

   impl<C: CredentialProvider> GitClient<C> {
       pub fn new(credential_provider: C) -> Self {
           Self { credential_provider }
       }
       
       pub fn clone_repository(&self, url: &str, path: &str) -> Result<(), GitError> {
           let credentials = self.credential_provider.get_credentials(url)?;
           // Use credentials securely...
           // Clear credentials from memory when done
           Ok(())
       }
   }
   ```

3. Configuration Validation
   ```rust
   #[derive(Debug, Deserialize, Validate)]
   pub struct RepositoryConfig {
       #[validate(url)]
       pub origin: String,
       
       #[validate(path_exists)]
       pub path: String,
       
       pub branch: Option<String>,
   }

   pub fn load_config(path: &str) -> Result<Config, ConfigError> {
       let content = fs::read_to_string(path)
           .map_err(|e| ConfigError::IoError(path.to_string(), e))?;
       
       let config: Config = toml::from_str(&content)
           .map_err(|e| ConfigError::ParseError(e))?;
       
       config.validate()
           .map_err(ConfigError::ValidationError)?;
       
       Ok(config)
   }
   ```

4. Modular Command Execution
   ```rust
   pub struct GitCommand {
       repo_path: PathBuf,
       args: Vec<String>,
       timeout: Duration,
   }

   impl GitCommand {
       pub fn new<P: AsRef<Path>>(repo_path: P) -> Self {
           Self {
               repo_path: repo_path.as_ref().to_path_buf(),
               args: Vec::new(),
               timeout: Duration::from_secs(30),
           }
       }
       
       pub fn arg<S: AsRef<str>>(mut self, arg: S) -> Self {
           self.args.push(arg.as_ref().to_string());
           self
       }
       
       pub fn args<I, S>(mut self, args: I) -> Self 
       where 
           I: IntoIterator<Item = S>,
           S: AsRef<str>,
       {
           for arg in args {
               self.args.push(arg.as_ref().to_string());
           }
           self
       }
       
       pub fn timeout(mut self, timeout: Duration) -> Self {
           self.timeout = timeout;
           self
       }
       
       pub fn execute(self) -> Result<CommandOutput, GitCommandError> {
           // Implementation with proper timeout handling, output parsing, etc.
       }
   }

   // Usage
   let output = GitCommand::new(repo_path)
       .arg("fetch")
       .arg("--all")
       .timeout(Duration::from_secs(60))
       .execute()?;