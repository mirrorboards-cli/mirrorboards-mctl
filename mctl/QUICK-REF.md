# mctl Quick Reference Guide

This document provides a concise reference for all commands and options available in the Mirror Control (mctl) CLI tool.

## Command Structure

```
mctl [GLOBAL OPTIONS] COMMAND [SUBCOMMAND] [ARGUMENTS] [OPTIONS]
```

## Global Options

| Option | Description |
|--------|-------------|
| `-c, --config <PATH>` | Path to the mirror.toml file |
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Enable quiet mode (minimal output) |
| `--color <WHEN>` | Control when to use colored output (always, auto, never) |
| `-h, --help` | Print help information |
| `-V, --version` | Print version information |

## Commands and Subcommands

### init

Initialize a new mirror.toml file.

```
mctl init [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-p, --path <PATH>` | Specify the path where the mirror.toml file should be created |
| `-f, --force` | Overwrite existing mirror.toml file if it exists |

### repo

Manage repositories in the mirror.toml file.

#### repo add

Add a new repository.

```
mctl repo add <ORIGIN> <PATH> [OPTIONS]
```

| Argument/Option | Description |
|-----------------|-------------|
| `<ORIGIN>` | Git repository URL (required) |
| `<PATH>` | Local path where the repository should be cloned (required) |
| `-i, --id <ID>` | Specify a custom ID for the repository |
| `-b, --branch <BRANCH>` | Specify the branch to use (defaults to "main") |
| `-t, --tag <TAG>...` | Add tags to the repository (can be specified multiple times) |
| `-l, --lock` | Lock the repository |

#### repo remove

Remove a repository.

```
mctl repo remove <ID> [OPTIONS]
```

| Argument/Option | Description |
|-----------------|-------------|
| `<ID>` | ID of the repository to remove (required) |
| `--force` | Force removal without confirmation |

#### repo update

Update an existing repository.

```
mctl repo update <ID> [OPTIONS]
```

| Argument/Option | Description |
|-----------------|-------------|
| `<ID>` | ID of the repository to update (required) |
| `-o, --origin <ORIGIN>` | Update the Git repository URL |
| `-p, --path <PATH>` | Update the local path |
| `-b, --branch <BRANCH>` | Update the branch |
| `-l, --lock <true/false>` | Update the lock status |

#### repo list

List repositories.

```
mctl repo list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-t, --tag <TAG>` | Filter repositories by tag |
| `--path <PREFIX>` | Filter repositories by path prefix |
| `-j, --json` | Output in JSON format |

#### repo show

Show details of a specific repository.

```
mctl repo show <ID>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | ID of the repository to show (required) |

### tag

Manage repository tags.

#### tag add

Add tags to a repository.

```
mctl tag add <ID> <TAG>...
```

| Argument | Description |
|----------|-------------|
| `<ID>` | ID of the repository to add tags to (required) |
| `<TAG>...` | Tags to add (required, can specify multiple) |

#### tag remove

Remove tags from a repository.

```
mctl tag remove <ID> <TAG>...
```

| Argument | Description |
|----------|-------------|
| `<ID>` | ID of the repository to remove tags from (required) |
| `<TAG>...` | Tags to remove (required, can specify multiple) |

#### tag list

List all tags used in the mirror.toml file.

```
mctl tag list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-j, --json` | Output in JSON format |

### config

Manage configuration settings.

#### config set

Set a configuration option.

```
mctl config set <NAME> <VALUE>
```

| Argument | Description |
|----------|-------------|
| `<NAME>` | Name of the configuration option (required) |
| `<VALUE>` | Value to set (required) |

#### config get

Get a configuration option value.

```
mctl config get <NAME>
```

| Argument | Description |
|----------|-------------|
| `<NAME>` | Name of the configuration option (required) |

#### config list

List all configuration options.

```
mctl config list [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-j, --json` | Output in JSON format |

## Common Command Patterns

### Repository Management Workflow

```bash
# Initialize a new mirror.toml file
mctl init

# Add repositories
mctl repo add https://github.com/example/repo1.git ./src/repo1
mctl repo add https://github.com/example/repo2.git ./src/repo2 --tag frontend

# List all repositories
mctl repo list

# Update a repository
mctl repo update <repo-id> --branch develop

# Remove a repository
mctl repo remove <repo-id>
```

### Tag Management Workflow

```bash
# Add tags to repositories
mctl tag add <repo-id> frontend
mctl tag add <repo-id> important critical

# List all tags
mctl tag list

# List repositories with a specific tag
mctl repo list --tag frontend

# Remove tags
mctl tag remove <repo-id> critical
```

### Configuration Workflow

```bash
# Set configuration options
mctl config set default_branch main
mctl config set default_tag production

# List all configuration options
mctl config list

# Get a specific configuration option
mctl config get default_branch
```

## Tips and Tricks

1. **Use Tab Completion**: mctl supports tab completion in compatible shells for commands, subcommands, and options.

2. **JSON Output for Scripting**: Use the `--json` option with list commands for machine-readable output that can be parsed by other tools.

3. **Filtering Repositories**: Combine tag and path filters to narrow down repository lists:
   ```bash
   mctl repo list --tag frontend --path src/
   ```

4. **Batch Operations**: Use shell loops to perform operations on multiple repositories:
   ```bash
   for id in $(mctl repo list --json | jq -r '.[].id'); do
     mctl tag add $id batch-operation
   done
   ```

5. **Quiet Mode for Scripts**: Use the `--quiet` option to suppress all non-essential output when using mctl in scripts.

6. **Verbose Mode for Debugging**: Use the `--verbose` option to see detailed information about what mctl is doing.

7. **Custom Configuration File**: Use the `--config` option to specify a different mirror.toml file:
   ```bash
   mctl --config /path/to/custom-mirror.toml repo list
   ```

8. **Force Operations**: Use the `--force` option with destructive operations to skip confirmation prompts.

9. **Color Control**: Disable colors in non-interactive environments:
   ```bash
   mctl --color never repo list
   ```

10. **Help System**: Use the `--help` option with any command to see detailed help:
    ```bash
    mctl repo add --help