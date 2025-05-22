# System Patterns

This file documents recurring patterns and standards used in the project.
It is optional, but recommended to be updated as the project evolves.
2025-05-22 20:01:03 - Initial creation of Memory Bank.

## Coding Patterns

* CLI command structure will follow a consistent pattern (verb-noun)
* Error handling will be consistent across all commands
* Configuration file handling will follow SDK patterns

## Architectural Patterns

* Command-line interface will use a layered architecture:
  * CLI layer (user interaction)
  * Command processing layer (validation, parsing)
  * SDK integration layer (calls to mirror-sdk)
* Each command will be modular and follow the same structure

## Testing Patterns

* Each command should have unit tests
* Integration tests should verify end-to-end functionality
* User experience should be validated with usability testing
2025-05-22 20:02:57 - Added CLI design patterns based on SDK analysis.

## Coding Patterns

* Command structure will follow the pattern: `mctl <command> [subcommand] [options] [arguments]`
* Options will use both short (-v) and long (--verbose) forms
* Error handling will follow a consistent pattern:
  * User errors (e.g., invalid input) will show helpful messages
  * System errors (e.g., file access issues) will show technical details
  * All errors will include suggestions for resolution when possible
* Configuration file handling will follow SDK patterns with additional validation

## Architectural Patterns

* Command-line interface will use a layered architecture:
  * CLI layer: Handles user input parsing, help text, and output formatting
  * Command layer: Implements command logic and validation
  * SDK layer: Interfaces with the mirror-sdk
* Each command will be implemented as a separate module
* Common functionality will be extracted into utility modules
* Output formatting will be consistent across all commands

## Testing Patterns

* Unit tests for each command module
* Integration tests for end-to-end command execution
* Mock tests for SDK interactions
* Test coverage for error handling scenarios