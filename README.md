# MirrorBoards Repository Management

A comprehensive Rust-based solution for managing Git repositories through mirror.toml configuration files.

## Overview

This workspace contains two main components:

1. **mirror-sdk**: A Rust library that provides a programmatic API for managing mirror.toml configuration files.
2. **mirror-cli**: A command-line interface built on top of the mirror-sdk for easy repository management.

Together, these components provide a powerful and flexible system for managing multiple Git repositories through a single configuration file.

## Purpose

The MirrorBoards Repository Management system is designed to solve the problem of managing multiple related Git repositories in a consistent and organized way. It allows you to:

- Define repository configurations in a single mirror.toml file
- Track multiple repositories with their origins, branches, and local paths
- Categorize repositories with tags for better organization
- Validate configurations to ensure consistency
- Manage repositories through both a programmatic API and a command-line interface

## Component Relationship

```
┌─────────────────┐     ┌─────────────────┐
│                 │     │                 │
│   mirror-cli    │────▶│   mirror-sdk    │
│  (CLI Interface)│     │  (Core Library) │
│                 │     │                 │
└─────────────────┘     └─────────────────┘
         │                      │
         │                      │
         ▼                      ▼
┌─────────────────────────────────────────┐
│                                         │
│            mirror.toml file             │
│                                         │
└─────────────────────────────────────────┘
```

- **mirror-sdk**: The core library that provides the fundamental functionality for parsing, validating, and manipulating mirror.toml configuration files.
- **mirror-cli**: A user-friendly command-line interface that leverages the mirror-sdk to provide easy access to repository management operations.

## Quick Start

### Installation

#### From Source

```bash
# Clone the repository
git clone https://github.com/mirrorboards/mirror-workspace.git
cd mirror-workspace

# Build the project
cargo build --release

# The binary will be available at target/release/mirror-cli
```

### Basic Usage

#### Using the CLI

```bash
# Initialize a new mirror.toml file
mirror-cli init

# Add a repository
mirror-cli add --origin "git@github.com:example/repo.git" --path "example/repo"

# List all repositories
mirror-cli list

# Remove a repository
mirror-cli remove --path "example/repo"
```

#### Using the SDK in Your Rust Code

```rust
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};

fn main() -> Result<(), MirrorError> {
    // Create a new SDK instance
    let sdk = MirrorSdk::new();
    
    // Load an existing configuration
    let mut config = sdk.load_config("mirror.toml")?;
    
    // Create a new repository
    let repo = RepositoryBuilder::new()
        .origin("git@github.com:example/repo.git")
        .branch("main")
        .path("example/repo")
        .tag("example")
        .build()?;
    
    // Add the repository to the configuration
    sdk.add_repository(&mut config, repo)?;
    
    // Save the updated configuration
    sdk.save_config(&config, "mirror.toml")?;
    
    Ok(())
}
```

## Features

### mirror-sdk

- Parse and serialize mirror.toml configuration files
- Manage repository configurations (add, remove, update)
- Support file system operations for working with repositories
- Handle configuration through default paths and environment variables
- Provide a comprehensive error handling strategy
- Expose a clean, well-documented public API

### mirror-cli

- Create new mirror.toml configuration files
- Add, remove, and update repositories
- List repositories with optional tag filtering
- Validate mirror.toml configurations
- Colorful terminal output for better user experience
- Specify mirror.toml file path via command-line argument or environment variable

## Documentation

Detailed documentation is available in the [docs](./docs) directory:

- [SDK API Reference](./docs/sdk-api-reference.md)
- [CLI Command Reference](./docs/cli-command-reference.md)
- [Configuration File Format](./docs/configuration-format.md)
- [Common Workflows](./docs/common-workflows.md)

## Examples

Check out the [examples](./examples) directory for sample code and usage patterns.

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run integration tests
cargo test --test integration
```

### Building Documentation

```bash
# Generate documentation
cargo doc --workspace --no-deps --open
```

## License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.