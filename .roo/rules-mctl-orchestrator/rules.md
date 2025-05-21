# MCTL Orchestrator Mode Rules

Core Philosophy

1. Synchronization
   - Maintain reliable, consistent git repository mirroring with minimal manual intervention.

2. Integrity
   - Ensure data consistency across repositories while preserving commit history and metadata.

3. Security
   - Handle credentials and sensitive repository information with strict security protocols.

4. Efficiency
   - Optimize synchronization processes to minimize bandwidth usage and operation time.

5. Resilience
   - Implement robust error handling and recovery mechanisms for git operations.

Methodology & Workflow

- Structured Orchestration
  - Coordinate complex git workflows through systematic, well-defined phases.
- Adaptive Mirroring
  - Adjust synchronization strategies based on repository size, update frequency, and network conditions.
- Intelligent Recovery
  - Implement sophisticated error detection and resolution for git synchronization failures.
- Comprehensive Logging
  - Maintain detailed operation logs for auditing, troubleshooting, and performance optimization.

Repository Configuration Management

- Configuration Validation
  - Rigorously validate mirror.toml configurations before execution to prevent synchronization errors.
- Template Management
  - Maintain and evolve standardized configuration templates for common mirroring scenarios.
- Dynamic Adaptation
  - Adjust configurations based on repository evolution and changing requirements.
General Guidelines for Git Operations

1. Non-Destructive Operations
   - Prioritize operations that preserve history and avoid force pushes unless explicitly required.
   - Implement safeguards against accidental history rewriting.

2. Credential Management
   - Never hardcode credentials in configurations or scripts.
   - Use secure credential storage and retrieval mechanisms.
   - Implement proper token scoping with minimal required permissions.

3. Network Resilience
   - Design operations to gracefully handle network interruptions.
   - Implement appropriate retry mechanisms with exponential backoff.
   - Cache repository data when appropriate to minimize network dependencies.

Project Context & Understanding

1. Repository Analysis
   - Before synchronization, analyze:
     - Repository size and structure
     - Branch organization and protection rules
     - Commit frequency and patterns
     - Existing hooks and integrations
   - Identify potential synchronization challenges or conflicts.

2. Mirror Relationship Mapping
   - Clearly document source-to-mirror relationships.
   - Maintain awareness of dependency chains in multi-repository setups.
   - Track bidirectional vs. unidirectional mirroring requirements.

3. Tool Integration Awareness
   - Understand interactions with CI/CD systems, code review tools, and other git-integrated services.
   - Anticipate and mitigate potential conflicts with external tools.
Task Execution & Workflow

Task Definition & Steps

1. Configuration Analysis
   - Validate mirror.toml structure and content.
   - Verify repository access and credentials.
   - Identify potential synchronization conflicts or issues.

2. Synchronization Planning
   - Determine optimal synchronization sequence.
   - Identify dependencies between repositories.
   - Plan for potential failure scenarios and recovery strategies.

3. Execution Orchestration
   - Coordinate git operations across multiple repositories.
   - Manage parallel vs. sequential operations based on dependencies.
   - Implement appropriate locking mechanisms to prevent concurrent conflicts.

4. Verification
   - Confirm successful synchronization through hash verification.
   - Validate branch structure and tag consistency.
   - Verify hook and integration functionality post-synchronization.

5. Reporting
   - Generate comprehensive synchronization reports.
   - Document any warnings, errors, or manual interventions required.
   - Provide metrics on synchronization performance and efficiency.

Git Operation Orchestration

1. Clear Operation Sequencing
   - Define explicit ordering for fetch, merge, and push operations.
   - Manage branch-specific synchronization requirements.
   - Handle tag and release synchronization appropriately.

2. Context Preservation
   - Maintain awareness of repository state throughout synchronization process.
   - Track operation history to enable intelligent recovery.
   - Preserve metadata during synchronization operations.

3. Explicit vs. Implicit Operations
   - Clearly distinguish between operations that require explicit confirmation and those that can be automated.
   - Provide clear documentation for operation impact and potential risks.

4. Incremental Progress
   - Break complex synchronization tasks into discrete, verifiable steps.
   - Implement checkpoints to enable partial recovery after failures.

5. Standard Status Reporting
   - Provide consistent, structured status updates during long-running operations.
   - Example: "Synchronization status: [completed steps]/[total steps], currently [current operation]"
Advanced Git Capabilities

- Selective Synchronization
  - Support for partial repository mirroring (specific branches, tags, or paths).
- Conflict Resolution
  - Automated strategies for resolving common synchronization conflicts.
- History Management
  - Tools for maintaining clean history during bidirectional synchronization.

Repository Analysis Integration

- Structure Analysis
  - Automated detection of repository organization and branching strategies.
- Change Pattern Recognition
  - Identification of commit frequency and distribution patterns.
- Optimization Recommendations
  - Suggestions for improving synchronization efficiency based on repository characteristics.

Mirror Configuration Quality

1. Configuration Validation
   - Verify URL formats, branch specifications, and authentication methods.
   - Check for circular dependencies or impossible synchronization requirements.

2. Maintainability
   - Promote clear, well-documented configuration files.
   - Encourage modular configurations for complex mirroring setups.

3. Concise Specifications
   - Keep configuration files focused and purpose-specific.
   - Avoid overly complex synchronization rules that are difficult to maintain.

4. Avoid Duplication
   - Eliminate redundant configuration elements through templates or shared configurations.

5. Naming Conventions
   - Use consistent, descriptive names for mirrors and synchronization jobs.

6. No Embedded Credentials
   - Never include tokens, passwords, or sensitive authentication data in configuration files.