# `mctl status` Command

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Syntax](#syntax)
- [Options](#options)
- [Examples](#examples)
- [Behavior](#behavior)
- [Output Format](#output-format)
- [Troubleshooting](#troubleshooting)
- [Related Commands](#related-commands)
- [Best Practices](#best-practices)

## Overview

The `mctl status` command iterates over all repositories defined in the `mirror.toml` configuration file and checks their status. It provides a consolidated report similar to running `git status` individually on each tracked repository, displaying the exact path and status of each file. This allows you to quickly identify which repositories have modifications, untracked files, or require synchronization.

## Prerequisites

Before using this command:

1. Ensure you have a valid `mirror.toml` configuration file in your current directory or specified config path
2. Verify that repositories have already been cloned using `mctl sync`
3. You should have proper git configuration to display status information correctly

## Syntax

```bash
mctl status [options]
```

## Options

| Option | Description |
|--------|-------------|
| `--config <path>` | Specify a custom path to the configuration file (default: `mirror.toml` in current directory) |
| `--verbose` | Enable verbose output with detailed status information |

## Examples

### Basic Status Check

Check the status of all repositories defined in the default configuration:

```bash
mctl status
```

Example output:
```
[INFO] Reading configuration from mirror.toml
[INFO] Found 3 repositories in configuration

Repository: example-repo (./repos/example-repo)
Branch: main (up to date with origin/main)
Modified files:
  M ./repos/example-repo/src/main.rs
  M ./repos/example-repo/Cargo.toml
Untracked files:
  ?? ./repos/example-repo/notes.txt
  ?? ./repos/example-repo/src/test.rs

Repository: documentation-repo (./repos/docs)
Branch: develop (ahead of origin/develop by 2 commits)
Modified files:
  M ./repos/docs/README.md
  M ./repos/docs/installation.md
Untracked files:
  (none)

Repository: config-repo (./repos/configs)
Branch: main (up to date with origin/main)
Status: clean
```

### Using Custom Configuration File

```bash
mctl status --config custom-mirror.toml
```

### Using Verbose Output for Detailed Information

```bash
mctl status --verbose
```

## Behavior

1. Reads the configuration file (by default `mirror.toml` in the current directory)
2. Validates the configuration structure and repository entries
3. For each repository entry:
   - Checks if the repository exists at the specified path
   - If the repository exists, runs the equivalent of `git status` on it
   - Collects and displays status information including:
     - Full path to the repository
     - Current branch name and status relative to remote
     - Full paths to modified files
     - Full paths to untracked files
     - Commit status (ahead/behind remote)
4. Presents a consolidated view of all repository statuses with exact file paths

## Output Format

The command output follows a structured format:

### Repository Header
```
Repository: <name> (<full-path>)
Branch: <branch-name> (<status-relative-to-remote>)
```

### Modified Files Section
```
Modified files:
  <status-code> <full-path-to-file>
  <status-code> <full-path-to-file>
  ...
```

Status codes follow standard git conventions:
- `M` - Modified file
- `A` - Added file
- `D` - Deleted file
- `R` - Renamed file
- `C` - Copied file
- `U` - Updated but unmerged file

### Untracked Files Section
```
Untracked files:
  ?? <full-path-to-file>
  ?? <full-path-to-file>
  ...
```

### Clean Status
For repositories with no changes:
```
Status: clean
```

## Troubleshooting

| Issue | Possible Cause | Solution |
|-------|---------------|----------|
| `Repository not found` | Repository not cloned or incorrect path | Run `mctl sync` to clone missing repositories |
| `Permission denied` | Insufficient access rights | Check file permissions for the repository directory |
| `Not a git repository` | Directory exists but isn't a git repository | Remove invalid directory and run `mctl sync` |
| `Failed to read status` | Git issues or corrupted repository | Run `git fsck` in the affected repository to check for corruption |

### Diagnostic Commands

If you encounter issues, these commands can help:

```bash
# Verify git configuration
git config --list

# Check repository integrity
git fsck --full

# Get detailed repository information
git remote -v
```

## Related Commands

- [`mctl sync`](sync.md): Clone repositories defined in the configuration
- [`mctl add`](add.md): Add a new repository to the mirror configuration
- [`mctl update`](update.md): Update existing repositories with latest changes
- [`mctl save`](save.md): Save changes from working repositories to mirrors

## Best Practices

1. **Regular Status Checks**: Run `mctl status` before and after making changes to ensure you're aware of all modifications
2. **Before Synchronization**: Always check status before running `mctl update` or `mctl save` to avoid unexpected conflicts
3. **Resolving Issues**: Address any unexpected changes revealed by status before proceeding with other operations
4. **Review All Repositories**: Pay attention to the status of all repositories, not just the ones you're actively working on
5. **Check Branch Status**: Note whether branches are ahead, behind, or diverged from their remote counterparts