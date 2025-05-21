# mctl - Mirror Control Tool

A Rust CLI tool for synchronizing git repositories to local directories based on a configuration file.

## Overview

`mctl` is a command-line tool that reads a configuration file (`mirror.toml`) and synchronizes git repositories to local directories. It can clone new repositories or update existing ones, making it easy to maintain a consistent local mirror of multiple git repositories.

## Features

- Read repository configurations from a TOML file
- Clone repositories that don't exist locally
- Update existing repositories with the latest changes
- Support for specifying branches
- Proper error handling and logging
- Single file implementation

## Installation

### Prerequisites

- Rust and Cargo (install from [rustup.rs](https://rustup.rs/))
- Git command-line tool

### Building from Source

1. Clone this repository
2. Build the project:

```bash
cargo build --release
```

3. The binary will be available at `target/release/mctl`

## Usage

```bash
# Synchronize repositories using the default mirror.toml in the current directory
mctl sync

# Specify a custom configuration file
mctl sync --config path/to/mirror.toml

# Enable verbose logging
mctl sync --verbose
```

## Configuration File Format

The configuration file (`mirror.toml`) should contain a list of repositories to synchronize. Each repository entry should specify:

- `origin`: The git URL of the repository
- `path`: The local path where the repository should be cloned/updated
- `branch` (optional): The branch to checkout

Example `mirror.toml`:

```toml
[[repositories]]
origin = "git@github.com:username/repo1.git"
path = "path/to/local/repo1"

[[repositories]]
origin = "git@github.com:username/repo2.git"
path = "path/to/local/repo2"
branch = "develop"

# Commented repositories are skipped
# [[repositories]]
# origin = "git@github.com:username/repo3.git"
# path = "path/to/local/repo3"
```

## License

[MIT License](LICENSE)