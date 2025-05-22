# Decision Log

This file records architectural and implementation decisions using a list format.
2025-05-22 20:00:54 - Initial creation of Memory Bank.

## Decision

* Create a comprehensive design document for the mctl CLI tool before implementation
* Focus on covering all major functionality of the mirror-sdk
* Design with user experience as a priority

## Rationale 

* A thorough design document will ensure all requirements are addressed
* Covering all SDK functionality ensures the CLI is complete and useful
* Good UX is critical for CLI tools to ensure adoption and ease of use

## Implementation Details

* Will analyze mirror-sdk to understand its full capabilities
* Will design command structure based on common CLI patterns and SDK functionality
* Will recommend appropriate CLI framework based on requirements
2025-05-22 20:02:47 - Added design decisions based on SDK analysis.

## Decision

* Use a command structure that follows Git-style subcommands (e.g., `mctl init`, `mctl repo add`)
* Recommend Clap as the CLI framework for the implementation
* Implement color-coded output for better user experience
* Provide both concise and verbose output modes

## Rationale 

* Git-style commands are familiar to developers and provide a consistent mental model
* Clap is a mature, feature-rich Rust CLI framework with excellent documentation and support
* Color-coded output improves readability and helps users quickly identify important information
* Different verbosity levels accommodate both quick usage and detailed information needs

## Implementation Details

* Each subcommand will map directly to SDK functionality
* Error messages will be user-friendly with suggestions for resolution
* Will provide shell completion scripts for common shells
* Will include comprehensive help text for all commands

2025-05-22 21:43:23 - Added cargo installation support.

## Decision

* Update Cargo.toml to make mctl installable from cargo
* Add metadata fields for better discoverability on crates.io
* Configure mirror-sdk dependency to support both development and publishing

## Rationale

* Making mctl installable from cargo improves accessibility for users
* Proper metadata helps users find and understand the tool on crates.io
* Supporting both development and publishing workflows ensures smooth transition to production

## Implementation Details

* Added repository, documentation, readme, keywords, and categories metadata
* Added [[bin]] section to specify binary name and path
* Configured mirror-sdk dependency with comments for development vs. publishing
2025-05-22 21:52:24 - Implemented status command for mctl.

## Decision

* Implement a new `status` command for mctl to show git status of all repositories
* Make paths relative to the mirror.toml file location for better usability
* Follow the same design patterns as existing commands

## Rationale

* Git status is a common operation that users need to perform across multiple repositories
* Relative paths make it easier to navigate to files in an IDE when they're clicked
* Consistent command structure maintains a predictable user experience
* Unified status view provides a holistic overview of the entire mirror configuration

## Implementation Details

* Used git2 crate to interact with git repositories
* Implemented path resolution relative to mirror.toml location
* Added support for filtering repositories by tag
* Formatted git status output similar to standard git status but with full paths
* Maintained consistent error handling and output formatting with other commands
2025-05-22 21:58:13 - Enhanced status command with improved visual presentation.

## Decision

* Enhance the `mctl status` command with improved visual presentation and usability
* Skip displaying repositories that are clean by default (with option to show them)
* Add color coding for different git status types
* Improve formatting with better indentation and visual cues

## Rationale

* Hiding clean repositories makes the output more concise and focused on what needs attention
* Color coding helps users quickly identify different types of changes
* Better formatting improves readability and makes the output more visually appealing
* Adding a status legend helps users understand the meaning of status codes

## Implementation Details

* Modified `status.rs` to skip repositories with no changes by default
* Added a new `--show-clean` flag to optionally display clean repositories
* Enhanced `format_git_status` to return colored status indicators
* Used a modern, clean design with minimal visual elements
* Included a status legend at the end of the output when changes are present
* Used different colors for staged vs. unstaged changes for better distinction

2025-05-22 22:01:04 - Refined status command with a more modern design.

## Decision

* Update the status command with a more modern, cleaner visual design
* Remove emoji icons in favor of simpler, more professional indicators
* Maintain color coding but with a more subtle and professional appearance

## Rationale

* A cleaner, more modern design improves the professional appearance of the tool
* Simpler indicators (like "→") provide visual cues without being distracting
* Consistent spacing and formatting enhances readability while maintaining a clean look

## Implementation Details

* Replaced emoji icons with simpler arrow indicators
* Standardized indentation and spacing for a cleaner appearance
* Simplified the status legend format while maintaining color coding
* Ensured consistent formatting between clean and modified repositories
2025-05-22 22:05:35 - Updated status command to respect .gitignore files.

## Decision

* Modify the status command to exclude files that are in .gitignore
* Configure git status options to filter out ignored files

## Rationale

* Showing ignored files clutters the output with files that are intentionally excluded from version control
* Users typically don't need to see ignored files in the status output
* This behavior is more consistent with the standard git status command

## Implementation Details

* Used git2's StatusOptions to configure what files are included in the status
* Set include_ignored(false) to exclude files that are in .gitignore
* Maintained other status options (include_untracked, exclude_submodules) for consistent behavior
* Successfully tested local installation with `cargo install --path .`
2025-05-22 22:08:31 - Further refined status command with improved visual design.

## Decision

* Remove the status legend for a cleaner output
* Color the file paths based on their status type, not just the status indicators
* Show changed and untracked files separately for better organization

## Rationale

* Removing the legend creates a cleaner, more streamlined output
* Coloring the entire file path makes it easier to identify file types at a glance
* Separating changed and untracked files provides better organization and clarity

## Implementation Details

* Removed the status legend from the output
* Created a new function to color file paths based on their git status
* Collected and displayed changed and untracked files in separate sections
* Maintained consistent color coding for different status types