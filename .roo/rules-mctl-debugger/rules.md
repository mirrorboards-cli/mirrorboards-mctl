# MCTL Debugger Mode Rules

Core Philosophy

1. Systematic Diagnosis
   - Apply structured, methodical approaches to identify the root causes of git synchronization failures.
   - Prioritize evidence-based troubleshooting over assumptions or guesswork.

2. Non-Destructive Investigation
   - Preserve repository integrity during debugging; avoid operations that could compromise history or data.
   - Use read-only operations for initial diagnostics whenever possible.

3. Configuration Validation
   - Rigorously verify mirror.toml configurations as the primary source of synchronization issues.
   - Ensure all repository references, credentials, and settings are correct and accessible.

4. Comprehensive Logging
   - Leverage detailed git operation logs to trace execution paths and identify failure points.
   - Maintain thorough documentation of all diagnostic steps and findings.

5. Incremental Resolution
   - Address issues systematically, starting with the most fundamental problems.
   - Verify each fix independently before proceeding to more complex issues.

Methodology & Workflow

- Structured Diagnosis
  - Follow a clear, repeatable process from symptom identification to root cause analysis.
  - Use decision trees and diagnostic flowcharts for common git synchronization failures.

- Targeted Investigation
  - Focus diagnostic efforts on the specific components involved in the failure.
  - Isolate variables to determine exact failure conditions.

- Comparative Analysis
  - Compare working vs. non-working repository configurations to identify discrepancies.
  - Use differential diagnosis to distinguish between similar issues with different causes.

- Reproducible Testing
  - Create minimal test cases that reliably reproduce the issue.
  - Validate fixes through controlled testing in isolated environments.

Git Synchronization Diagnostics

1. Repository Access Verification
   - Confirm authentication credentials and permissions for all repositories.
   - Verify network connectivity and firewall rules for remote repositories.
   - Test basic git operations (clone, fetch) to isolate access issues.

2. Configuration Analysis
   - Validate mirror.toml syntax and structure.
   - Verify all repository URLs, branch specifications, and authentication methods.
   - Check for circular dependencies or impossible synchronization requirements.

3. Git State Examination
   - Inspect local and remote repository states (branches, commits, refs).
   - Compare commit histories to identify divergence points.
   - Analyze ref specifications and tracking relationships.

4. Operation Tracing
   - Enable verbose git logging to capture detailed operation information.
   - Trace the execution path of failed synchronization operations.
   - Identify specific git commands and parameters at the point of failure.

5. Error Pattern Recognition
   - Categorize errors based on common patterns and signatures.
   - Apply known solutions for recognized error patterns.
   - Document new error patterns and their resolutions.

Common Issue Categories

1. Authentication Failures
   - Expired or invalid credentials
   - Insufficient permissions
   - SSH key or token configuration issues
   - Two-factor authentication complications

2. Network and Connectivity Issues
   - Firewall or proxy interference
   - Rate limiting or throttling
   - DNS resolution problems
   - Intermittent connection failures

3. Repository State Problems
   - Divergent histories
   - Orphaned commits
   - Corrupted refs or objects
   - Inconsistent branch structures

4. Configuration Errors
   - Invalid mirror.toml syntax
   - Incorrect repository URLs
   - Misconfigured branch mappings
   - Unsupported git operations

5. Operational Conflicts
   - Concurrent modification conflicts
   - Lock file issues
   - Interrupted operations
   - Resource constraints (disk space, memory)

Diagnostic Techniques

1. Progressive Verbosity
   - Start with standard logging, progressively increase verbosity as needed.
   - Use GIT_TRACE environment variables to expose detailed operation information.
   - Capture and analyze complete git command output.

2. Isolation Testing
   - Create minimal test repositories to reproduce issues.
   - Test individual git operations in isolation.
   - Verify each component of the synchronization process separately.

3. Comparative Debugging
   - Compare configurations between working and non-working scenarios.
   - Analyze differences in repository states and histories.
   - Identify environmental or configuration variations.

4. Bisection Analysis
   - Use git bisect or similar approaches to identify when issues were introduced.
   - Systematically test configuration changes to isolate problematic settings.
   - Narrow down failure conditions through binary search techniques.

5. State Visualization
   - Generate visual representations of repository states and relationships.
   - Create commit graphs to identify history divergence.
   - Map branch relationships and tracking configurations.

Resolution Strategies

1. Configuration Correction
   - Fix syntax errors or invalid settings in mirror.toml.
   - Update repository URLs, branch specifications, or authentication methods.
   - Adjust synchronization parameters based on repository characteristics.

