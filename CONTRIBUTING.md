# Contributing to Mirror Workspace

Thank you for considering contributing to the Mirror Workspace project! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project. We aim to foster an inclusive and welcoming community.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/mirror-workspace.git`
3. Create a new branch for your changes: `git checkout -b your-branch-name`
4. Make your changes
5. Run tests: `cargo test --workspace`
6. Commit your changes: `git commit -m "Description of changes"`
7. Push to your fork: `git push origin your-branch-name`
8. Create a pull request

## Development Environment

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)
- Git

### Setting Up

```bash
# Clone the repository
git clone https://github.com/mirrorboards/mirror-workspace.git
cd mirror-workspace

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace
```

## Project Structure

```
mirror-workspace/
├── .github/workflows/    # GitHub Actions workflows
├── docs/                 # Documentation
├── examples/             # Examples
├── mirror-sdk/           # Core SDK library
├── mirror-cli/           # Command-line interface
├── tests/                # Integration tests
├── Cargo.toml            # Workspace configuration
├── README.md             # Project overview
└── CONTRIBUTING.md       # This file
```

## Coding Standards

### Rust Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` to format your code
- Use `cargo clippy` to check for common mistakes and improve your code
- Write documentation comments for public API items
- Keep functions small and focused on a single task
- Use meaningful variable and function names

### Commit Messages

- Use clear and descriptive commit messages
- Start with a short summary (50 chars or less)
- Optionally, follow with a blank line and a more detailed explanation
- Reference issues and pull requests where appropriate

## Testing

- Write tests for all new features and bug fixes
- Ensure all tests pass before submitting a pull request
- Include both unit tests and integration tests where appropriate
- Use descriptive test names that explain what is being tested

```bash
# Run all tests
cargo test --workspace

# Run specific tests
cargo test --package mirror-sdk
cargo test --package mirror-cli
cargo test --test integration_test
```

## Documentation

- Update documentation for all new features and changes
- Write clear and concise documentation
- Include examples for complex features
- Keep the API reference up-to-date

```bash
# Generate documentation
cargo doc --workspace --no-deps --open
```

## Pull Request Process

1. Ensure your code follows the coding standards
2. Update documentation as necessary
3. Add or update tests as necessary
4. Ensure all tests pass
5. Submit your pull request with a clear description of the changes
6. Wait for review and address any feedback

## Release Process

1. Update version numbers in Cargo.toml files
2. Update CHANGELOG.md with the changes in the new version
3. Create a new release on GitHub with release notes
4. Publish to crates.io (if applicable)

## Getting Help

If you need help with contributing, please:

- Open an issue with your question
- Reach out to the maintainers
- Check the documentation and existing issues

Thank you for contributing to Mirror Workspace!