# MCTL - Military-Grade Multi-Repository Management

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

MCTL (Mirror Control) is a robust, secure command-line tool for efficiently managing multiple Git repositories with military-grade quality and security. It enables centralized control of repository operations across your entire codebase with a focus on security, reliability, and performance.

## Key Features

- **Secure SSH Authentication**
  - Full support for SSH key-based authentication
  - Repository-specific SSH key configuration
  - Intelligent error handling with detailed diagnostic messages
  - Passphrase support via command or SSH agent

- **Parallel Processing**
  - Concurrent operations across multiple repositories
  - Configurable thread limits and timeouts
  - Progress tracking and reporting

- **Comprehensive Repository Management**
  - Clone multiple repositories with a single command
  - Check status across all repositories simultaneously
  - Commit and push changes in multiple repositories
  - Repository filtering by tags

- **Robust Configuration**
  - TOML-based configuration with rich options
  - Environment variable substitution
  - Command-specific settings
  - Repository-specific overrides

- **Clean, Modular Architecture**
  - Layered design for maintainability and extensibility
  - Comprehensive error handling
  - Detailed logging

## Installation

### From Cargo (Recommended)

```bash
cargo install mctl
```

### From Source

```bash
git clone https://github.com/example/mctl.git
cd mctl
cargo build --release
```

The binary will be available at `./target/release/mctl`. You can copy it to a directory in your PATH:

```bash
# Linux/macOS
sudo cp ./target/release/mctl /usr/local/bin/

# Or add to your personal bin directory
cp ./target/release/mctl ~/.local/bin/
```

### Prerequisites

- Rust 1.65 or later
- Git 2.25 or later (with SSH support)
- SSH client configured on your system

## Usage

### Initializing a Configuration

Create a new configuration file with default settings:

```bash
mctl init --output mirror.toml
```

Specify a custom SSH key:

```bash
mctl init --output mirror.toml --ssh-key ~/.ssh/custom_key
```

Force overwrite of an existing configuration:

```bash
mctl init --output mirror.toml --force
```

### Synchronizing Repositories

Clone or update all repositories defined in your configuration:

```bash
mctl sync
```

Synchronize only repositories with specific tags:

```bash
mctl sync --tags backend,infrastructure
```

Limit concurrency:

```bash
mctl sync --max-threads 4
```

### Checking Repository Status

Check the status of all repositories:

```bash
mctl status
```

Get detailed status including untracked files:

```bash
mctl status --untracked
```

Filter repositories by tags:

```bash
mctl status --tags frontend
```

### Saving Changes

Commit and push changes across all repositories:

```bash
mctl save --message "Update all repositories"
```

Commit only (without pushing):

```bash
mctl save --message "Update all repositories" --no-push
```

Target specific repositories:

```bash
mctl save --message "Update backend services" --tags backend
```

## Configuration

MCTL uses a TOML configuration file to define repositories and settings. See [Configuration Format](docs/configuration-format.md) for complete documentation.

Example configuration:

```toml
# Global settings applied to all repositories
[global]
parallel = true
max_threads = 8

# SSH authentication settings
[auth.ssh]
key_path = "~/.ssh/id_rsa"
known_hosts_path = "~/.ssh/known_hosts"

# Logging configuration
[logging]
level = "info"
file = "~/.mctl/mctl.log"
format = "text"

# Command-specific settings
[commands.sync]
recursive = true
depth = 1

# Repository definitions
[[repositories]]
path = "~/projects/service-api"
origin = "git@github.com:company/service-api.git"
branch = "main"
tags = ["backend", "api"]

# Repository with specific settings
[[repositories]]
path = "~/projects/frontend"
origin = "git@github.com:company/frontend.git"
tags = ["frontend"]

[repositories.auth.ssh]
key_path = "~/.ssh/frontend_deploy_key"
```

## Architecture

MCTL is designed with a clean, layered architecture:

1. **Presentation Layer**: CLI interface, command parsing, output formatting
2. **Application Layer**: Command orchestration, business logic
3. **Domain Layer**: Core entities, repository operations, interfaces
4. **Infrastructure Layer**: Git integration, filesystem operations, logging

This modular design ensures maintainability, testability, and extensibility.

## License

[MIT](LICENSE)