# `mctl add` Command

## Overview

The `mctl add` command adds a git repository to the `mirror.toml` configuration file with specific settings. This command is used to start tracking a repository for synchronization.

## Syntax

```
mctl add [--git-url <URL>] [--path <LOCAL_PATH>] [--branch <BRANCH_NAME>]
```

Or using the short notation:

```
mctl add <GIT_URL> <LOCAL_PATH> [--branch <BRANCH_NAME>]
```

## Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `--git-url <URL>` | The git URL of the repository to add | Yes |
| `--path <LOCAL_PATH>` | The local path where the repository will be cloned | Yes |
| `--branch <BRANCH_NAME>` | The specific branch to track (optional) | No |

## Examples

### Basic Usage

```bash
mctl add --git-url git@github.com:mirrorboards/mirrorboards-vscode.git --path .vscode --branch configs/lunacrafts
```

### Short Notation

```bash
mctl add git@github.com:mirrorboards/mirrorboards-vscode.git .vscode --branch configs/lunacrafts
```

## Behavior

1. Validates the provided git URL and local path
2. Adds a new repository entry to the `mirror.toml` file
3. Does not clone the repository (use `mctl sync` after adding to clone)
4. If the repository is already in the configuration, it will update the existing entry

## Notes

- The git URL should be in a valid format (SSH or HTTPS)
- The local path is relative to the directory containing the `mirror.toml` file
- Branch specification is optional - if not provided, the default branch will be used
- Credentials should not be hardcoded in the git URL - use environment variables or credential helpers