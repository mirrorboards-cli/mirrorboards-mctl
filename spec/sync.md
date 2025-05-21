# `mctl sync` Command

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Syntax](#syntax)
- [Options](#options)
- [Examples](#examples)
- [Behavior](#behavior)
- [Output](#output)
- [Authentication](#authentication)
- [Troubleshooting](#troubleshooting)
- [Related Commands](#related-commands)
- [Best Practices](#best-practices)
- [Security Considerations](#security-considerations)
- [Performance Considerations](#performance-considerations)
- [Configuration Example](#configuration-example)

## Overview

The `mctl sync` command reads the `mirror.toml` configuration file and initializes the repository mirroring setup by cloning all defined repositories that do not yet exist locally. For existing repositories, it attempts to update them by pulling the latest changes. If a repository has conflicts, uncommitted changes, or other issues, it will be skipped with an informative message. This command establishes and maintains synchronized local copies of repositories according to your configuration.

## Prerequisites

Before using this command:

1. Ensure you have a valid `mirror.toml` configuration file in your current directory or specified by `--config`/`--mirror`
2. If using `--dest`, ensure the destination directory exists or can be created with proper permissions
3. Verify network connectivity to access remote git repositories
4. Ensure you have proper authentication configured (SSH keys, tokens, or credentials) for the repositories
5. Sufficient disk space available for the repositories to be cloned
6. Consider committing or stashing any local changes in existing repositories to avoid conflicts

## Syntax

```bash
mctl sync [options]
```

## Options

| Option | Description |
|--------|-------------|
| `--config <path>` | Specify a custom path to the configuration file (default: `mirror.toml` in current directory) |
| `--mirror <path>` | Alias for `--config`, specify a custom path to the configuration file |
| `--dest <path>` | Specify a custom destination directory for cloned repositories (default: current directory) |
| `--verbose` | Enable verbose output with detailed progress information |
| `--no-pull` | Skip pulling updates for existing repositories (clone-only mode) |
| `--force` | Attempt to pull changes even if it might cause conflicts (use with caution) |
| `--parallel <num>` | Clone or pull multiple repositories in parallel (default: sequential processing) |

## Examples

### Basic Synchronization

Clone new repositories and update existing ones defined in the default `mirror.toml` file:

```bash
mctl sync
```

Example output:
```
[INFO] Reading configuration from mirror.toml
[INFO] Found 3 repositories in configuration
[INFO] Checking repository: example-repo
[INFO] Cloning example-repo to ./repos/example-repo
[INFO] Checking repository: another-repo
[INFO] Repository exists, pulling latest changes
[INFO] Updated another-repo: 2 files changed, 15 insertions(+), 5 deletions(-)
[INFO] Checking repository: third-repo
[INFO] Repository exists, has uncommitted changes - skipping pull
[INFO] Synchronization complete: 1 repository cloned, 1 updated, 1 skipped
```

### Using Custom Configuration File

```bash
mctl sync --config custom-mirror.toml
```

### Using Custom Destination Directory

```bash
mctl sync --dest ./backup-repos
```

### Using Both Custom Configuration and Destination

```bash
mctl sync --mirror ./examples/mirror.toml --dest ./tmp
```

### Using Verbose Output for Detailed Progress

```bash
mctl sync --verbose
```

### Clone-Only Mode (Skip Pull Operations)

```bash
mctl sync --no-pull
```

Example output:
```
[INFO] Reading configuration from mirror.toml
[INFO] Found 3 repositories in configuration
[INFO] Checking repository: example-repo
[INFO] Cloning example-repo to ./repos/example-repo
[INFO] Checking repository: another-repo
[INFO] Repository already exists at ./repos/another-repo, skipping (--no-pull flag)
[INFO] Checking repository: third-repo
[INFO] Repository already exists at ./repos/third-repo, skipping (--no-pull flag)
[INFO] Synchronization complete: 1 repository cloned, 0 updated, 2 skipped
```

### Parallel Processing for Faster Synchronization

```bash
mctl sync --parallel 4
```

## Behavior

1. Reads the configuration file (by default `mirror.toml` in the current directory, or specified by `--config`/`--mirror`)
2. Determines the base directory (current directory by default, or specified by `--dest`)
3. Validates the configuration structure and repository entries
4. For each repository entry:
   - Resolves the full local path based on configuration settings and the base directory
   - Checks if the repository already exists at the specified path
   - If not cloned:
     - Creates parent directories as needed for the repository path
     - Clones the repository using the specified git URL
     - Sets up the default branch as specified in the configuration
     - Configures remote URLs according to the repository mapping
   - If already cloned (and `--no-pull` is not specified):
     - Attempts to pull the latest changes from the remote repository
     - If the pull is successful, updates the local repository
     - If there are conflicts, uncommitted changes, or other issues:
       - Skips the repository without modifying it
       - Reports the specific issue for troubleshooting
5. Respects branch specifications defined in the configuration
6. Reports a summary of actions performed (cloned, updated, skipped)

## Output

The command provides structured output indicating:

1. Configuration file being used
2. Number of repositories processed
3. Action taken for each repository (cloned, updated, or skipped)
4. For updated repositories, a summary of changes (files changed, insertions, deletions)
5. For skipped repositories, the reason why they were skipped
6. Summary of operations performed
7. Any errors encountered during the process

With `--verbose` flag, additional information is displayed:
- Git commands being executed
- Remote URLs being configured
- Branch setup details
- Detailed progress of clone and pull operations
- Complete git output for each operation

## Authentication

The `sync` command relies on your git authentication methods:

1. **SSH Keys**: For SSH URLs (git@github.com:user/repo.git), your SSH keys should be properly configured
2. **HTTPS with Credentials**: For HTTPS URLs, credentials may be:
   - Stored in your git credential helper
   - Provided via environment variables (e.g., `GIT_USERNAME` and `GIT_PASSWORD`)
   - Prompted during execution if not found elsewhere

```bash
# Example of using environment variables for authentication
export GIT_USERNAME=your-username
export GIT_PASSWORD=your-token
mctl sync
```

## Environment Variables

The following environment variables affect the `sync` command behavior:

| Variable | Purpose | Example |
|----------|---------|---------|
| `GIT_USERNAME` | Username for HTTPS authentication | `export GIT_USERNAME=your-username` |
| `GIT_PASSWORD` | Password or token for HTTPS authentication | `export GIT_PASSWORD=your-token` |
| `GIT_SSL_NO_VERIFY` | Disable SSL verification (not recommended) | `export GIT_SSL_NO_VERIFY=1` |
| `MCTL_CONFIG_PATH` | Default configuration file path | `export MCTL_CONFIG_PATH=/path/to/config.toml` |
| `MCTL_LOG_LEVEL` | Control verbosity (debug, info, warn, error) | `export MCTL_LOG_LEVEL=debug` |

## Troubleshooting

| Issue | Possible Cause | Solution |
|-------|---------------|----------|
| `Authentication failed` | Missing or invalid credentials | Check your SSH keys or git credentials |
| `Repository not found` | Incorrect URL or no access | Verify the repository URL and your access permissions |
| `Destination path already exists and is not an empty directory` | Folder exists but is not a git repository | Remove or rename the existing folder |
| `Could not read from remote repository` | Network issue or repository URL problem | Check your network connection and repository URL |
| `Failed to create directory` | Insufficient permissions | Check write permissions on parent directories |
| `Cannot pull with uncommitted changes` | Local changes not committed | Commit or stash your changes before syncing |
| `Cannot pull: You have unmerged files` | Previous merge conflict not resolved | Resolve the conflicts or reset the repository |
| `Cannot pull: Local branch is ahead of remote` | Local commits not pushed | Push local changes or use `mctl save` first |
| `Auto-merging failed. Resolve conflicts and commit the result` | Conflicts between local and remote changes | Manually resolve conflicts or use `--no-pull` to skip updates |

### Diagnostic Commands

If you encounter issues, these commands can help:

```bash
# Verify git configuration
git config --list

# Test authentication to a repository
git ls-remote <repository-url>

# Check disk space
df -h

# Check repository status
cd <repository-path> && git status

# View repository remote configuration
cd <repository-path> && git remote -v
```

## Related Commands

- [`mctl add`](add.md): Add a new repository to the mirror configuration
- [`mctl status`](status.md): Check status of all mirrored repositories
- [`mctl update`](update.md): Update existing repositories with latest changes
- [`mctl save`](save.md): Save changes from working repositories to mirrors

## Best Practices

1. **Initial Setup**: Run `mctl sync` after creating or updating your `mirror.toml` file to initialize repositories
2. **Configuration Management**: Keep your `mirror.toml` file in version control
3. **Authentication**: Set up SSH keys for secure, password-less authentication
4. **Regular Verification**: Run `mctl status` after sync to verify repository state
5. **Clean Repositories**: Commit or stash local changes before running sync to avoid conflicts
6. **Separate Updates**: Use `--no-pull` when you only want to clone new repositories without updating existing ones
7. **Custom Destinations**: Use `--dest` to clone repositories to a different location than the current directory
8. **Backup Workflows**: Combine `--mirror` and `--dest` for efficient backup workflows
9. **Troubleshooting**: Use `--verbose` when encountering issues to see detailed operation output
10. **Parallel Processing**: For large numbers of repositories, use the `--parallel` option to speed up synchronization

## Security Considerations

1. **Credential Storage**: Never hardcode credentials in your configuration files
2. **Token Permissions**: When using access tokens, limit permissions to the minimum required
3. **SSH Keys**: Use SSH keys with passphrases for enhanced security
4. **Private Repositories**: Ensure secure access control for private repository contents
5. **Audit**: Regularly review your mirroring setup and access patterns
6. **Credential Rotation**: Regularly rotate access tokens and update SSH keys

```bash
# Recommended: Using SSH keys with limited scope
git@github.com:username/repo.git

# Not recommended: Hardcoded credentials in URLs
https://username:password@github.com/username/repo.git
```

## Performance Considerations

When synchronizing multiple repositories, especially large ones, consider the following:

1. **Parallel Processing**: Use the `--parallel` option to clone/update multiple repositories simultaneously
2. **Disk Space**: Ensure sufficient disk space for all repositories, including their full history
3. **Network Bandwidth**: Large repositories require significant bandwidth, particularly during initial cloning
4. **Shallow Clones**: For specific use cases where full history isn't needed, consider configuring shallow clones
5. **Selective Syncing**: Consider splitting large mirror.toml files into smaller ones to sync only what's needed

## Configuration Example

Here's an example `mirror.toml` file used by the `sync` command:

```toml
# mirror.toml - Configuration for mctl repository synchronization

# Global settings
base_path = "./repos"  # Base directory for all repositories

# Repository definitions
[[repositories]]
name = "example-repo"
origin = "git@github.com:example/repo.git"
path = "example-repo"  # Will be cloned to ./repos/example-repo
branch = "main"        # Use main branch

[[repositories]]
name = "documentation"
origin = "https://github.com/example/docs.git"
path = "docs"          # Will be cloned to ./repos/docs
branch = "develop"     # Use develop branch

[[repositories]]
name = "config-repo"
origin = "git@gitlab.com:example/configs.git"
path = "configs"       # Will be cloned to ./repos/configs
branch = "main"        # Use main branch
```

This configuration defines three repositories to be synchronized, each with its own remote URL, local path, and branch specification. When running `mctl sync`, the tool will ensure all these repositories are properly cloned and updated according to these specifications.