```bash
cargo install --git ssh://git@github.com/mirrorboards/mirrorboards-mctl.git --config net.git-fetch-with-cli=true
```

# mctl - Mirror Configuration Management Tool

A Rust-based CLI tool and SDK for managing `mirror.toml` configuration files that define collections of git repositories for large-scale IT projects.

## Features

- **CLI Tool**: Command-line interface for managing mirror configurations
- **Rust SDK**: Library for programmatic access to mirror.toml files
- **Multiple URL Formats**: Support for both SSH and HTTPS git URLs
- **Unique Hash IDs**: Generate unique identifiers for repositories
- **Configuration Validation**: Comprehensive validation and error checking
- **JSON Output**: Machine-readable output for automation

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/mirrorboards/mctl.git
cd mctl

# Build and install
cargo install --path mctl
```

### Using Cargo

```bash
cargo install mctl
```

## Quick Start

```bash
# Initialize a new mirror configuration
mctl init

# Add repositories
mctl add git@github.com:org/repo.git
mctl add https://github.com/external/lib.git --branch v2.0 --skip-push

# List all repositories
mctl list

# Show detailed information
mctl show a1b2c3d4

# Remove a repository
mctl remove a1b2c3d4

# Validate configuration
mctl validate
```

## Configuration Format

The `mirror.toml` file uses the following format:

```toml
[[repositories]]
git = "git@github.com:org/repo.git"
path = "org/repo"
branch = "main"        # optional, defaults to "main"
skip-push = false      # optional, defaults to false

[[repositories]]
git = "https://github.com/external/readonly.git"
path = "external/readonly"
branch = "v2.1"
skip-push = true       # read-only repository
```

## CLI Commands

### `mctl init`

Initialize a new `mirror.toml` configuration file in the current directory.

```bash
mctl init                    # Create new file
mctl init --force            # Overwrite existing file
```

### `mctl add`

Add a repository to the configuration.

```bash
mctl add <git-url>                           # Use defaults
mctl add <git-url> --path custom/path        # Custom local path
mctl add <git-url> --branch develop          # Custom branch
mctl add <git-url> --skip-push               # Read-only repository
```

**Examples:**
```bash
mctl add git@github.com:org/repo.git
mctl add https://github.com/external/lib.git --branch v2.0 --skip-push --path libs/external
```

### `mctl list`

List all repositories with their hash IDs.

```bash
mctl list                    # Table format
mctl list --json             # JSON format
```

### `mctl remove`

Remove a repository by its hash ID (supports partial matching).

```bash
mctl remove <hash>           # Interactive confirmation
mctl remove <hash> --force   # Skip confirmation
```

### `mctl show`

Show detailed information about a repository.

```bash
mctl show <hash>             # Show repository details
```

### `mctl validate`

Validate the configuration file.

```bash
mctl validate                # Basic validation
mctl validate --detailed     # Detailed validation report
```

## Global Options

- `--config <file>`: Use custom configuration file (default: `mirror.toml`)
- `--verbose`: Enable verbose output
- `--help`: Show help information

## Examples

### Basic Workflow

```bash
# Initialize configuration
mctl init

# Add some repositories
mctl add git@github.com:myorg/backend.git
mctl add git@github.com:myorg/frontend.git --branch develop
mctl add https://github.com/external/library.git --skip-push

# List repositories
mctl list
# Output:
# Hash     | Git URL                              | Path            | Branch  | Skip Push
# ---------|--------------------------------------|-----------------|---------|----------
# a1b2c3d4 | git@github.com:myorg/backend.git     | myorg/backend   | main    | ✗
# e5f6g7h8 | git@github.com:myorg/frontend.git    | myorg/frontend  | develop | ✗
# i9j0k1l2 | https://github.com/external/lib.git  | external/lib    | main    | ✓

# Show details for a specific repository
mctl show a1b2

# Validate the configuration
mctl validate --detailed

# Remove a repository
mctl remove i9j0k1l2
```

### JSON Integration

```bash
# Export configuration as JSON for automation
mctl list --json > repositories.json

# Use with jq for filtering
mctl list --json | jq '.[] | select(.skip_push == false)'
```

### Custom Configuration Files

```bash
# Use different configuration files for different projects
mctl --config backend.toml init
mctl --config backend.toml add git@github.com:org/api.git

mctl --config frontend.toml init  
mctl --config frontend.toml add git@github.com:org/ui.git
```

## URL Format Support

mctl supports various git URL formats and automatically extracts the appropriate path:

| URL Format | Example | Extracted Path |
|------------|---------|----------------|
| SSH | `git@github.com:org/repo.git` | `org/repo` |
| HTTPS | `https://github.com/org/repo.git` | `org/repo` |
| GitLab | `git@gitlab.com:group/subgroup/repo.git` | `group/subgroup/repo` |
| Custom Host | `git@git.company.com:team/project.git` | `team/project` |

## Hash IDs

Each repository gets a unique 8-character hash ID based on all its metadata (git URL, path, branch, and skip-push setting). This allows for:

- **Unique Identification**: No two repositories with different configurations will have the same hash
- **Partial Matching**: You can use just the first few characters (minimum 4) for commands
- **Change Detection**: Hash changes when any repository metadata changes

## Rust SDK

The `mirror-sdk` crate provides programmatic access to mirror configurations:

```rust
use mirror_sdk::{ConfigManager, Repository, MirrorConfig};

// Load configuration
let manager = ConfigManager::new("mirror.toml");
let config = manager.load()?;

// Create new repository
let repo = Repository::from_url("git@github.com:org/repo.git".to_string())?;
println!("Hash: {}", repo.compute_hash());

// Add to configuration
manager.add_repository(repo)?;

// List all repositories
let repositories = manager.list_repositories()?;
for repo in repositories {
    println!("{}: {}", repo.compute_hash(), repo.git);
}
```

## Development

### Building

```bash
# Build both crates
cargo build

# Run tests
cargo test

# Build with optimizations
cargo build --release
```

### Project Structure

```
mctl/
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── DESIGN.md               # Technical specification
├── mctl/                   # CLI binary crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── cli.rs          # Command definitions
│   │   └── commands/       # Command implementations
│   └── tests/
└── mirror-sdk/             # Library crate
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs          # Library entry point
    │   ├── models.rs       # Data structures
    │   ├── config.rs       # Configuration management
    │   ├── hash.rs         # Hash generation
    │   ├── url_parser.rs   # URL parsing
    │   └── error.rs        # Error types
    └── tests/
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p mirror-sdk
cargo test -p mctl

# Run with verbose output
cargo test -- --nocapture
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Run the test suite (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Changelog

### v0.1.0 (Initial Release)

- Basic mirror.toml configuration management
- CLI with init, add, list, remove, show, validate commands
- Support for SSH and HTTPS git URLs
- Unique hash ID generation
- JSON output support
- Comprehensive validation
- Rust SDK for programmatic access