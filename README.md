```bash
cargo install --git ssh://git@github.com/mirrorboards-cli/mirrorboards-mctl.git --config net.git-fetch-with-cli=true
```

# mctl - Mirror Configuration Management Tool (Next)

A new version of the CLI tool for managing multiple git repositories defined in a `mirror.toml` file. It uses the external `git` CLI instead of libgit2, which solves SSH agent issues across different systems.

## Features

- **Workspaces** - group repositories into logical spaces
- **Versioning** - branch/rev/tag for each repo
- **Includes** - compose configuration from multiple files
- **Remote Config** - synchronize mirror.toml with a remote repo
- **Snapshots** - create snapshots with exact commit hashes

## Installation

```bash
cargo install --git ssh://git@github.com/mirrorboards-cli/mirrorboards-mctl.git --config net.git-fetch-with-cli=true
```

## Quick start

```bash
# Initialization
mctl init

# Adding repositories
mctl add git@github.com:org/repo.git --workspace api
mctl add git@github.com:org/lib.git --workspace api --workspace core

# Synchronization
mctl sync                    # all
mctl sync api                # only the api workspace
mctl sync --create-missing-branches  # if the configured branch doesn't exist remotely, create it from the default one

# Status
mctl status
mctl status api --detailed

# Saving changes
mctl save --message "Update"
mctl save api --message "Update API"

# Snapshot
mctl snapshot                # → mirror.snapshot.toml
mctl snapshot api            # only the api workspace
```

## Configuration format

```toml
# mirror.toml

# Optional: include other files
[includes]
paths = [
    "teams/frontend.toml",
    "teams/backend.toml",
]

# Optional: synchronize with remote
[remote]
git = "git@github.com:org/mirror-config.git"
branch = "main"

# Repositories
[[repositories]]
git = "git@github.com:org/api.git"
path = "services/api"
branch = "main"              # or: rev = "abc123..." or: tag = "v1.0.0"
workspaces = ["api", "core"]

[[repositories]]
git = "git@github.com:external/lib.git"
path = "external/lib"
tag = "v2.0.0"
skip-push = true             # read-only
workspaces = ["external"]
```

## Commands

| Command | Description |
|---------|-------------|
| `mctl init` | Initialize a new configuration |
| `mctl add <url>` | Add a repository |
| `mctl list [workspace]` | List repositories |
| `mctl remove <path>` | Remove a repository |
| `mctl show <path>` | Repository details |
| `mctl validate` | Validate the configuration |
| `mctl sync [workspace]` | Synchronization (clone/pull); `--create-missing-branches` creates the branch from the default one when the configured branch doesn't exist remotely |
| `mctl status [workspace]` | Repository status |
| `mctl diff [workspace]` | Diff of changes |
| `mctl save [workspace]` | Commit and push changes |
| `mctl snapshot [workspace]` | Create a snapshot |
| `mctl from-org <org>` | Generate mirror.toml from a GitHub organization (alias: `get-repos`) |
| `mctl config init <url>` | Initialize remote config |
| `mctl config pull` | Pull config from remote |
| `mctl config push` | Push config to remote |
| `mctl config diff` | Diff against remote config |

## Global options

- `--config <file>` - use a different configuration file (default: `mirror.toml`)
- `--verbose` / `-v` - detailed output
- `--no-color` - disable colors

## Workspaces

Workspaces let you group repositories and run operations only on a selected group:

```bash
# Add a repo to multiple workspaces
mctl add git@github.com:org/shared.git --workspace api --workspace web

# Workspace operations
mctl sync api           # sync only api
mctl status web         # status only web
mctl save core          # save only core
```

## Includes

Compose configuration from multiple files:

```toml
# mirror.toml
[includes]
paths = [
    "teams/frontend.toml",
    "teams/backend.toml",
]
```

## Remote Config

Synchronize configuration between machines:

```bash
# Set the remote
mctl config init git@github.com:org/mirror-config.git

# Push local configuration
mctl config push -m "Update config"

# Pull on another machine
mctl config pull
```

## Snapshot

Create a snapshot with exact commit hashes:

```bash
mctl snapshot                          # → mirror.snapshot.toml
mctl snapshot --output prod.toml       # → prod.toml
mctl snapshot api                      # only the api workspace

# Restore from a snapshot
mctl --config mirror.snapshot.toml sync
```

## Generating from a GitHub organization

Build a `mirror.toml` from all repositories of an organization (or a user). The command uses the GitHub CLI (`gh`), so it relies on its authentication and pagination — it must be installed and logged in (`gh auth login`).

```bash
# Print mirror.toml to stdout (pipeable)
mctl from-org holonym-foundation > mirror.toml

# Alias
mctl get-repos holonym-foundation

# Write directly to a file
mctl from-org holonym-foundation --output mirror.toml

# Assign all repos to a workspace, use HTTPS, pin the default branch
mctl from-org holonym-foundation --workspace holonym --https --pin-branch
```

By default, archived repositories and forks are skipped (`--include-archived`, `--include-forks` to include them). SSH URLs are used by default (`--https` for HTTPS). Diagnostics go to stderr, so `> mirror.toml` produces a clean configuration file.

## License

MIT OR Apache-2.0
