# Decision Log

This file records architectural and implementation decisions using a list format.
2025-05-15 10:36:31 - Initial creation for MCTL architecture redesign.

## Decision: Adopt Layered Architecture Pattern

* **Rationale**: 
  - Provides clear separation of concerns
  - Facilitates unit testing by isolating components
  - Makes the system more maintainable and extensible
  - Allows for dependency inversion where appropriate

* **Implementation Details**:
  - Presentation Layer: CLI interface, command parsing, user output formatting
  - Application Layer: Command implementations, orchestration, business logic
  - Domain Layer: Core entities, repository interfaces, domain services
  - Infrastructure Layer: Git operations, filesystem access, configuration management

## Decision: Use Trait-Based Interface Design

* **Rationale**:
  - Enables dependency injection for better testability
  - Provides clear interface contracts between layers
  - Allows for multiple implementations (real vs mock)
  - Aligns with Rust's zero-cost abstraction philosophy

* **Implementation Details**:
  - Define traits for all service interfaces
  - Implement concrete services that fulfill these traits
  - Use dependency injection in higher-level components
  - Leverage Rust's trait bounds for compile-time guarantees

## Decision: Implement Comprehensive Error Handling

* **Rationale**:
  - Military-grade reliability requires robust error handling
  - Users need actionable error messages
  - System should be resilient to various failure modes
  - Debugging requires detailed error context

* **Implementation Details**:
  - Create domain-specific error types using thiserror
  - Implement context-preserving error propagation
## Decision: SSH Authentication for Git Operations

* **Rationale**:
  - SSH is the industry standard for secure Git authentication
  - Military-grade security requires proper credential handling
  - System Git ensures consistent authentication behavior with user's existing setup
  - Avoids storing or managing credentials within the application

* **Implementation Details**:
  - Use system Git command with proper environment variables
  - Implement GitSshHandler to configure SSH behavior
  - Support custom SSH key paths as optional configuration
  - Proper error handling for authentication failures with clear guidance

## Decision: Thread Pool Based Parallel Processing 

* **Rationale**:
  - Thread pools provide controlled parallelism with resource management
  - Repository operations are mostly I/O bound, benefiting from parallelism
  - Structured approach enables progress tracking and error handling
  - Scales well with increasing repository count

* **Implementation Details**:
  - Implement RepositoryOrchestrator with configurable thread pool
  - Use channels for progress reporting and result collection
  - Provide atomic counters for thread-safe progress tracking
  - Allow configuration of concurrency levels
  - Provide user-friendly error messages at the presentation layer
  - Log detailed error information for debugging
## Decision: Bottom-Up Implementation Approach

* **Rationale**:
  - Infrastructure components (especially Git operations) present the highest technical risk
  - Early implementation of core functionality enables iterative testing
  - Domain and application layers depend on infrastructure interfaces
  - Allows for refinement of interfaces based on implementation experience
  - Enables parallel work on different layers once core interfaces are stable

* **Implementation Details**:
  - Start with Git Module and SSH authentication implementation
  - Implement FileSystem Module and Config Provider in parallel
  - Build domain layer on top of stable infrastructure interfaces
  - Implement application layer commands progressively
  - Develop presentation layer once application layer is functional
  - Follow critical path focusing on repository operations

## Decision: Phased Testing Strategy

* **Rationale**:
  - Military-grade quality requires comprehensive testing at all levels
  - Different components require different testing approaches
  - Testing should validate both individual components and their interactions
  - Performance testing is essential for parallel processing capabilities

* **Implementation Details**:
  - Unit tests for all components with >90% coverage
  - Integration tests for layer interactions
  - End-to-end tests for complete workflows
  - Performance testing for parallel processing
  - Mock dependencies for isolated testing
## Decision: Enhanced Configuration Format

* **Rationale**:
  - Current configuration is minimal and lacks critical functionality
  - Military-grade tool requires comprehensive configuration options
  - SSH authentication needs explicit configuration support
  - Repository operations need fine-grained control
  - Users need flexibility for different environments and use cases

* **Implementation Details**:
  - Use TOML format with nested structure for clarity and organization
  - Provide global settings with repository-specific overrides
  - Explicitly support SSH authentication configuration
  - Add command-specific settings for fine-grained control
  - Support environment variable substitution and path expansion
  - Implement layered configuration loading (system, user, local)
  - Maintain backward compatibility with minimal configuration format
  - Add validation for configuration entries
  - Continuous testing throughout development