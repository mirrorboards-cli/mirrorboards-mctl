# Contributing to MCTL

Thank you for considering contributing to MCTL! This document provides guidelines and instructions for contributing to the project.

## Development Setup

### Prerequisites

- Rust 1.65 or later
- Git 2.25 or later
- SSH client
- Cargo and rustup (latest versions recommended)
- An IDE with Rust support (VS Code with rust-analyzer recommended)

### Setting Up the Development Environment

1. **Clone the repository:**

   ```bash
   git clone https://github.com/example/mctl.git
   cd mctl
   ```

2. **Install development tools:**

   ```bash
   # Install development tools
   rustup component add clippy rustfmt
   cargo install cargo-audit cargo-watch cargo-tarpaulin
   ```

3. **Build the project:**

   ```bash
   cargo build
   ```

4. **Run tests:**

   ```bash
   cargo test
   ```

### Project Structure

The codebase follows a clean, layered architecture:

```
src/
├── application/        # Application layer: command implementations
│   └── commands/       # Individual command implementations
├── domain/             # Domain layer: core entities and interfaces
│   ├── configuration/  # Configuration structures
│   └── repository/     # Repository operations and interfaces
├── infrastructure/     # Infrastructure layer: external integrations
│   ├── config/         # Config loading and parsing
│   ├── filesystem/     # Filesystem operations
│   ├── git/            # Git operations implementation
│   └── logging/        # Logging implementation
├── presentation/       # Presentation layer: CLI and output formatting
└── main.rs             # Application entry point
```

## Code Style Guidelines

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Adhere to standard Rust idioms and patterns
- Use `rustfmt` for code formatting
- Run `cargo clippy` and address all warnings

### Specific Guidelines

1. **Documentation**
   - Document all public items (functions, structs, traits, modules)
   - Use Rustdoc-style comments (`///` for items, `//!` for modules)
   - Include examples for complex functions

2. **Error Handling**
   - Use `anyhow` for error context and propagation
   - Define domain-specific errors using `thiserror`
   - Provide helpful error messages with context

3. **Code Organization**
   - Keep functions focused and small
   - Use meaningful variable and function names
   - Maintain separation of concerns between layers

4. **File Format**
   - Use 4 spaces for indentation
   - Maximum line length of 100 characters
   - No trailing whitespace

### Pre-commit Checks

Run these checks before committing:

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test
```

## Testing Guidelines

### Test Structure

- **Unit tests**: Test individual components in isolation
- **Integration tests**: Test interactions between components
- **End-to-end tests**: Test command execution with mock repositories

### Guidelines

1. **Coverage**
   - Aim for at least 80% code coverage
   - All public interfaces must have tests

2. **Test Organization**
   - Place unit tests in the same file as the code they test
   - Place integration tests in a dedicated `tests/` directory
   - Use appropriate test fixtures and mocks

3. **Running Tests**

   ```bash
   # Run all tests
   cargo test

   # Run specific test
   cargo test test_name

   # Run tests with coverage report
   cargo tarpaulin
   ```

4. **Mocking**
   - Use `mockall` for creating mock implementations
   - Isolate external dependencies in tests

## Pull Request Process

### Creating a Pull Request

1. **Fork the repository** and create a new branch from `main`
2. **Make your changes** following the code style guidelines
3. **Add tests** for any new functionality
4. **Update documentation** if needed
5. **Run the pre-commit checks** to ensure your code meets the standards
6. **Submit a pull request** to the `main` branch

### PR Standards

1. **Descriptive title** that summarizes the changes
2. **Detailed description** including:
   - The problem being solved
   - How your changes solve the problem
   - Any design decisions or trade-offs made
   - Screenshots or examples for UI/UX changes

3. **Link to related issues** if applicable
4. **Breaking changes** should be clearly noted

### Review Process

1. All PRs require at least one review from a maintainer
2. Address all review comments
3. All tests must pass
4. CI checks must pass

### Commit Guidelines

- Use meaningful commit messages
- Follow the [Conventional Commits](https://www.conventionalcommits.org/) format:
  - `feat: add new feature`
  - `fix: resolve bug`
  - `docs: update documentation`
  - `test: add tests`
  - `refactor: improve code structure`

## Security Considerations

Given MCTL's focus on security and handling Git credentials:

1. Never commit sensitive information (keys, passwords, tokens)
2. Report security issues privately to maintainers
3. Be careful with SSH credential handling in code changes
4. Avoid introducing dependencies with security vulnerabilities

## Code of Conduct

Please be respectful and considerate when interacting with other contributors. We aim to foster an inclusive and welcoming community.

## License

By contributing to MCTL, you agree that your contributions will be licensed under the project's [MIT License](LICENSE).