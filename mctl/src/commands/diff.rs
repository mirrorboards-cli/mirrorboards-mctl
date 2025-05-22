//! Diff command implementation
//!
//! This module implements git diff functionality across multiple repositories.

use std::path::{Path, PathBuf};
use git2::{Repository as GitRepository, DiffOptions, DiffFormat};
use mirror_sdk::MirrorConfig;
use crate::cli::diff::DiffArgs;
use crate::output::OutputFormatter;
use crate::utils::resolve_relative_path;
use super::{CommandResult, CommandError};
use colored::*;
use glob::Pattern;

/// Statistics for a diff operation
#[derive(Debug, Default)]
struct DiffStats {
    files_changed: usize,
    insertions: usize,
    deletions: usize,
}

/// Result of a diff operation for a single repository
#[derive(Debug)]
struct DiffResult {
    repository_name: String,
    repository_path: PathBuf,
    has_changes: bool,
    diff_output: String,
    stats: DiffStats,
    changed_files: Vec<String>,
}

/// Execute the diff command
pub fn execute(args: DiffArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    // Load the mirror.toml file
    let config_path_str = config_path.clone().unwrap_or_else(|| "mirror.toml".to_string());
    let config_path_buf = PathBuf::from(&config_path_str);
    
    // Load the mirror.toml file
    let config = if let Some(path) = config_path {
        formatter.info(&format!("Loading mirror.toml from {}", path));
        MirrorConfig::load_from(path)
    } else {
        formatter.info("Loading mirror.toml from default location");
        MirrorConfig::load()
    }?;

    // Get repositories, filtered by tag and/or ID
    let repositories = filter_repositories(&config, &args);

    if repositories.is_empty() {
        formatter.warning("No repositories found matching the specified criteria");
        return Ok(());
    }

    formatter.info(&format!("Found {} repositories to diff", repositories.len()));

    // Compile include/exclude patterns
    let include_patterns: Result<Vec<Pattern>, _> = args.include.iter()
        .map(|p| Pattern::new(p).map_err(|e| CommandError::Input(format!("Invalid include pattern '{}': {}", p, e))))
        .collect();
    let include_patterns = include_patterns?;

    let exclude_patterns: Result<Vec<Pattern>, _> = args.exclude.iter()
        .map(|p| Pattern::new(p).map_err(|e| CommandError::Input(format!("Invalid exclude pattern '{}': {}", p, e))))
        .collect();
    let exclude_patterns = exclude_patterns?;

    // Process each repository
    let mut diff_results = Vec::new();
    let mut total_stats = DiffStats::default();

    for repo in repositories {
        let repo_path_str = &repo.path;
        let repo_path = resolve_relative_path(&config_path_buf, repo_path_str);
        
        // Check if repository exists
        if !repo_path.exists() {
            formatter.warning(&format!("Repository not found at {}", repo_path.display()));
            continue;
        }

        // Generate diff for this repository
        match generate_repository_diff(&repo_path, &args, &include_patterns, &exclude_patterns) {
            Ok(diff_result) => {
                // Update total statistics
                total_stats.files_changed += diff_result.stats.files_changed;
                total_stats.insertions += diff_result.stats.insertions;
                total_stats.deletions += diff_result.stats.deletions;

                diff_results.push(diff_result);
            },
            Err(e) => {
                formatter.error(&format!("Failed to generate diff for {}: {}", repo_path.display(), e));
                continue;
            }
        }
    }

    // Filter results if changes_only is specified
    if args.changes_only {
        diff_results.retain(|result| result.has_changes);
    }

    // Display results
    if diff_results.is_empty() && args.changes_only {
        formatter.success("No repositories with changes found");
        return Ok(());
    }

    format_diff_output(&diff_results, &args, formatter, &total_stats)?;

    Ok(())
}

