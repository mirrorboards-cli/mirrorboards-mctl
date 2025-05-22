# mctl(1) -- Mirror Control CLI tool

## SYNOPSIS

`mctl` [*GLOBAL OPTIONS*] *COMMAND* [*SUBCOMMAND*] [*ARGUMENTS*] [*OPTIONS*]

## DESCRIPTION

**mctl** (Mirror Control) is a command-line interface tool that leverages the mirror-sdk to manage repositories defined in mirror.toml files. It provides commands for initializing, configuring, and managing repositories and their tags.

The tool follows a Git-style command structure with main commands and subcommands.

## GLOBAL OPTIONS

* `-c`, `--config` *PATH*:
  Path to the mirror.toml file. If not specified, mctl looks for a mirror.toml file in the current directory.

* `-v`, `--verbose`:
  Enable verbose output with additional details.

* `-q`, `--quiet`:
  Enable quiet mode with minimal output. Useful for scripting.

* `--color` *WHEN*:
  Control when to use colored output. Valid values are "always", "auto" (default), and "never".

* `-h`, `--help`:
  Print help information for the command.

* `-V`, `--version`:
  Print version information.

## COMMANDS

### init

Initialize a new mirror.toml file.

**Usage**: `mctl init [OPTIONS]`

**Options**:

* `-p`, `--path` *PATH*:
  Specify the path where the mirror.toml file should be created.

* `-f`, `--force`:
  Overwrite existing mirror.toml file if it exists.

**Examples**:

```
mctl init
mctl init --path /path/to/project
mctl init --force
```

### repo

Manage repositories in the mirror.toml file.

#### repo add

Add a new repository to the mirror.toml file.

**Usage**: `mctl repo add <ORIGIN> <PATH> [OPTIONS]`

**Arguments**:

* `<ORIGIN>`:
  Git repository URL (required).

* `<PATH>`:
  Local path where the repository should be cloned (required).

**Options**:

* `-i`, `--id` *ID*:
  Specify a custom ID for the repository. If not provided, an ID will be auto-generated.

* `-b`, `--branch` *BRANCH*:
  Specify the branch to use. Defaults to "main".

* `-t`, `--tag` *TAG*...:
  Add tags to the repository. Can be specified multiple times.

* `-l`, `--lock`:
  Lock the repository.

**Examples**:

```
mctl repo add git@github.com:user/repo.git path/to/clone
mctl repo add --id custom-id --branch develop git@github.com:user/repo.git path/to/clone
mctl repo add --tag frontend --tag important git@github.com:user/repo.git path/to/clone
```

#### repo remove

Remove a repository from the mirror.toml file.

**Usage**: `mctl repo remove <ID> [OPTIONS]`

**Arguments**:

* `<ID>`:
  ID of the repository to remove (required).

**Options**:

* `--force`:
  Force removal without confirmation.

**Examples**:

```
mctl repo remove repo-id
mctl repo remove repo-id --force
```

#### repo update

Update an existing repository in the mirror.toml file.

**Usage**: `mctl repo update <ID> [OPTIONS]`

**Arguments**:

* `<ID>`:
  ID of the repository to update (required).

**Options**:

* `-o`, `--origin` *ORIGIN*:
  Update the Git repository URL.

* `-p`, `--path` *PATH*:
  Update the local path.

* `-b`, `--branch` *BRANCH*:
  Update the branch.

* `-l`, `--lock` *true/false*:
  Update the lock status.

**Examples**:

```
mctl repo update repo-id --branch main
mctl repo update repo-id --path new/path --lock true
```

#### repo list

List repositories in the mirror.toml file.

**Usage**: `mctl repo list [OPTIONS]`

**Options**:

* `-t`, `--tag` *TAG*:
  Filter repositories by tag.

* `--path` *PREFIX*:
  Filter repositories by path prefix.

* `-j`, `--json`:
  Output in JSON format.

**Examples**:

