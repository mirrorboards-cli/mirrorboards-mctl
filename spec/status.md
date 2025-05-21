# `mctl status` Command

## Overview

The `mctl status` command iterates over all repositories defined in the `mirror.toml` configuration file and checks their status. It provides a consolidated report similar to running `git status` individually on each tracked repository.

## Syntax

```
mctl status
```

## Parameters

This command does not require any parameters. It automatically processes all repositories defined in the `mirror.toml` file.

## Examples

```bash
mctl status
```

## Behavior

1. Reads the `mirror.toml` configuration file
2. For each repository entry:
   - Checks if the repository exists at the specified path
   - If the repository exists, runs the equivalent of `git status` on it
   - Collects and displays status information including:
     - Modified files
     - Untracked files
     - Branch information
     - Commit status (ahead/behind remote)
3. Presents a consolidated view of all repository statuses

## Output

The command output includes:

- Repository identification (name and path)
- Current branch information
- Modified files (if any)
- Untracked files (if any)
- Commit status relative to remote (ahead/behind)
- Error messages for repositories that cannot be accessed

## Notes

- This command is read-only and does not modify any repositories
- It requires that repositories have already been cloned (use `mctl sync` first if needed)
- The command provides a convenient way to check the status of multiple repositories at once
- Status information is similar to what you would get from running `git status` on each repository
- The command helps identify which repositories have pending changes that may need to be committed