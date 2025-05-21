# MCTL - Mirror Control

MCTL (Mirror Control) is a Rust-based command-line interface (CLI) tool designed for efficient git repository synchronization and mirroring. It provides a robust solution for managing multiple git repositories across different environments and locations.

## Features

- **Configuration-driven**: Define all mirror relationships in a structured TOML file
- **Flexible mirroring**: Support for various mirroring scenarios (one-to-many, many-to-one)
- **Branch control**: Specify which branches to synchronize and how they should be mapped
- **Secure authentication**: Handle git credentials securely without hardcoding
- **Error resilience**: Robust error handling and recovery for git operations
- **Efficient synchronization**: Optimize network usage and operation time

## Installation

### From Source

```bash
git clone https://github.com/yourusername/mctl.git
cd mctl
cargo build --release
```

The binary will be available at `target/release/mctl`.

## Usage

### Configuration

MCTL uses a TOML configuration file (`mirror.toml`) to define repository mirror relationships. By default, it looks for this file in the current directory, but you can specify a different path using the `--config` option.

Example `mirror.toml`:

```toml
[[repositories]]
git-url = "https://github.com/example/repo.git"
path = "./local-repo"
branch = "main"

[[repositories]]
git-url = "https://github.com/another/repo.git"
path = "./another-local-repo"
```

### Commands

#### Add a Repository

```bash
mctl add --git-url https://github.com/example/repo.git --path ./local-repo [--branch main]
```

Or using positional arguments:

```bash
mctl add https://github.com/example/repo.git ./local-repo
```

#### Sync Repositories

Clone or update all repositories defined in the configuration:

```bash
mctl sync [--no-pull] [--force] [--parallel <NUM>]
```

Options:
- `--no-pull`: Skip pulling updates for existing repositories
- `--force`: Force pull even if it might cause conflicts
- `--parallel <NUM>`: Clone or pull multiple repositories in parallel

#### Check Status

Check the status of all repositories:

```bash
mctl status
```

#### Update Repositories

Update existing repositories with the latest changes:

```bash
mctl update [--force] [--dry-run] [--repo <NAME>]
```

Options:
- `--force`: Force update even when there might be conflicts
- `--dry-run`: Show what would be updated without making changes
- `--repo <NAME>`: Update only the specified repository

#### Save Changes

Commit and push changes in all repositories:

```bash
mctl save -m "Commit message"
```

Or:

```bash
mctl save "Commit message"
```

## License

MIT