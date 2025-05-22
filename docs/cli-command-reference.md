# Mirror CLI Command Reference

This document provides a comprehensive reference for the Mirror CLI commands.

## Global Options

These options can be used with any command:

| Option | Description |
|--------|-------------|
| `--config <FILE>`, `-c <FILE>` | Path to the mirror.toml file. If not specified, the CLI will look for the file in the following order: <br>1. The `MIRROR_CONFIG` environment variable<br>2. The default path (mirror.toml in the current directory) |
| `--help`, `-h` | Print help information |
| `--version`, `-V` | Print version information |

## Commands

### `init`

Create a new empty mirror.toml file.

```bash
mirror-cli init [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Force creation even if file exists |

#### Examples

```bash
# Create a new mirror.toml file
mirror-cli init

# Force creation of a new mirror.toml file
mirror-cli init --force

# Create a new mirror.toml file at a specific path
mirror-cli --config custom-mirror.toml init
```

### `add`

Add a new repository to the configuration.

```bash
mirror-cli add [OPTIONS] --origin <ORIGIN> --path <PATH>
```

#### Options

| Option | Description |
|--------|-------------|
| `--origin <ORIGIN>`, `-o <ORIGIN>` | Git repository origin URL (required) |
| `--branch <BRANCH>`, `-b <BRANCH>` | Git branch to use (default: "main") |
| `--path <PATH>`, `-p <PATH>` | Local filesystem path where the repository should be cloned (required) |
| `--id <ID>`, `-i <ID>` | Optional unique identifier for the repository |
| `--branch-lock` | Whether the branch is locked (cannot be changed) |
| `--tags <TAGS>`, `-t <TAGS>` | Optional tags for categorizing repositories (comma-separated) |

#### Examples

```bash
# Add a repository with default branch (main)
mirror-cli add --origin "git@github.com:example/repo.git" --path "example/repo"

# Add a repository with a specific branch
mirror-cli add --origin "git@github.com:example/repo.git" --path "example/repo" --branch "develop"

# Add a repository with tags
mirror-cli add --origin "git@github.com:example/repo.git" --path "example/repo" --tags "example,test"

# Add a repository with a unique ID and locked branch
mirror-cli add --origin "git@github.com:example/repo.git" --path "example/repo" --id "example-repo" --branch-lock
```

### `remove`

Remove a repository from the configuration.

```bash
mirror-cli remove [OPTIONS] --path <PATH> | --id <ID>
```

#### Options

| Option | Description |
|--------|-------------|
| `--path <PATH>`, `-p <PATH>` | Repository path to remove |
| `--id <ID>`, `-i <ID>` | Repository ID to remove |

**Note:** You must specify either `--path` or `--id`, but not both.

#### Examples

```bash
# Remove a repository by path
mirror-cli remove --path "example/repo"

# Remove a repository by ID
mirror-cli remove --id "example-repo"
```

### `list`

List all repositories in the configuration.

```bash
mirror-cli list [OPTIONS]
```

#### Options

| Option | Description |
|--------|-------------|
| `--tag <TAG>`, `-t <TAG>` | Filter repositories by tag |

#### Examples

```bash
# List all repositories
mirror-cli list

# List repositories with a specific tag
mirror-cli list --tag "example"
```

#### Output Format

The `list` command outputs repositories in the following format:

```
Listing: All repositories:
1 example/repo
  Origin: git@github.com:example/repo.git
  Branch: main
  Tags: example, test
```

If a repository has an ID or branch lock, those will also be displayed:

```
1 example/repo
  ID: example-repo
  Origin: git@github.com:example/repo.git
  Branch: main
  Branch Lock: true
  Tags: example, test
```

### `update`

Update a repository's properties.

```bash
mirror-cli update [OPTIONS] --path <PATH>
```

#### Options

| Option | Description |
|--------|-------------|
| `--path <PATH>`, `-p <PATH>` | Repository path to update (required) |
| `--origin <ORIGIN>`, `-o <ORIGIN>` | New Git repository origin URL |
| `--branch <BRANCH>`, `-b <BRANCH>` | New Git branch to use |
| `--new-path <NEW_PATH>` | New local filesystem path |
| `--id <ID>`, `-i <ID>` | New unique identifier |
| `--branch-lock` | Whether the branch is locked (cannot be changed) |
| `--add-tags <ADD_TAGS>` | Tags to add (comma-separated) |
| `--remove-tags <REMOVE_TAGS>` | Tags to remove (comma-separated) |

#### Examples

```bash
# Update a repository's origin
mirror-cli update --path "example/repo" --origin "git@github.com:new/repo.git"

# Update a repository's branch
mirror-cli update --path "example/repo" --branch "develop"

# Update a repository's path
mirror-cli update --path "example/repo" --new-path "new/path"

# Add tags to a repository
mirror-cli update --path "example/repo" --add-tags "important,production"

# Remove tags from a repository
mirror-cli update --path "example/repo" --remove-tags "test"

# Update multiple properties at once
mirror-cli update --path "example/repo" --branch "develop" --add-tags "important" --branch-lock
```

### `validate`

Validate the mirror.toml file.

```bash
mirror-cli validate
```

#### Examples

```bash
# Validate the default mirror.toml file
mirror-cli validate

# Validate a specific mirror.toml file
mirror-cli --config custom-mirror.toml validate
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `MIRROR_CONFIG` | Path to the mirror.toml file. This is used if the `--config` option is not specified. |

## Exit Codes

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | Error (details will be printed to stderr) |

## Color Coding

The CLI uses color coding to improve readability:

| Color | Usage |
|-------|-------|
| Green | Success messages |
| Red | Error messages |
| Blue | Informational messages |
| Yellow | Repository numbers in list output |

## Common Workflows

### Creating and Populating a Configuration

```bash
# Initialize a new configuration
mirror-cli init

# Add repositories
mirror-cli add --origin "git@github.com:example/repo1.git" --path "example/repo1"
mirror-cli add --origin "git@github.com:example/repo2.git" --path "example/repo2" --tags "example"

# List the repositories
mirror-cli list
```

### Managing Repository Tags

```bash
# Add tags to a repository
mirror-cli update --path "example/repo1" --add-tags "important,production"

# List repositories with a specific tag
mirror-cli list --tag "important"

# Remove a tag from a repository
mirror-cli update --path "example/repo1" --remove-tags "production"
```

### Using a Custom Configuration File

```bash
# Specify the configuration file path
mirror-cli --config custom-mirror.toml list

# Or use the environment variable
export MIRROR_CONFIG=custom-mirror.toml
mirror-cli list