/// Filter repositories based on tag and ID criteria
fn filter_repositories<'a>(config: &'a MirrorConfig, args: &DiffArgs) -> Vec<&'a mirror_sdk::Repository> {
    let mut repositories: Vec<&'a mirror_sdk::Repository> = if let Some(tag) = &args.tag {
        config.get_repositories_by_tag(tag)
    } else {
        config.get_repositories().iter().collect()
    };

    // Further filter by IDs if specified
    if !args.id.is_empty() {
        repositories.retain(|repo| {
            if let Some(id) = &repo.id {
                args.id.contains(id)
            } else {
                false
            }
        });
    }

    repositories
}

/// Generate diff for a single repository
fn generate_repository_diff(
    repo_path: &Path,
    args: &DiffArgs,
    include_patterns: &[Pattern],
    exclude_patterns: &[Pattern],
) -> CommandResult<DiffResult> {
    // Open the git repository
    let git_repo = GitRepository::open(repo_path).map_err(|e| {
        CommandError::Other(format!("Failed to open repository at {}: {}", repo_path.display(), e))
    })?;

    let repo_name = repo_path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| repo_path.to_string_lossy().to_string());

    // Set up diff options
    let mut diff_opts = DiffOptions::new();
    diff_opts.context_lines(args.context);
    diff_opts.ignore_whitespace(false);

    // Add pathspec filters if include patterns are specified
    if !include_patterns.is_empty() {
        for pattern in include_patterns {
            diff_opts.pathspec(pattern.as_str());
        }
    }

    // Get the diff based on the specified mode
    let diff = if args.staged {
        // Staged changes (index vs HEAD)
        let head_commit = git_repo.head()
            .map_err(|e| CommandError::Other(format!("Failed to get HEAD: {}", e)))?
            .peel_to_commit()
            .map_err(|e| CommandError::Other(format!("Failed to peel HEAD to commit: {}", e)))?;
        let head_tree = head_commit.tree()
            .map_err(|e| CommandError::Other(format!("Failed to get HEAD tree: {}", e)))?;
        let mut index = git_repo.index()
            .map_err(|e| CommandError::Other(format!("Failed to get index: {}", e)))?;
        let index_tree_oid = index.write_tree()
            .map_err(|e| CommandError::Other(format!("Failed to write index tree: {}", e)))?;
        let index_tree = git_repo.find_tree(index_tree_oid)
            .map_err(|e| CommandError::Other(format!("Failed to find index tree: {}", e)))?;
        git_repo.diff_tree_to_tree(Some(&head_tree), Some(&index_tree), Some(&mut diff_opts))
            .map_err(|e| CommandError::Other(format!("Failed to create staged diff: {}", e)))?
    } else if let Some(base) = &args.base {
        if let Some(target) = &args.target {
            // Commit-to-commit diff
            let base_obj = git_repo.revparse_single(base)
                .map_err(|e| CommandError::Other(format!("Failed to resolve base revision '{}': {}", base, e)))?;
            let target_obj = git_repo.revparse_single(target)
                .map_err(|e| CommandError::Other(format!("Failed to resolve target revision '{}': {}", target, e)))?;
            let base_tree = base_obj.peel_to_tree()
                .map_err(|e| CommandError::Other(format!("Failed to peel base to tree: {}", e)))?;
            let target_tree = target_obj.peel_to_tree()
                .map_err(|e| CommandError::Other(format!("Failed to peel target to tree: {}", e)))?;
            git_repo.diff_tree_to_tree(Some(&base_tree), Some(&target_tree), Some(&mut diff_opts))
                .map_err(|e| CommandError::Other(format!("Failed to create commit diff: {}", e)))?
        } else {
            // Base vs working tree
            let base_obj = git_repo.revparse_single(base)
                .map_err(|e| CommandError::Other(format!("Failed to resolve base revision '{}': {}", base, e)))?;
            let base_tree = base_obj.peel_to_tree()
                .map_err(|e| CommandError::Other(format!("Failed to peel base to tree: {}", e)))?;
            git_repo.diff_tree_to_workdir(Some(&base_tree), Some(&mut diff_opts))
                .map_err(|e| CommandError::Other(format!("Failed to create base vs workdir diff: {}", e)))?
        }
    } else {
        // Default: HEAD vs working tree
        let head_tree = git_repo.head()
            .and_then(|head| head.peel_to_tree())
            .ok();
        git_repo.diff_tree_to_workdir(head_tree.as_ref(), Some(&mut diff_opts))
            .map_err(|e| CommandError::Other(format!("Failed to create workdir diff: {}", e)))?
    };

    // Collect statistics and file information
    let mut stats = DiffStats::default();
    let mut changed_files = Vec::new();
    let mut has_changes = false;

    diff.foreach(
        &mut |delta, _progress| {
            let file_path = delta.new_file().path().unwrap_or_else(|| 
                delta.old_file().path().unwrap_or(std::path::Path::new("unknown"))
            );
            
            // Apply exclude patterns
            if exclude_patterns.iter().any(|pattern| pattern.matches_path(file_path)) {
                return true; // Skip this file
            }

            let path_str = file_path.to_string_lossy().to_string();
            if !changed_files.contains(&path_str) {
                changed_files.push(path_str);
                stats.files_changed += 1;
                has_changes = true;
            }
            true
        },
        None,
        None,
        Some(&mut |_delta, _hunk, line| {
            match line.origin() {
                '+' => stats.insertions += 1,
                '-' => stats.deletions += 1,
                _ => {}
            }
            true
        }),
    ).map_err(|e| CommandError::Other(format!("Failed to process diff: {}", e)))?;

    // Generate diff output based on the requested format
    let diff_output = if args.stat {
        generate_stat_output(&stats, &changed_files)
    } else if args.name_only {
        changed_files.join("\n")
    } else {
        generate_full_diff_output(&diff, !args.no_color)?
    };

    Ok(DiffResult {
        repository_name: repo_name,
        repository_path: repo_path.to_path_buf(),
        has_changes,
        diff_output,
        stats,
        changed_files,
    })
}

