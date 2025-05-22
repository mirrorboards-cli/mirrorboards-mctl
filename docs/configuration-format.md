# Mirror Configuration File Format

This document describes the format of the mirror.toml configuration file used by the Mirror SDK and CLI.

## Overview

The mirror.toml file is a TOML-formatted configuration file that defines a collection of Git repositories to be managed. It uses the [TOML](https://toml.io/) format, which is designed to be easy to read and write.

## File Structure

The mirror.toml file consists of an array of repository definitions, each represented by a `[[repositories]]` section.

### Basic Example

```toml
[[repositories]]
origin = "git@github.com:example/repo1.git"
branch = "main"
path = "example/repo1"

[[repositories]]
origin = "git@github.com:example/repo2.git"
branch = "develop"
path = "example/repo2"
tags = ["example", "test"]
```

## Repository Definition

Each repository is defined by a `[[repositories]]` section with the following fields:

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `origin` | String | Yes | - | The Git repository origin URL. This can be an HTTPS or SSH URL. |
| `branch` | String | Yes | "main" | The Git branch to use. |
| `path` | String | Yes | - | The local filesystem path where the repository should be cloned. This can be absolute or relative to the mirror.toml file. |
| `id` | String | No | - | A unique identifier for the repository. If not provided, the path is used as the identifier. |
| `branch-lock` | Boolean | No | false | Whether the branch is locked (cannot be changed). |
| `tags` | Array of Strings | No | [] | Tags for categorizing repositories. |

### Field Details

#### `origin`

The Git repository origin URL. This is typically an SSH URL (e.g., `git@github.com:example/repo.git`) or an HTTPS URL (e.g., `https://github.com/example/repo.git`).

Example:
```toml
origin = "git@github.com:example/repo.git"
```

#### `branch`

The Git branch to use. This is typically "main" or "master", but can be any valid Git branch name.

Example:
```toml
branch = "main"
```

#### `path`

The local filesystem path where the repository should be cloned. This can be absolute or relative to the mirror.toml file.

Example:
```toml
path = "example/repo"
```

#### `id` (optional)

A unique identifier for the repository. If not provided, the path is used as the identifier. This is useful for repositories with long or complex paths.

Example:
```toml
id = "example-repo"
```

#### `branch-lock` (optional)

Whether the branch is locked (cannot be changed). This is useful for repositories that should always use a specific branch.

Example:
```toml
branch-lock = true
```

#### `tags` (optional)

Tags for categorizing repositories. This is an array of strings.

Example:
```toml
tags = ["example", "test"]
```

## Complete Example

```toml
# Repository with minimal configuration
[[repositories]]
origin = "git@github.com:example/repo1.git"
branch = "main"
path = "example/repo1"

# Repository with a unique ID
[[repositories]]
id = "example-repo"
origin = "git@github.com:example/repo2.git"
branch = "develop"
path = "example/repo2"

# Repository with a locked branch
[[repositories]]
origin = "git@github.com:example/repo3.git"
branch = "stable"
branch-lock = true
path = "example/repo3"

# Repository with tags
[[repositories]]
origin = "git@github.com:example/repo4.git"
branch = "main"
path = "example/repo4"
tags = ["example", "test"]

# Repository with all options
[[repositories]]
id = "full-example"
origin = "git@github.com:example/repo5.git"
branch = "develop"
branch-lock = true
path = "example/repo5"
tags = ["example", "test", "full"]
```

## Validation Rules

The Mirror SDK and CLI apply the following validation rules to the mirror.toml file:

1. Each repository must have a unique path.
2. If provided, each repository must have a unique ID.
3. The origin URL must be a valid Git URL.
4. The path must be a valid filesystem path.
5. The branch must be a valid Git branch name.

## Comments

TOML supports comments using the `#` character. Comments can be used to add notes or to temporarily disable repositories.

Example:
```toml
# This is a comment
[[repositories]]
origin = "git@github.com:example/repo.git"
branch = "main"
path = "example/repo"

# Temporarily disabled repository
# [[repositories]]
# origin = "git@github.com:example/disabled-repo.git"
# branch = "main"
# path = "example/disabled-repo"
```

## Organizing Repositories

You can use comments and tags to organize repositories in the mirror.toml file.

Example:
```toml
# Frontend repositories
[[repositories]]
origin = "git@github.com:example/frontend.git"
branch = "main"
path = "frontend"
tags = ["frontend"]

[[repositories]]
origin = "git@github.com:example/ui-components.git"
branch = "main"
path = "ui-components"
tags = ["frontend", "components"]

# Backend repositories
[[repositories]]
origin = "git@github.com:example/api.git"
branch = "main"
path = "backend/api"
tags = ["backend", "api"]

[[repositories]]
origin = "git@github.com:example/database.git"
branch = "main"
path = "backend/database"
tags = ["backend", "database"]
```

## Environment Variables

The Mirror SDK and CLI support the following environment variables for working with mirror.toml files:

| Variable | Description |
|----------|-------------|
| `MIRROR_CONFIG` | Path to the mirror.toml file. This is used if the `--config` option is not specified. |

## Best Practices

1. **Use meaningful paths**: Choose paths that reflect the repository's purpose or structure.
2. **Use tags for categorization**: Tags make it easier to filter and manage repositories.
3. **Add comments for context**: Comments help explain why repositories are included or how they relate to each other.
4. **Use IDs for complex paths**: If a repository has a long or complex path, consider adding a unique ID for easier reference.
5. **Group related repositories**: Use comments to group related repositories together in the file.
6. **Lock critical branches**: Use branch-lock for repositories that should always use a specific branch.
7. **Validate your configuration**: Use the `validate` command to check your mirror.toml file for errors.