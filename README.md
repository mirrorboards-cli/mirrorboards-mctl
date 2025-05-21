# MCTL - Mirror Control

MCTL (Mirror Control) is a command-line interface (CLI) tool for efficient git repository synchronization and mirroring. It allows you to manage multiple git repositories through a single configuration file, making it easy to clone, update, and maintain mirrors of your repositories.

## Features

- **Centralized Configuration**: Manage all your repositories in a single TOML file
- **Efficient Synchronization**: Clone and update multiple repositories with a single command
- **Status Monitoring**: Check the status of all repositories at once
- **Batch Operations**: Commit and push changes across multiple repositories
- **Secure Credential Handling**: Support for SSH keys and environment variables
- **Extensible Design**: Modular architecture for easy extension

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/mctl.git
cd mctl

# Build the project
cargo build --release

# Install the binary
cargo install --path .
```

### Using Cargo

```bash
cargo install mctl
```

## Quick Start

1. Create a `mirror.toml` file in your project directory:

```toml
# mirror.toml
[[repositories]]
git-url = "git@github.com:example/repo1.git"
path = "repo1"

[[repositories]]
git-url = "git@github.com:example/repo2.git"
path = "repo2"
branch = "develop"
```

2. Sync all repositories:

```bash
mctl sync
```

3. Check status of all repositories:

```bash
mctl status
```

## Usage

### Adding a Repository

```bash
# Add a repository with explicit options
mctl add --git-url git@github.com:example/repo.git --path ./example-repo --branch main

# Add a repository with short syntax
mctl add git@github.com:example/repo.git ./example-repo --branch main
```

### Syncing Repositories

```bash
# Sync all repositories using default configuration
mctl sync

# Sync with custom configuration file
mctl sync --config custom-mirror.toml

# Sync to a custom destination directory
mctl sync --dest ./repos

# Skip pulling updates for existing repositories
mctl sync --no-pull

# Sync repositories in parallel
mctl sync --parallel 4
```

### Checking Status

```bash
# Check status of all repositories
mctl status

# Check status with verbose output
mctl status --verbose

# Check status with custom configuration file
mctl status --config custom-mirror.toml
```

### Saving Changes

```bash
# Save changes with default commit message
mctl save

# Save changes with custom commit message
mctl save --message "Update configuration files"

# Save changes with custom commit message (short form)
mctl save "Update configuration files"
```

### Updating Repositories

```bash
# Update all repositories
mctl update

# Update with verbose output
mctl update --verbose

# Force update even with potential conflicts
mctl update --force

# Perform a dry run without making changes
mctl update --dry-run

# Update a specific repository
mctl update --repo example-repo
```

## Configuration

MCTL uses a TOML configuration file (`mirror.toml` by default) to define repositories and their properties.

### Basic Configuration

```toml
# Optional global settings
base_path = "./repos"
default_branch = "main"

# Repository definitions
[[repositories]]
git-url = "git@github.com:example/repo.git"
path = "example-repo"
branch = "main"

[[repositories]]
git-url = "https://github.com/example/docs.git"
path = "docs"
branch = "develop"
```

### Configuration Options

| Option | Description |
|--------|-------------|
| `base_path` | Base directory for all repositories (optional) |
| `default_branch` | Default branch to use if not specified (optional) |
| `repositories` | Array of repository configurations |

#### Repository Options

| Option | Description | Required |
|--------|-------------|----------|
| `git-url` | Git URL of the repository | Yes |
| `path` | Local path where the repository will be cloned | Yes |
| `branch` | Specific branch to track | No |
| `name` | Custom name for the repository | No |

## Environment Variables

MCTL supports the following environment variables:

| Variable | Purpose | Example |
|----------|---------|---------|
| `GIT_USERNAME` | Username for HTTPS authentication | `export GIT_USERNAME=your-username` |
| `GIT_PASSWORD` | Password or token for HTTPS authentication | `export GIT_PASSWORD=your-token` |
| `MCTL_CONFIG_PATH` | Default configuration file path | `export MCTL_CONFIG_PATH=/path/to/config.toml` |
| `MCTL_LOG_LEVEL` | Control verbosity (debug, info, warn, error) | `export MCTL_LOG_LEVEL=debug` |

## Security Considerations

- **SSH Keys**: Use SSH keys with passphrases for enhanced security
- **Environment Variables**: Use environment variables for credentials instead of hardcoding them
- **Credential Helpers**: Leverage git credential helpers for secure credential storage
- **Token Permissions**: When using access tokens, limit permissions to the minimum required

## Best Practices

1. **Keep Configuration in Version Control**: Store your `mirror.toml` file in version control
2. **Regular Status Checks**: Run `mctl status` before and after operations
3. **Clean Repositories**: Commit or stash local changes before running sync or update
4. **Selective Updates**: Use the `--repo` option for targeted updates
5. **Dry Runs**: Use `--dry-run` to preview changes before applying them

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Inspired by the need for efficient management of multiple git repositories
- Built with Rust for performance and reliability