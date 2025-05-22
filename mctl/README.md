# Mirror Control (mctl)

A powerful command-line interface tool for managing mirror.toml configuration files.

## Overview

Mirror Control (mctl) is a CLI tool that leverages the mirror-sdk to manage repositories defined in mirror.toml files. It provides a comprehensive set of commands for initializing configuration files, adding and managing repositories, organizing repositories with tags, and configuring tool behavior.

## Features

- Initialize mirror.toml configuration files
- Add, remove, update, list, and show repositories
- Tag repositories for better organization
- Filter repositories by tags or paths
- Customize output format (human-readable, JSON, table)
- Color-coded output for better readability
- Comprehensive error messages with suggestions

## Installation

### From GitHub (Recommended)

```bash
# Install directly from GitHub repository
cargo install --git ssh://git@github.com/mirrorboards/mirrorboards-mctl.git --path mctl --config net.git-fetch-with-cli=true
```

### From Source

```bash
# Clone the repository
git clone https://github.com/mirrorboards/mirrorboards-mctl.git
cd mirrorboards-mctl/mctl

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

```bash
# Initialize a new mirror.toml file
mctl init

# Add a repository
mctl repo add https://github.com/example/repo.git ./src/example

# List all repositories
mctl repo list

# Add tags to a repository
mctl tag add <repo-id> frontend important

# List repositories with a specific tag
mctl repo list --tag frontend
```

## Command Structure

The `mctl` CLI follows a Git-style command structure with the following main commands:

| Command   | Description                                      |
|-----------|--------------------------------------------------|
| `init`    | Initialize a new mirror.toml file                |
| `repo`    | Manage repositories (add, remove, update, list, show) |
| `tag`     | Manage repository tags                           |
| `config`  | Manage configuration settings                    |
| `help`    | Display help information                         |
| `version` | Display version information                      |

### Global Options

These options can be used with any command:

- `-c, --config <PATH>`: Path to the mirror.toml file
- `-v, --verbose`: Enable verbose output
- `-q, --quiet`: Enable quiet mode (minimal output)
- `--color <WHEN>`: Control when to use colored output (always, auto, never)
- `-h, --help`: Print help information
- `-V, --version`: Print version information

## Detailed Usage

### Initialize a new mirror.toml file

```bash
mctl init [--path PATH] [--force]
```

Options:
- `-p, --path <PATH>`: Specify the path where the mirror.toml file should be created
- `-f, --force`: Overwrite existing mirror.toml file if it exists

Examples:
```bash
# Initialize in the current directory
mctl init

# Initialize at a specific path
mctl init --path /path/to/project

# Force overwrite of existing file
mctl init --force
```

### Repository Management

#### Add a repository

```bash
mctl repo add <ORIGIN> <PATH> [--id ID] [--branch BRANCH] [--tag TAG...] [--lock]
```

Arguments:
- `<ORIGIN>`: Git repository URL (required)
- `<PATH>`: Local path where the repository should be cloned (required)

Options:
- `-i, --id <ID>`: Specify a custom ID for the repository
- `-b, --branch <BRANCH>`: Specify the branch to use (defaults to "main")
- `-t, --tag <TAG>...`: Add tags to the repository (can be specified multiple times)
- `-l, --lock`: Lock the repository

Examples:
```bash
# Add a repository with auto-generated ID
mctl repo add git@github.com:user/repo.git path/to/clone

# Add a repository with custom ID and branch
mctl repo add --id custom-id --branch develop git@github.com:user/repo.git path/to/clone

# Add a repository with tags
mctl repo add --tag frontend --tag important git@github.com:user/repo.git path/to/clone
```

#### Remove a repository

```bash
mctl repo remove <ID> [--force]
```

Arguments:
- `<ID>`: ID of the repository to remove (required)

Options:
- `--force`: Force removal without confirmation

Examples:
```bash
# Remove a repository (with confirmation)
mctl repo remove repo-id