/// Generate statistics output
fn generate_stat_output(stats: &DiffStats, changed_files: &[String]) -> String {
    let mut output = String::new();
    
    for file in changed_files {
        output.push_str(file);
        output.push('\n');
    }
    
    if !changed_files.is_empty() {
        output.push('\n');
    }
    
    output.push_str(&format!(
        "{} file{} changed",
        stats.files_changed,
        if stats.files_changed == 1 { "" } else { "s" }
    ));
    
    if stats.insertions > 0 {
        output.push_str(&format!(", {} insertion{}", stats.insertions, if stats.insertions == 1 { "" } else { "s" }));
    }
    
    if stats.deletions > 0 {
        output.push_str(&format!(", {} deletion{}", stats.deletions, if stats.deletions == 1 { "" } else { "s" }));
    }
    
    output
}

/// Generate full diff output with optional coloring
fn generate_full_diff_output(diff: &git2::Diff, use_color: bool) -> CommandResult<String> {
    let mut output = String::new();
    
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        let line_str = std::str::from_utf8(line.content()).unwrap_or("<invalid UTF-8>");
        
        if use_color {
            match line.origin() {
                '+' => output.push_str(&format!("{}{}", "+".green(), line_str.green())),
                '-' => output.push_str(&format!("{}{}", "-".red(), line_str.red())),
                '@' => output.push_str(&format!("{}{}", "@".cyan(), line_str.cyan())),
                ' ' => output.push_str(&format!(" {}", line_str)),
                _ => output.push_str(line_str),
            }
        } else {
            match line.origin() {
                '+' | '-' | '@' | ' ' => output.push_str(&format!("{}{}", line.origin(), line_str)),
                _ => output.push_str(line_str),
            }
        }
        
        true
    }).map_err(|e| CommandError::Other(format!("Failed to generate diff output: {}", e)))?;
    
    Ok(output)
}

