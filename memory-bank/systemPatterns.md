# System Patterns

This file documents recurring patterns and standards used in the project.
It is updated as the project evolves.
2025-05-15 10:36:45 - Initial creation for MCTL architecture.

## Coding Patterns

* **Trait-Based Architecture**:
  - Service interfaces defined as traits
  - Concrete implementations separated from interface definitions
  - Dependency injection via trait objects or generic constraints
  - Mock implementations for testing

* **Error Handling Pattern**:
  - Domain-specific error types using thiserror
  - Context-preserving error propagation using anyhow
  - Error conversion between layers with additional context
  - Result type usage throughout with ? operator

* **Command Pattern**:
  - Commands encapsulated in their own modules
  - Common command trait with execute method
  - Command factory for instantiation
  - Command-specific validation logic

## Architectural Patterns

* **Layered Architecture**:
  - Presentation → Application → Domain → Infrastructure
  - Dependency inversion for infrastructure services
  - Clean separation between layers with clear interfaces
  - Unidirectional dependencies (higher layers depend on lower ones)

* **Repository Pattern**:
  - Abstract repository interfaces in domain layer
  - Concrete implementations in infrastructure layer
  - Dependency injection for repository instances
  - Common operations standardized across repositories

* **Parallel Processing**:
  - Task-based parallelism with thread pools
  - Non-blocking I/O where applicable
  - Thread-safe shared state
  - Progress reporting from parallel operations

## Testing Patterns

* **Unit Testing**:
  - Mock dependencies using trait-based DI
  - High coverage of core domain logic
  - Separation of pure logic from side effects
  - Table-driven tests for edge cases

* **Integration Testing**:
  - Test real components working together
  - Mock external systems (Git) when appropriate
  - Validate command workflows end-to-end
  - Verify error handling across component boundaries