# MCTL Configuration Format

This document defines the comprehensive configuration format for the MCTL tool, with a focus on flexibility, security, and usability.

## Configuration File Location

MCTL searches for configuration files in the following locations (in order of precedence):

1. Explicitly specified path via `--config` flag
2. `./mirror.toml` in the current working directory
3. `~/.config/mctl/mirror.toml` in the user's home configuration directory
4. `/etc/mctl/mirror.toml` for system-wide configuration

## Configuration Format

The configuration uses TOML format and supports the following structure:

```toml
# MCTL Configuration File (mirror.toml)

# Global settings applied to all repositories
[global]
# Parallel operations settings
parallel = true
max_threads = 8

# SSH authentication settings
[auth.ssh]
key_path = "~/.ssh/id_rsa"  # Default SSH key path, supports expansion
passphrase_command = ""     # Optional command to retrieve passphrase
known_hosts_path = "~/.ssh/known_hosts"

# Logging configuration
[logging]
level = "info"              # debug, info, warn, error
file = "~/.mctl/mctl.log"   # Optional log file path
format = "text"             # text or json

# Command-specific settings
[commands]
# Sync command settings
[commands.sync]
recursive = true            # Clone submodules
depth = 1                   # Git clone depth (1 for shallow)
timeout_seconds = 300       # Operation timeout

# Status command settings
[commands.status]
include_untracked = false   # Include untracked files in status
timeout_seconds = 60        # Operation timeout

# Save command settings
[commands.save]
push = true                 # Push after commit
sign_commits = false        # GPG sign commits

# Repository definitions
[[repositories]]
path = "repo1"              # Local path
origin = "git@github.com:user/repo1.git"
branch = "main"             # Specific branch to track
git = true                  # Is a git repository
enabled = true              # Can be disabled temporarily
tags = ["core", "frontend"] # Custom tags for grouping

# Repository-specific settings override global ones
[repositories.auth.ssh]
key_path = "~/.ssh/custom_key"

[repositories.commands.sync]
recursive = false

[[repositories]]
path = "repo2"
origin = "git@github.com:user/repo2.git"
```

## Configuration Sections

### Global Settings

```toml
[global]
parallel = true       # Enable/disable parallel processing
max_threads = 8       # Maximum number of parallel operations
```

- `parallel`: Boolean flag to enable/disable parallel repository operations
- `max_threads`: Integer specifying maximum number of concurrent operations

### Authentication Settings

```toml
[auth.ssh]
key_path = "~/.ssh/id_rsa"      # Path to SSH private key
passphrase_command = ""         # Command to retrieve passphrase
known_hosts_path = "~/.ssh/known_hosts"
```

- `key_path`: Path to SSH private key, supports `~` expansion for home directory
- `passphrase_command`: Optional shell command that outputs the passphrase
- `known_hosts_path`: Path to SSH known hosts file

### Logging Configuration

```toml
[logging]
level = "info"               # Log level: debug, info, warn, error
file = "~/.mctl/mctl.log"    # Log file path
format = "text"              # Log format: text or json
```

- `level`: String specifying log level
- `file`: Optional path for log file output
- `format`: String specifying log format

### Command-Specific Settings

```toml
[commands.sync]
recursive = true            # Clone submodules recursively
depth = 1                   # Git clone depth
timeout_seconds = 300       # Operation timeout

[commands.status]
include_untracked = false   # Include untracked files
timeout_seconds = 60        # Operation timeout

[commands.save]
push = true                 # Push after commit
sign_commits = false        # GPG sign commits
```

Each command can have specific settings that control its behavior.

### Repository Definitions

```toml
[[repositories]]
path = "repo1"                    # Local path
origin = "git@github.com:user/repo1.git"  # Git remote URL
branch = "main"                   # Branch to use
git = true                        # Is a git repository
enabled = true                    # Repository is active
tags = ["core", "frontend"]       # Custom tags
```

- `path`: String with local path to repository
- `origin`: String with Git remote URL
- `branch`: Optional string with branch name
- `git`: Boolean indicating if this is a Git repository
- `enabled`: Boolean to temporarily enable/disable repository
- `tags`: Array of strings for grouping repositories

### Repository-Specific Overrides

```toml
[repositories.auth.ssh]
key_path = "~/.ssh/custom_key"

[repositories.commands.sync]
recursive = false
```

Each repository can override global settings by specifying the same settings under the repository entry.

## Environment Variable Substitution

Configuration values support environment variable substitution using the syntax `${ENV_VAR}`:

```toml
[auth.ssh]
key_path = "${SSH_KEY_PATH:-~/.ssh/id_rsa}"
```

The syntax `${ENV_VAR:-default}` provides a default value if the environment variable is not set.

## Special Path Handling

Paths in the configuration support:

1. Home directory expansion (`~` expands to the user's home directory)
2. Environment variable substitution
3. Relative paths (relative to the configuration file location)

## Configuration Validation

MCTL validates the configuration file before using it, checking for:

1. Required fields
2. Type correctness
3. Path existence where applicable
4. Logical consistency

## Backward Compatibility

For backward compatibility with older versions, MCTL supports the minimal configuration format:

```toml
[[repositories]]
path = "repo1"
origin = "git@github.com:user/repo1.git"
git = true
```

When using the minimal format, default values are applied for all other settings.