# MCTL Architectural Decision Records

This document captures the key architectural decisions made for the MCTL (Mirror Control) system, including the context, considerations, and rationale behind each decision.

## Table of Contents

1. [ADR-001: Modular Architecture](#adr-001-modular-architecture)
2. [ADR-002: Configuration Format](#adr-002-configuration-format)
3. [ADR-003: Error Handling Strategy](#adr-003-error-handling-strategy)
4. [ADR-004: Credential Management](#adr-004-credential-management)
5. [ADR-005: Git Interface Abstraction](#adr-005-git-interface-abstraction)
6. [ADR-006: CLI Framework](#adr-006-cli-framework)
7. [ADR-007: Testing Strategy](#adr-007-testing-strategy)

## ADR-001: Modular Architecture

### Context

MCTL needs to manage multiple git repositories with various operations (add, sync, status, save, update). The system must be maintainable, extensible, and secure.

### Decision

Implement a modular architecture with clear separation of concerns:

1. CLI Interface: Handles command-line parsing and user interaction
2. Configuration Manager: Manages reading, validating, and writing configuration files
3. Repository Engine: Core component for repository operations
4. Status Monitor: Monitors repository status and provides reporting
5. Git Interface: Abstracts git operations
6. Security Layer: Manages credentials and secure operations
7. Error Handler: Centralized error handling and reporting

### Rationale

- **Maintainability**: Each module has a single responsibility, making the code easier to maintain
- **Extensibility**: New commands or features can be added by extending existing modules or adding new ones
- **Testability**: Modules can be tested in isolation with mock dependencies
- **Security**: Sensitive operations are isolated in the Security Layer
- **Reusability**: Common functionality is shared across commands

### Consequences

- Increased initial development effort to establish module boundaries
- More boilerplate code for module interfaces
- Better long-term maintainability and extensibility
- Easier onboarding for new developers

## ADR-002: Configuration Format

### Context

MCTL needs a configuration format to store repository information, including git URLs, local paths, and branch information.

### Decision

Use TOML (Tom's Obvious, Minimal Language) as the configuration format with the following structure:

```toml
# Optional global settings
base_path = "./repos"
default_branch = "main"

# Repository definitions
[[repositories]]
git-url = "git@github.com:example/repo.git"
path = "example-repo"
branch = "main"
```

### Rationale

- **Human-readable**: TOML is easy for humans to read and edit
- **Structured**: TOML supports nested structures and arrays
- **Widely supported**: Rust has excellent TOML parsing libraries
- **Minimal syntax**: TOML has less syntax overhead than JSON or YAML
- **Comments**: TOML supports comments, which are useful for documentation

### Consequences

- Users need to learn TOML syntax (though it's simple)
- Limited support for complex data structures compared to JSON
- Better readability and maintainability than JSON or INI formats
- Consistent with other Rust projects that commonly use TOML

## ADR-003: Error Handling Strategy

### Context

MCTL performs operations that can fail for various reasons (network issues, git errors, file system errors). The system needs a consistent way to handle and report errors.

### Decision

Implement a comprehensive error handling strategy with:

1. Domain-specific error types that implement a common error trait
2. Contextual errors that include information about where they occurred
3. User-friendly error messages
4. Recovery suggestions where possible
5. Detailed logging for debugging

### Rationale

- **Consistency**: Uniform error handling across the application
- **User experience**: Clear error messages help users understand and resolve issues
- **Debugging**: Detailed logs help developers diagnose problems
- **Recovery**: Suggestions help users recover from errors
- **Type safety**: Domain-specific error types provide compile-time checks

### Consequences

- More code to implement error types and handling
- Better user experience when errors occur
- Easier debugging and troubleshooting
- More robust error recovery

## ADR-004: Credential Management

### Context

MCTL needs to authenticate with git servers using various methods (SSH keys, HTTPS with username/password, tokens).

### Decision

Implement a secure credential management system with:

1. Support for environment variables (GIT_USERNAME, GIT_PASSWORD)
2. Integration with git credential helpers
3. SSH key support with passphrases
4. No storage of credentials in configuration files
5. Secure credential handling in memory

### Rationale

- **Security**: Credentials are not stored in plain text
- **Flexibility**: Multiple authentication methods are supported
- **Integration**: Works with existing git authentication mechanisms
- **User experience**: Minimal credential prompting
- **Best practices**: Follows security best practices for credential management

### Consequences

- More complex implementation
- Dependency on system credential stores
- Better security posture
- Improved user experience with fewer credential prompts

## ADR-005: Git Interface Abstraction

### Context

MCTL needs to interact with git repositories through various operations (clone, pull, status, commit, push).

### Decision

Create an abstraction layer for git operations using the `git2` crate, with:

1. High-level interface for common git operations
2. Error handling and recovery
3. Progress reporting
4. Credential management integration
5. Fallback to command-line git for complex operations

### Rationale

- **Abstraction**: Isolates git implementation details
- **Performance**: Direct library calls are faster than spawning git processes
- **Control**: Fine-grained control over git operations
- **Flexibility**: Can fall back to command-line git when needed
- **Progress**: Can report progress for long-running operations

### Consequences

- Learning curve for the `git2` API
- Potential limitations compared to command-line git
- Better performance for most operations
- More control over git operations
- Improved error handling and recovery

## ADR-006: CLI Framework

### Context

MCTL is a command-line tool with multiple commands and options.

### Decision

Use the `clap` crate for command-line argument parsing, with:

1. Subcommand structure for different operations
2. Consistent option naming
3. Help text and documentation
4. Validation of command-line arguments
5. Support for both long and short options

### Rationale

- **Maturity**: `clap` is a mature and widely used CLI framework
- **Features**: Comprehensive support for command-line parsing
- **Documentation**: Excellent documentation and examples
- **Validation**: Built-in validation of command-line arguments
- **Help**: Automatic generation of help text

### Consequences

- Dependency on the `clap` crate
- Consistent command-line interface
- Better user experience with helpful error messages
- Automatic help text generation
- Simplified command-line parsing code

## ADR-007: Testing Strategy

### Context

MCTL needs comprehensive testing to ensure reliability and correctness.

### Decision

Implement a multi-layered testing strategy:

1. Unit tests for individual modules
2. Integration tests for command workflows
3. Property-based testing for configuration and input validation
4. Mock dependencies for isolated testing
5. Test fixtures for git repositories

### Rationale

- **Coverage**: Comprehensive test coverage ensures reliability
- **Isolation**: Unit tests verify module behavior in isolation
- **Integration**: Integration tests verify end-to-end workflows
- **Properties**: Property-based tests verify behavior across input ranges
- **Fixtures**: Test fixtures provide realistic test scenarios

### Consequences

- More code for tests and test infrastructure
- Higher confidence in code correctness
- Easier refactoring with test safety net
- Better documentation of expected behavior
- Faster detection of regressions

## Conclusion

These architectural decisions provide the foundation for the MCTL system. They balance various concerns including maintainability, security, performance, and user experience. As the system evolves, these decisions may be revisited and updated based on new requirements or constraints.

Each decision is documented with its context, the decision itself, the rationale behind it, and the consequences of the decision. This documentation helps current and future developers understand why the system is designed the way it is and provides guidance for future changes.