```
mctl repo list
mctl repo list --tag frontend
mctl repo list --json
mctl repo list --path projects/
```

#### repo show

Show details of a specific repository.

**Usage**: `mctl repo show <ID>`

**Arguments**:

* `<ID>`:
  ID of the repository to show (required).

**Examples**:

```
mctl repo show repo-id
```

### tag

Manage repository tags.

#### tag add

Add tags to a repository.

**Usage**: `mctl tag add <ID> <TAG>...`

**Arguments**:

* `<ID>`:
  ID of the repository to add tags to (required).

* `<TAG>...`:
  Tags to add (required, can specify multiple).

**Examples**:

```
mctl tag add repo-id frontend
mctl tag add repo-id frontend important
```

#### tag remove

Remove tags from a repository.

**Usage**: `mctl tag remove <ID> <TAG>...`

**Arguments**:

* `<ID>`:
  ID of the repository to remove tags from (required).

* `<TAG>...`:
  Tags to remove (required, can specify multiple).

**Examples**:

```
mctl tag remove repo-id frontend
mctl tag remove repo-id frontend important
```

#### tag list

List all tags used in the mirror.toml file.

**Usage**: `mctl tag list [OPTIONS]`

**Options**:

* `-j`, `--json`:
  Output in JSON format.

**Examples**:

```
mctl tag list
mctl tag list --json
```

### config

Manage configuration settings.

#### config set

Set a configuration option.

**Usage**: `mctl config set <NAME> <VALUE>`

**Arguments**:

* `<NAME>`:
  Name of the configuration option (required).

* `<VALUE>`:
  Value to set (required).

**Examples**:

```
mctl config set default_branch main
mctl config set default_tag production
```

#### config get

Get a configuration option value.

**Usage**: `mctl config get <NAME>`

**Arguments**:

* `<NAME>`:
  Name of the configuration option (required).

**Examples**:

```
mctl config get default_branch
```

#### config list

List all configuration options.

**Usage**: `mctl config list [OPTIONS]`

**Options**:

* `-j`, `--json`:
  Output in JSON format.

**Examples**:

```
mctl config list
mctl config list --json
```

## OUTPUT FORMATS

mctl supports different output formats:

* **Human-readable** (default):
  Formatted text with color coding (when enabled).

* **JSON** (`--json`):
  Structured JSON output for programmatic consumption.

* **Quiet** (`--quiet`):
  Minimal output, suitable for scripting.

* **Verbose** (`--verbose`):
  Detailed output with additional information.

## COLOR CODING

When color is enabled, mctl uses the following color scheme:

* **Green**: Success messages, added items
* **Red**: Error messages, removed items
* **Yellow**: Warnings, important notes
* **Blue**: Informational messages
* **Cyan**: IDs and paths
* **Magenta**: Tags

## FILES

* **mirror.toml**:
  The configuration file that stores repository information. By default, mctl looks for this file in the current directory, but a different path can be specified with the `--config` option.

* **~/.config/mctl/config.toml**:
  User configuration file for default options (if implemented).

## ENVIRONMENT

* **RUST_LOG**:
  Controls the logging level. Set to "debug" for verbose logging.

## EXIT STATUS

* **0**: Success
* **1**: General error
* **2**: Usage error

## EXAMPLES

Initialize a new mirror.toml file:

```
mctl init --path my-project/mirror.toml
```

Add a repository:

```
mctl repo add https://github.com/example/repo.git src/example --tag frontend --branch main
```

List repositories with a specific tag:

```
mctl repo list --tag frontend
```

Add a tag to a repository:

```
mctl tag add c3fcf695 backend
```

Set a default branch configuration:

```
mctl config set default_branch main
```

## BUGS

Report bugs to: https://github.com/example/mirrorboards/issues

## AUTHOR

MirrorBoards Team

## COPYRIGHT

Copyright © 2025 MirrorBoards Team. License MIT.

## SEE ALSO

* mirror-sdk(3)
* git(1)