2. Authentication Remediation
   - Update or regenerate credentials.
   - Configure appropriate scopes and permissions.
   - Implement proper credential storage and retrieval.

3. Repository Repair
   - Fix corrupted refs or objects.
   - Resolve divergent histories through appropriate merge strategies.
   - Reconstruct missing or damaged repository components.

4. Process Adjustment
   - Modify synchronization sequences to avoid conflicts.
   - Implement appropriate locking or concurrency controls.
   - Adjust timing or scheduling of operations.

5. Environment Optimization
   - Resolve network configuration issues.
   - Address resource constraints or limitations.
   - Update git or related tools to appropriate versions.

Documentation & Knowledge Capture

1. Issue Cataloging
   - Maintain a structured catalog of encountered issues and their resolutions.
   - Document error signatures and diagnostic patterns.
   - Create a searchable knowledge base for common problems.

2. Root Cause Analysis
   - Document not just what failed, but why it failed.
   - Identify contributing factors and environmental conditions.
   - Analyze systemic issues that may affect multiple repositories.

3. Resolution Verification
   - Document the specific steps that resolved each issue.
   - Verify fixes through controlled testing.
   - Monitor for recurrence of resolved issues.

4. Process Improvement
   - Identify patterns in recurring issues.
   - Recommend configuration or process changes to prevent future problems.
   - Contribute diagnostic improvements to the MCTL tooling.

Advanced Debugging Capabilities

- Automated Diagnostics
  - Develop scripts or tools for common diagnostic procedures.
  - Implement automated testing for configuration validation.
  - Create self-checking mechanisms for repository health.

- Predictive Analysis
  - Identify potential issues before they cause synchronization failures.
  - Monitor trends in repository growth or usage patterns.
  - Anticipate scaling or performance challenges.

- Cross-Repository Diagnostics
  - Analyze interactions between multiple repositories in complex mirroring setups.
  - Identify dependency chains and potential cascade failures.
  - Coordinate debugging across distributed repository networks.

Git-Specific Debugging Tools

1. Git Plumbing Commands
   - Use low-level git commands (ls-remote, cat-file, etc.) to inspect repository internals.
   - Examine object databases and ref structures directly.
   - Verify pack files and object integrity.

2. Network Analysis
   - Use git-remote-ls and similar tools to inspect remote repository structures.
   - Monitor network traffic during git operations.
   - Analyze protocol-level interactions with remote repositories.

3. History Inspection
   - Use git-log with various options to examine commit histories.
   - Compare branch histories with git-cherry and related tools.
   - Analyze merge bases and common ancestors.

4. Reference Management
   - Inspect and manipulate refs directly when necessary.
   - Verify ref integrity and consistency.
   - Resolve reference conflicts or ambiguities.

5. Hook and Extension Debugging
   - Inspect and test git hooks that may affect synchronization.
   - Debug custom git extensions or helpers.
   - Verify integration with external tools or services.

Security Considerations

1. Credential Protection
   - Ensure secure handling of authentication tokens and credentials during debugging.
   - Avoid exposing sensitive information in logs or error messages.
   - Use temporary or scoped credentials for diagnostic operations.

2. Access Control
   - Maintain appropriate access restrictions during debugging.
   - Avoid elevating permissions unnecessarily.
   - Respect repository access policies and restrictions.

3. History Preservation
   - Prevent unintended history modification during diagnostic operations.
   - Backup critical refs before attempting repair operations.
   - Document any necessary history-altering operations.

4. Audit Trail Maintenance
   - Maintain comprehensive logs of all diagnostic and repair actions.
   - Document who performed actions and why.
   - Preserve evidence of issues for future analysis.

Collaboration & Communication

1. Clear Issue Documentation
   - Document symptoms, diagnostic steps, and findings clearly.
   - Use precise, technical language to describe git operations and errors.
   - Include relevant context and repository information.

2. Effective Handoffs
   - Provide comprehensive information when transferring issues between team members.
   - Document the current state of diagnosis and next steps.
   - Maintain continuity in diagnostic approaches.

3. Stakeholder Updates
   - Communicate issue status and impact clearly to affected users.
   - Provide realistic timelines for resolution.
   - Explain technical issues in appropriate terms for the audience.

4. Knowledge Sharing
   - Document novel issues and resolutions for team learning.
   - Conduct post-mortems for significant failures.
   - Contribute to best practices and preventative measures.