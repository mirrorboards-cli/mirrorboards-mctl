# `mctl update` Command

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Syntax](#syntax)
- [Options](#options)
- [Examples](#examples)
- [Behavior](#behavior)
- [Output](#output)
- [Conflict Resolution](#conflict-resolution)
- [Authentication](#authentication)
- [Troubleshooting](#troubleshooting)
- [Related Commands](#related-commands)
- [Best Practices](#best-practices)
- [Security Considerations](#security-considerations)

## Overview

The `mctl update` command updates all repositories defined in the `mirror.toml` configuration file with the latest changes from their remote sources. This command performs the equivalent of a `git fetch` and `git merge` (or `git pull`) for each repository, ensuring your local copies stay synchronized with their remote counterparts. This is essential for maintaining up-to-date mirrors of multiple repositories.

## Prerequisites

Before using this command:

1. Ensure you have a valid `mirror.toml` configuration file in your current directory or specified config path
2. Verify that repositories have already been cloned using `mctl sync`
3. Verify network connectivity to access remote git repositories
4. Ensure you have proper authentication configured (SSH keys, tokens, or credentials) for the repositories
5. Check the status of repositories with `mctl status` to be aware of any local changes

## Syntax

```bash
mctl update [options]
```

## Options

| Option | Description |
|--------|-------------|
| `--config <path>` | Specify a custom path to the configuration file (default: `mirror.toml` in current directory) |
| `--verbose` | Enable verbose output with detailed progress information |
| `--force` | Force update even when there might be conflicts (use with caution) |
| `--dry-run` | Show what would be updated without actually making changes |
| `--repo <name>` | Update only the specified repository instead of all repositories |

## Examples

### Basic Update

Update all repositories defined in the default configuration:

```bash
mctl update
```

Example output:
```
[INFO] Reading configuration from mirror.toml
[INFO] Found 3 repositories in configuration
[INFO] Updating repository: example-repo
[INFO] Fetching latest changes for example-repo
[INFO] Merging changes (Fast-forward)
[INFO] Updated: 5 files changed, 120 insertions(+), 34 deletions(-)
[INFO] Updating repository: documentation-repo
[INFO] Fetching latest changes for documentation-repo
[INFO] Already up to date
[INFO] Updating repository: config-repo
[INFO] Fetching latest changes for config-repo
[INFO] Merging changes (Fast-forward)
[INFO] Updated: 2 files changed, 15 insertions(+), 7 deletions(-)
[INFO] Update complete: 2 repositories updated, 1 already up to date
```

### Using Custom Configuration File

```bash
mctl update --config custom-mirror.toml
```

### Updating a Specific Repository

```bash
mctl update --repo example-repo
```

### Performing a Dry Run

```bash
mctl update --dry-run
```

Example output:
```
[INFO] Reading configuration from mirror.toml
[INFO] Found 3 repositories in configuration
[INFO] DRY RUN: Would update repository: example-repo
[INFO] DRY RUN: Would fetch from origin
[INFO] DRY RUN: Would merge origin/main into main
[INFO] DRY RUN: Would update repository: documentation-repo
[INFO] DRY RUN: Would fetch from origin
[INFO] DRY RUN: Would merge origin/develop into develop
[INFO] DRY RUN: Would update repository: config-repo
[INFO] DRY RUN: Would fetch from origin
[INFO] DRY RUN: Would merge origin/main into main
[INFO] DRY RUN complete: Would update 3 repositories
```

## Behavior

1. Reads the configuration file (by default `mirror.toml` in the current directory)
2. Validates the configuration structure and repository entries
3. For each repository entry (or the specified repository if `--repo` is used):
   - Checks if the repository exists at the specified path
   - If the repository exists:
     - Changes to the repository directory
     - Performs a `git fetch` to retrieve the latest changes from the remote
     - For each configured branch:
       - Checks out the branch
       - Performs a `git merge` to update the local branch with remote changes
       - Reports any conflicts encountered
   - If the repository doesn't exist, reports an error and suggests using `mctl sync`
4. Provides a summary of the update operations performed

## Output

The command provides structured output indicating:

1. Configuration file being used
2. Number of repositories processed
3. Action taken for each repository
4. Status of each update operation (fast-forward, merge, conflict, etc.)
5. Summary of changes (files changed, insertions, deletions)
6. Summary of overall operation results
7. Any errors encountered during the process

With `--verbose` flag, additional information is displayed:
- Git commands being executed
- Branch-specific update details
- Detailed fetch and merge statistics

## Conflict Resolution

When conflicts are encountered during the update process:

1. The command will display an error message indicating which repository and files have conflicts
2. The repository will be left in a conflicted state for manual resolution
3. You'll need to:
   - Navigate to the repository directory
   - Resolve the conflicts manually using standard git procedures
   - Complete the merge with `git commit`
   - Continue with your workflow

If you want to avoid conflicts temporarily, you can use `--dry-run` to preview updates before applying them.

## Authentication

The `update` command relies on your git authentication methods:

1. **SSH Keys**: For SSH URLs (git@github.com:user/repo.git), your SSH keys should be properly configured
2. **HTTPS with Credentials**: For HTTPS URLs, credentials may be:
   - Stored in your git credential helper
   - Provided via environment variables (e.g., `GIT_USERNAME` and `GIT_PASSWORD`)
   - Prompted during execution if not found elsewhere

```bash
# Example of using environment variables for authentication
export GIT_USERNAME=your-username
export GIT_PASSWORD=your-token
mctl update
```

## Troubleshooting

| Issue | Possible Cause | Solution |
|-------|---------------|----------|
| `Repository not found` | Repository not cloned or incorrect path | Run `mctl sync` to clone missing repositories |
| `Authentication failed` | Missing or invalid credentials | Check your SSH keys or git credentials |
| `Merge conflict` | Local changes conflict with remote changes | Resolve conflicts manually in the repository |
| `Cannot update: You have unstaged changes` | Uncommitted changes in repository | Commit, stash, or discard local changes |
| `Cannot update: Non-fast-forward updates were rejected` | Divergent branch histories | Fetch and merge/rebase manually or use `--force` |
| `Network error` | Connectivity issues or repository URL problem | Check your network connection and repository URLs |

### Diagnostic Commands

If you encounter issues, these commands can help:

```bash
# Check repository status
cd <repository-path> && git status

# View remote configuration
cd <repository-path> && git remote -v

# Check branch tracking information
cd <repository-path> && git branch -vv

# Test authentication to a repository
git ls-remote <repository-url>
```

## Related Commands

- [`mctl sync`](sync.md): Clone repositories defined in the configuration
- [`mctl status`](status.md): Check status of all mirrored repositories 
- [`mctl add`](add.md): Add a new repository to the mirror configuration
- [`mctl save`](save.md): Save changes from working repositories to mirrors

## Best Practices

1. **Check Status First**: Run `mctl status` before updating to be aware of any local changes that might conflict
2. **Regular Updates**: Schedule regular updates to keep repositories in sync and minimize large divergences
3. **Commit Local Changes**: Always commit or stash local changes before updating to avoid conflicts
4. **Use Dry Run**: When working in critical environments, use `--dry-run` first to preview changes
5. **Selective Updates**: Use the `--repo` option for targeted updates when you don't need to update everything
6. **Rebase Workflow**: For cleaner history in active development, consider manually rebasing instead of using automatic merges
7. **Post-Update Verification**: Run `mctl status` after updating to verify everything is in the expected state

## Security Considerations

1. **Credential Management**: Never hardcode credentials in your configuration files
2. **Token Permissions**: When using access tokens, limit permissions to the minimum required
3. **SSH Keys**: Use SSH keys with passphrases for enhanced security
4. **Update Triggers**: Be cautious about automatic updates in production environments
5. **Code Review**: Review changes fetched from remote repositories before merging in sensitive environments
6. **Hook Scripts**: Be aware of any git hooks that might execute during the update process
7. **Network Security**: Ensure connections to remote repositories are secure, especially when using public networks

```bash
# Recommended: Using SSH keys with limited scope
git@github.com:username/repo.git

# Not recommended: Hardcoded credentials in URLs
https://username:password@github.com/username/repo.git