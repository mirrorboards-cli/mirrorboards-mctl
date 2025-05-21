# `mctl save` Command

## Overview

The `mctl save` command iterates over all repositories defined in the `mirror.toml` configuration file, performs `git add` on all changes, and then pushes those changes to their respective remote repositories. This command provides a convenient way to commit and push changes across multiple repositories at once.

## Syntax

```
mctl save [--message <COMMIT_MESSAGE>]
```

## Parameters

| Parameter | Description | Required |
|-----------|-------------|----------|
| `--message <COMMIT_MESSAGE>` | Custom commit message to use | No |

## Examples

```bash
mctl save
```

With a custom commit message:

```bash
mctl save --message "Update configuration files across repositories"
```

## Behavior

1. Reads the `mirror.toml` configuration file
2. For each repository entry:
   - Checks if the repository exists at the specified path
   - If the repository exists and has changes:
     - Performs `git add .` to stage all changes
     - Creates a commit with the specified message (or default message)
     - Pushes the changes to the remote repository
   - If no changes are detected, skips the repository

## Default Commit Message

If no custom message is provided, the default commit message format is:

```
${repository.org}${repository.name} - ${timestamp}
```

For example:
```
mirrorboards/mirrorboards-vscode - 2025-05-21T10:15:30Z
```

## Notes

- This command requires that repositories have already been cloned (use `mctl sync` first if needed)
- It will only commit and push changes in repositories that have modifications
- The command requires proper git credentials to be configured for pushing to remote repositories
- It's recommended to run `mctl status` before using this command to review changes
- This command performs operations that modify repositories and push to remotes, so use it with caution
- Network connectivity is required to push changes to remote repositories