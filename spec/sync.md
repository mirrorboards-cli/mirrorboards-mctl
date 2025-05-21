# `mctl sync` Command

## Overview

The `mctl sync` command reads the `mirror.toml` configuration file and clones all defined repositories. If a repository is already cloned, it skips that repository, but if it's not yet cloned, it will clone it to the specified path.

## Syntax

```
mctl sync
```

## Parameters

This command does not require any parameters. It automatically processes all repositories defined in the `mirror.toml` file.

## Examples

```bash
mctl sync
```

## Behavior

1. Reads the `mirror.toml` configuration file
2. For each repository entry:
   - Checks if the repository already exists at the specified path
   - If not cloned, clones the repository using the specified git URL
   - If already cloned, skips the repository
3. Respects branch specifications defined in the configuration
4. Creates parent directories as needed for repository paths

## Notes

- This command is typically run after adding new repositories with `mctl add`
- It only performs clone operations, not updates (it doesn't pull changes for existing repositories)
- The command is idempotent - running it multiple times will only clone repositories that haven't been cloned yet
- Network connectivity is required to access remote git repositories
- Authentication may be required depending on the repository access settings
- The command uses the credentials configured in the git environment