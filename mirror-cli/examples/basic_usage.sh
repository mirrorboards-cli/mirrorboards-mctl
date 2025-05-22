#!/bin/bash
# Basic usage examples for mirror-cli

# Ensure the script exits on error
set -e

echo "Mirror CLI Basic Usage Examples"
echo "==============================="

# Create a temporary directory for testing
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"
echo "Working in temporary directory: $TEMP_DIR"
echo

# Initialize a new mirror.toml file
echo "1. Initializing a new mirror.toml file"
mirror-cli init
echo

# Add repositories
echo "2. Adding repositories"
mirror-cli add --origin "git@github.com:example/repo1.git" --path "example/repo1" --tags "example,test"
echo "Added first repository"

mirror-cli add --origin "git@github.com:example/repo2.git" --path "example/repo2" --branch "develop" --id "custom-id" --branch-lock
echo "Added second repository"
echo

# List repositories
echo "3. Listing all repositories"
mirror-cli list
echo

# List repositories with a specific tag
echo "4. Listing repositories with tag 'example'"
mirror-cli list --tag "example"
echo

# Update a repository
echo "5. Updating a repository"
mirror-cli update --path "example/repo1" --origin "git@github.com:example/updated-repo.git" --add-tags "important"
echo

# List repositories after update
echo "6. Listing repositories after update"
mirror-cli list
echo

# Validate the configuration
echo "7. Validating the configuration"
mirror-cli validate
echo

# Remove a repository
echo "8. Removing a repository by path"
mirror-cli remove --path "example/repo2"
echo

# List repositories after removal
echo "9. Listing repositories after removal"
mirror-cli list
echo

# Clean up
echo "Cleaning up temporary directory"
cd ..
rm -rf "$TEMP_DIR"

echo "All examples completed successfully!"