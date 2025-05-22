# Mirror CLI

A command-line interface for managing mirror.toml configuration files.

## Features

- Initialize new mirror.toml files
- Add repositories to mirror.toml
- Remove repositories from mirror.toml
- List repositories in mirror.toml
- Update repository configurations
- Support for custom paths and environment variables
- Automatic repository ID generation

## Installation

```bash
# Clone the repository
git clone https://github.com/mirrorboards/mirror-sdk.git
cd mirror-sdk

# Build the CLI
cargo build --release

# Install the CLI (optional)
cargo install --path mirror-cli
```

## Usage

### Basic Commands

```bash
# Show help
mirror --help

# Show version
mirror --version

# Show help for a specific command
mirror repo --help
```

### Repository Management

```bash
# Add a repository
mirror repo add git@github.com:example/repo.git ./example/repo

# Add a repository with options
mirror repo add --branch main --lock --tags tag1,tag2 git@github.com:example/repo.git ./example/repo

# Remove a repository by ID
mirror repo remove --id repo-id

# Remove a repository by path
mirror repo remove --path ./example/repo

# List all repositories
mirror repo list

# List repositories with detailed information
mirror repo list --detailed

# List repositories with a specific tag
mirror repo list --tag tag1

# Update a repository
mirror repo update repo-id --origin git@github.com:new/repo.git --path ./new/path
```

### Configuration Path

By default, the CLI looks for a `mirror.toml` file in the current directory. You can specify a custom path:

```bash
# Use a custom configuration file
mirror repo list --config /custom/path/mirror.toml
```

Or use an environment variable:

```bash
# Set MIRROR_CONFIG_PATH environment variable
export MIRROR_CONFIG_PATH=/custom/path/mirror.toml

# Commands will use the environment variable
mirror repo list
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.