/// Format and display the diff output
fn format_diff_output(
    diff_results: &[DiffResult],
    args: &DiffArgs,
    formatter: &mut dyn OutputFormatter,
    total_stats: &DiffStats,
) -> CommandResult<()> {
    let use_color = !args.no_color;
    
    // Separate repositories with changes from those without
    let (changed_repos, unchanged_repos): (Vec<_>, Vec<_>) = diff_results.iter()
        .partition(|result| result.has_changes);
    
    // Display repositories with changes
    for (i, result) in changed_repos.iter().enumerate() {
        if i > 0 {
            formatter.info(""); // Add spacing between repositories
        }
        
        // Repository header
        let header = if use_color {
            format!("{} {}", "→".bold(), result.repository_name.bold().yellow())
        } else {
            format!("→ {}", result.repository_name)
        };
        formatter.warning(&header);
        
        // Display diff content
        if !result.diff_output.is_empty() {
            // Split output into lines and indent each line
            for line in result.diff_output.lines() {
                if line.starts_with("diff --git") || line.starts_with("index ") {
                    if use_color {
                        formatter.info(&format!("  {}", line.bright_white().bold()));
                    } else {
                        formatter.info(&format!("  {}", line));
                    }
                } else if line.starts_with("+++") || line.starts_with("---") {
                    if use_color {
                        formatter.info(&format!("  {}", line.bright_white()));
                    } else {
                        formatter.info(&format!("  {}", line));
                    }
                } else {
                    formatter.info(&format!("  {}", line));
                }
            }
        }
        
        // Display stats if in stat mode
        if args.stat && !result.changed_files.is_empty() {
            let stats_line = format!(
                "  {} file{} changed",
                result.stats.files_changed,
                if result.stats.files_changed == 1 { "" } else { "s" }
            );
            
            let mut full_stats = stats_line;
            if result.stats.insertions > 0 {
                full_stats.push_str(&format!(", {} insertion{}", result.stats.insertions, if result.stats.insertions == 1 { "" } else { "s" }));
            }
            if result.stats.deletions > 0 {
                full_stats.push_str(&format!(", {} deletion{}", result.stats.deletions, if result.stats.deletions == 1 { "" } else { "s" }));
            }
            
            if use_color {
                formatter.info(&full_stats.bright_white().to_string());
            } else {
                formatter.info(&full_stats);
            }
        }
    }
    
    // Display summary for unchanged repositories
    if !unchanged_repos.is_empty() {
        // Add spacing if there were changed repositories
        if !changed_repos.is_empty() {
            formatter.info("");
        }
        
        if unchanged_repos.len() == 1 {
            // Single unchanged repository - show it normally
            let result = unchanged_repos[0];
            let header = if use_color {
                format!("{} {}", "→".bold(), result.repository_name.bold().yellow())
            } else {
                format!("→ {}", result.repository_name)
            };
            formatter.warning(&header);
            formatter.success("  No changes");
        } else {
            // Multiple unchanged repositories - show concise summary
            let summary_message = format!("{} {} {} up to date",
                unchanged_repos.len(),
                if unchanged_repos.len() == 1 { "repository" } else { "repositories" },
                if unchanged_repos.len() == 1 { "is" } else { "are" }
            );
            
            if use_color {
                formatter.success(&summary_message.green().to_string());
            } else {
                formatter.success(&summary_message);
            }
        }
    }
    
    // Display total statistics if multiple repositories and there were changes
    if diff_results.len() > 1 && (args.stat || args.name_only) && total_stats.files_changed > 0 {
        formatter.info("");
        let total_line = format!(
            "Total: {} file{} changed",
            total_stats.files_changed,
            if total_stats.files_changed == 1 { "" } else { "s" }
        );
        
        let mut full_total = total_line;
        if total_stats.insertions > 0 {
            full_total.push_str(&format!(", {} insertion{}", total_stats.insertions, if total_stats.insertions == 1 { "" } else { "s" }));
        }
        if total_stats.deletions > 0 {
            full_total.push_str(&format!(", {} deletion{}", total_stats.deletions, if total_stats.deletions == 1 { "" } else { "s" }));
        }
        
        if use_color {
            formatter.success(&full_total.bold().to_string());
        } else {
            formatter.success(&full_total);
        }
    }
    
    Ok(())
}