# Force remove without confirmation
mctl repo remove repo-id --force
```

#### Update a repository

```bash
mctl repo update <ID> [--origin ORIGIN] [--path PATH] [--branch BRANCH] [--lock LOCK]
```

Arguments:
- `<ID>`: ID of the repository to update (required)

Options:
- `-o, --origin <ORIGIN>`: Update the Git repository URL
- `-p, --path <PATH>`: Update the local path
- `-b, --branch <BRANCH>`: Update the branch
- `-l, --lock <true/false>`: Update the lock status

Examples:
```bash
# Update the branch of a repository
mctl repo update repo-id --branch main

# Update multiple properties
mctl repo update repo-id --path new/path --lock true
```

#### List repositories

```bash
mctl repo list [--tag TAG] [--path PATH] [--json]
```

Options:
- `-t, --tag <TAG>`: Filter repositories by tag
- `--path <PATH>`: Filter repositories by path prefix
- `-j, --json`: Output in JSON format

Examples:
```bash
# List all repositories
mctl repo list

# List repositories with a specific tag
mctl repo list --tag frontend

# List repositories in JSON format
mctl repo list --json

# List repositories in a specific directory
mctl repo list --path projects/
```

#### Show repository details

```bash
mctl repo show <ID>
```

Arguments:
- `<ID>`: ID of the repository to show (required)

Examples:
```bash
# Show details of a specific repository
mctl repo show repo-id
```

### Tag Management

#### Add tags to a repository

```bash
mctl tag add <ID> <TAG>...
```

Arguments:
- `<ID>`: ID of the repository to add tags to (required)
- `<TAG>...`: Tags to add (required, can specify multiple)

Examples:
```bash
# Add a single tag
mctl tag add repo-id frontend

# Add multiple tags
mctl tag add repo-id frontend important
```

#### Remove tags from a repository

```bash
mctl tag remove <ID> <TAG>...
```

Arguments:
- `<ID>`: ID of the repository to remove tags from (required)
- `<TAG>...`: Tags to remove (required, can specify multiple)

Examples:
```bash
# Remove a single tag
mctl tag remove repo-id frontend

# Remove multiple tags
mctl tag remove repo-id frontend important
```

#### List all tags

```bash
mctl tag list [--json]
```

Options:
- `-j, --json`: Output in JSON format

Examples:
```bash
# List all tags
mctl tag list

# List all tags in JSON format
mctl tag list --json
```

### Configuration Management

#### Set a configuration option

```bash
mctl config set <NAME> <VALUE>
```

Arguments:
- `<NAME>`: Name of the configuration option
- `<VALUE>`: Value to set

Examples:
```bash
# Set default branch
mctl config set default_branch main

# Set default tag
mctl config set default_tag production
```

#### Get a configuration option

```bash
mctl config get <NAME>
```

Arguments:
- `<NAME>`: Name of the configuration option

Examples:
```bash
# Get default branch
mctl config get default_branch
```

#### List all configuration options

```bash
mctl config list [--json]
```

Options:
- `-j, --json`: Output in JSON format

Examples:
```bash
# List all configuration options
mctl config list

# List all configuration options in JSON format
mctl config list --json
```

## Troubleshooting

### Common Issues

#### Repository Not Found

If you get an error that a repository ID was not found, check that:
- You're using the correct ID
- You're using the correct mirror.toml file
- The repository exists in the file

```bash
# List all repositories to see available IDs
mctl repo list
```

#### Permission Denied

If you get a permission denied error when accessing a file:
- Check that you have the necessary permissions
- Try running with elevated privileges if necessary

#### Invalid Configuration

If you get an error about invalid configuration:
- Check the format of your mirror.toml file
- Try initializing a new file with `mctl init`

### Error Messages

mctl provides detailed error messages with suggestions for resolution. For example:

```
Error: Failed to add repository
  Problem: Repository with ID 'custom-id' already exists
  Solution: Use a different ID or update the existing repository
  Command: mctl repo update custom-id --origin git@github.com:user/repo.git
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT