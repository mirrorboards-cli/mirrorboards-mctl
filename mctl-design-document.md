# mctl (Mirror Control) CLI Design Document

## Overview

`mctl` is a command-line interface (CLI) tool that leverages the mirror-sdk to manage repositories in mirror.toml files. This document outlines the design of the CLI tool, including its command structure, user experience, and implementation strategy.

## 1. Command Structure

### 1.1 Main Commands

The `mctl` CLI follows a Git-style command structure with the following main commands:

| Command | Description |
|---------|-------------|
| `init`  | Initialize a new mirror.toml file |
| `repo`  | Manage repositories (add, remove, update, list, show) |
| `tag`   | Manage repository tags |
| `config`| Manage configuration settings |
| `help`  | Display help information |
| `version` | Display version information |

### 1.2 Subcommands

#### 1.2.1 `init` Subcommands

| Subcommand | Description |
|------------|-------------|
| (none)     | Initialize a mirror.toml file in the current directory |

#### 1.2.2 `repo` Subcommands

| Subcommand | Description |
|------------|-------------|
| `add`      | Add a new repository to the mirror.toml file |
| `remove`   | Remove a repository from the mirror.toml file |
| `update`   | Update an existing repository's properties |
| `list`     | List all repositories in the mirror.toml file |
| `show`     | Show details of a specific repository |

#### 1.2.3 `tag` Subcommands

| Subcommand | Description |
|------------|-------------|
| `add`      | Add tags to a repository |
| `remove`   | Remove tags from a repository |
| `list`     | List all tags used in the mirror.toml file |

#### 1.2.4 `config` Subcommands

| Subcommand | Description |
|------------|-------------|
| `set`      | Set a configuration option |
| `get`      | Get a configuration option value |
| `list`     | List all configuration options |

### 1.3 Command Arguments and Options

#### 1.3.1 Global Options

These options can be used with any command:

| Option | Description |
|--------|-------------|
| `-h, --help` | Display help information for the command |
| `-v, --verbose` | Enable verbose output |
| `-q, --quiet` | Enable quiet mode (minimal output) |
| `-c, --config <path>` | Specify the path to the mirror.toml file |
| `--color <when>` | Control when to use colored output (always, auto, never) |

#### 1.3.2 `init` Options

| Option | Description |
|--------|-------------|
| `-p, --path <path>` | Specify the path where the mirror.toml file should be created |
| `-f, --force` | Overwrite existing mirror.toml file if it exists |

#### 1.3.3 `repo add` Arguments and Options

| Argument/Option | Description |
|-----------------|-------------|
| `<origin>` | Git repository URL (required) |
| `<path>` | Local path where the repository should be cloned (required) |
| `-i, --id <id>` | Specify a custom ID for the repository |
| `-b, --branch <branch>` | Specify the branch to use |
| `-t, --tag <tag>...` | Add tags to the repository (can be specified multiple times) |
| `-l, --lock` | Lock the repository |

#### 1.3.4 `repo remove` Arguments and Options

| Argument/Option | Description |
|-----------------|-------------|
| `<id>` | ID of the repository to remove (required) |
| `--force` | Force removal without confirmation |

#### 1.3.5 `repo update` Arguments and Options

| Argument/Option | Description |
|-----------------|-------------|
| `<id>` | ID of the repository to update (required) |
| `-o, --origin <origin>` | Update the Git repository URL |
| `-p, --path <path>` | Update the local path |
| `-b, --branch <branch>` | Update the branch |
| `-l, --lock <true/false>` | Update the lock status |

#### 1.3.6 `repo list` Options

| Option | Description |
|--------|-------------|
| `-t, --tag <tag>` | Filter repositories by tag |
| `-j, --json` | Output in JSON format |
| `--path <prefix>` | Filter repositories by path prefix |

#### 1.3.7 `repo show` Arguments

| Argument | Description |
|----------|-------------|
| `<id>` | ID of the repository to show (required) |

#### 1.3.8 `tag add` Arguments and Options

| Argument/Option | Description |
|-----------------|-------------|
| `<id>` | ID of the repository to add tags to (required) |
| `<tag>...` | Tags to add (required, can specify multiple) |

#### 1.3.9 `tag remove` Arguments and Options

| Argument/Option | Description |
|-----------------|-------------|
| `<id>` | ID of the repository to remove tags from (required) |
| `<tag>...` | Tags to remove (required, can specify multiple) |

### 1.4 Command Usage Examples

#### 1.4.1 Initialize a new mirror.toml file

```bash
# Initialize in the current directory
mctl init

# Initialize at a specific path
mctl init --path /path/to/project

# Force overwrite of existing file
mctl init --force
```

#### 1.4.2 Add a repository

```bash
# Add a repository with auto-generated ID
mctl repo add git@github.com:user/repo.git path/to/clone

# Add a repository with custom ID and branch
mctl repo add --id custom-id --branch develop git@github.com:user/repo.git path/to/clone

# Add a repository with tags
mctl repo add --tag frontend --tag important git@github.com:user/repo.git path/to/clone
```

#### 1.4.3 List repositories

```bash
# List all repositories
mctl repo list

# List repositories with a specific tag
mctl repo list --tag frontend

# List repositories in JSON format
mctl repo list --json

# List repositories in a specific directory
mctl repo list --path projects/
```

#### 1.4.4 Show repository details

```bash
# Show details of a specific repository
mctl repo show repo-id
```

#### 1.4.5 Update a repository

```bash
# Update the branch of a repository
mctl repo update repo-id --branch main

# Update multiple properties
mctl repo update repo-id --path new/path --lock true
```

#### 1.4.6 Remove a repository

```bash
# Remove a repository (with confirmation)
mctl repo remove repo-id

# Force remove without confirmation
mctl repo remove repo-id --force
```

#### 1.4.7 Manage tags

```bash
# Add tags to a repository
mctl tag add repo-id frontend important

# Remove tags from a repository
mctl tag remove repo-id frontend

# List all tags
mctl tag list
```

## 2. User Experience

### 2.1 User Interaction

The `mctl` CLI is designed to be intuitive and user-friendly, following these principles:

1. **Consistency**: Commands follow a consistent structure and naming convention.
2. **Discoverability**: Help text is comprehensive and easily accessible.
3. **Feedback**: The CLI provides clear feedback on actions taken.
4. **Safety**: Destructive operations require confirmation unless forced.
5. **Efficiency**: Common operations can be performed with minimal typing.

### 2.2 Output Formatting

#### 2.2.1 Color Coding

The CLI uses color coding to enhance readability:

| Color | Usage |
|-------|-------|
| Green | Success messages, added items |
| Red | Error messages, removed items |
| Yellow | Warnings, important notes |
| Blue | Informational messages |
| Cyan | IDs and paths |
| Magenta | Tags |

#### 2.2.2 Output Levels

The CLI supports three output levels:

1. **Quiet** (`-q, --quiet`): Minimal output, suitable for scripting.
2. **Normal** (default): Standard output with essential information.
3. **Verbose** (`-v, --verbose`): Detailed output with additional information.

#### 2.2.3 Output Formats

The CLI supports multiple output formats:

1. **Human-readable** (default): Formatted text with color coding.
2. **JSON** (`--json`): Structured JSON output for programmatic consumption.
3. **Table** (`--table`): Tabular output for certain commands.

### 2.3 Error Messages and Feedback

Error messages are designed to be helpful and actionable:

1. **Context**: Clearly state what operation was being performed.
2. **Problem**: Describe what went wrong.
3. **Solution**: Suggest how to fix the issue.
4. **Details**: Provide technical details when relevant.

Examples:

```
Error: Failed to add repository
  Problem: Repository with ID 'custom-id' already exists
  Solution: Use a different ID or update the existing repository
  Command: mctl repo update custom-id --origin git@github.com:user/repo.git
```

```
Error: Failed to load mirror.toml
  Problem: File not found at '/path/to/mirror.toml'
  Solution: Initialize a new file or specify the correct path
  Command: mctl init or mctl --config /correct/path/mirror.toml
```

### 2.4 Help System

The CLI includes a comprehensive help system:

1. **Command Help**: `mctl help <command>` or `mctl <command> --help`
2. **Examples**: Each help section includes practical examples.
3. **Related Commands**: Help text suggests related commands.
4. **Man Pages**: Generated man pages for Unix-like systems.

## 3. Implementation Strategy

### 3.1 CLI Framework

The recommended CLI framework for implementing `mctl` is [Clap](https://github.com/clap-rs/clap) (Command Line Argument Parser) for Rust. Clap is a mature, feature-rich framework that provides:

1. **Declarative API**: Define commands, arguments, and options using a declarative syntax.
2. **Automatic Help Generation**: Automatically generate help text based on command definitions.
3. **Argument Validation**: Validate arguments and provide helpful error messages.
4. **Subcommand Support**: First-class support for nested subcommands.
5. **Completion Scripts**: Generate shell completion scripts.

### 3.2 Integration with mirror-sdk

The CLI will integrate with the mirror-sdk as follows:

1. **Direct Dependency**: Include mirror-sdk as a dependency in Cargo.toml.
2. **Command Mapping**: Map CLI commands directly to SDK functions:
   - `mctl init` → `MirrorConfig::init()` or `MirrorConfig::init_at()`
   - `mctl repo add` → `MirrorConfig::add_repository()`
   - `mctl repo remove` → `MirrorConfig::remove_repository()`
   - `mctl repo list` → `MirrorConfig::get_repositories()`
   - `mctl repo list --tag <tag>` → `MirrorConfig::get_repositories_by_tag()`

3. **Error Handling**: Map SDK errors to user-friendly CLI error messages.
4. **Configuration Management**: Handle loading and saving of mirror.toml files.

### 3.3 Project Structure

The recommended project structure is:

```
mctl/
├── Cargo.toml
├── src/
│   ├── main.rs           # Entry point
│   ├── cli/              # CLI definition and parsing
│   │   ├── mod.rs
│   │   ├── init.rs       # Init command
│   │   ├── repo.rs       # Repo commands
│   │   ├── tag.rs        # Tag commands
│   │   └── config.rs     # Config commands
│   ├── commands/         # Command implementation
│   │   ├── mod.rs
│   │   ├── init.rs
│   │   ├── repo.rs
│   │   ├── tag.rs
│   │   └── config.rs
│   ├── output/           # Output formatting
│   │   ├── mod.rs
│   │   ├── color.rs
│   │   ├── table.rs
│   │   └── json.rs
│   └── utils/            # Utility functions
│       ├── mod.rs
│       ├── error.rs
│       └── config.rs
├── tests/                # Integration tests
└── completions/          # Shell completion scripts
```

### 3.4 Additional Features

Beyond the core SDK capabilities, the CLI should include these additional features:

1. **Shell Completions**: Generate completion scripts for Bash, Zsh, Fish, and PowerShell.
2. **Batch Operations**: Support for batch operations on multiple repositories.
3. **Interactive Mode**: An interactive mode for guided operations.
4. **Configuration File**: A user configuration file for default options.
5. **Aliases**: Support for command aliases.
6. **Plugins**: A plugin system for extending functionality.
7. **Dry Run**: A dry-run mode to preview changes without applying them.
8. **Backup**: Automatic backup of mirror.toml before modifications.
9. **Import/Export**: Import/export repositories from/to other formats.
10. **Validation**: Validate repository URLs and paths.

### 3.5 Performance Considerations

To ensure good performance, the CLI should:

1. **Lazy Loading**: Only load the mirror.toml file when needed.
2. **Caching**: Cache repository information for faster access.
3. **Parallel Operations**: Use parallelism for batch operations.
4. **Efficient Parsing**: Use efficient parsing techniques for large files.

## 4. Conclusion

The `mctl` CLI tool will provide a comprehensive, user-friendly interface to the mirror-sdk, enabling users to efficiently manage repositories in mirror.toml files. By following the design principles outlined in this document, the CLI will be intuitive, powerful, and extensible.

The implementation will leverage the Clap framework for robust command-line parsing and the mirror-sdk for core functionality, while adding value through improved user experience, additional features, and performance optimizations.