# Mirror CLI

A command-line interface for managing mirror.toml configuration files using the mirror-sdk.

## Features

- Create new mirror.toml configuration files
- Add, remove, and update repositories
- List repositories with optional tag filtering
- Validate mirror.toml configurations
- Colorful terminal output for better user experience
- Specify mirror.toml file path via command-line argument or environment variable

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/mirrorboards/mirror-cli.git
cd mirror-cli

# Build the project
cargo build --release

# The binary will be available at target/release/mirror-cli
```

## Usage

```bash
# Get help
mirror-cli --help

# Initialize a new mirror.toml file
mirror-cli init

# Add a repository
mirror-cli add --origin "git@github.com:example/repo.git" --path "example/repo"

# List all repositories
mirror-cli list

# List repositories with a specific tag
mirror-cli list --tag "example"

# Remove a repository by path
mirror-cli remove --path "example/repo"

# Remove a repository by ID
mirror-cli remove --id "repo-id"

# Update a repository
mirror-cli update --path "example/repo" --origin "git@github.com:new/repo.git"

# Validate the configuration
mirror-cli validate
```

## Environment Variables

- `MIRROR_CONFIG`: Path to the mirror.toml file (overrides default)

## Command Reference

### `init`

Create a new empty mirror.toml file.

```bash
mirror-cli init [--force]
```

Options:
- `--force`: Force creation even if file exists

### `add`

Add a new repository to the configuration.

```bash
mirror-cli add --origin <ORIGIN> --path <PATH> [OPTIONS]
```

Options:
- `--origin`, `-o`: Git repository origin URL (required)
- `--branch`, `-b`: Git branch to use (default: "main")
- `--path`, `-p`: Local filesystem path where the repository should be cloned (required)
- `--id`, `-i`: Optional unique identifier for the repository
- `--branch-lock`: Whether the branch is locked (cannot be changed)
- `--tags`, `-t`: Optional tags for categorizing repositories (comma-separated)

### `remove`

Remove a repository from the configuration.

```bash
mirror-cli remove --path <PATH> | --id <ID>
```

Options:
- `--path`, `-p`: Repository path to remove
- `--id`, `-i`: Repository ID to remove

### `list`

List all repositories in the configuration.

```bash
mirror-cli list [--tag <TAG>]
```

Options:
- `--tag`, `-t`: Filter repositories by tag

### `update`

Update a repository's properties.

```bash
mirror-cli update --path <PATH> [OPTIONS]
```

Options:
- `--path`, `-p`: Repository path to update (required)
- `--origin`, `-o`: New Git repository origin URL
- `--branch`, `-b`: New Git branch to use
- `--new-path`: New local filesystem path
- `--id`, `-i`: New unique identifier
- `--branch-lock`: Whether the branch is locked (cannot be changed)
- `--add-tags`: Tags to add (comma-separated)
- `--remove-tags`: Tags to remove (comma-separated)

### `validate`

Validate the mirror.toml file.

```bash
mirror-cli validate
```

## Examples

### Creating a New Configuration

```bash
# Initialize a new mirror.toml file
mirror-cli init

# Add a repository
mirror-cli add --origin "git@github.com:example/repo.git" --path "example/repo" --branch "main" --tags "example,test"
```

### Managing Repositories

```bash
# List all repositories
mirror-cli list

# Update a repository's origin
mirror-cli update --path "example/repo" --origin "git@github.com:new/repo.git"

# Add tags to a repository
mirror-cli update --path "example/repo" --add-tags "important,production"

# Remove tags from a repository
mirror-cli update --path "example/repo" --remove-tags "test"
```

### Using a Custom Configuration File

```bash
# Specify the configuration file path
mirror-cli --config custom-mirror.toml list

# Or use the environment variable
export MIRROR_CONFIG=custom-mirror.toml
mirror-cli list
```

## License

MIT