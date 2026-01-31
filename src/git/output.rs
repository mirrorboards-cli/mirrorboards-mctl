//! Parsers for git command output.

use crate::git::status::{BranchInfo, FileStatus, FileStatusCode};
use regex::Regex;
use std::sync::LazyLock;

// Regex patterns for porcelain v2 output
static BRANCH_OID_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^# branch\.oid (.+)$").unwrap());

static BRANCH_HEAD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^# branch\.head (.+)$").unwrap());

static BRANCH_UPSTREAM_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^# branch\.upstream (.+)$").unwrap());

static BRANCH_AB_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^# branch\.ab \+(\d+) -(\d+)$").unwrap());

// Ordinary changed entry: 1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>
static ORDINARY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^1 ([A-Z.])([A-Z.]) .+ .+ .+ .+ .+ (.+)$").unwrap());

// Renamed/copied entry: 2 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <X><score> <path><sep><origPath>
static RENAMED_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^2 ([A-Z.])([A-Z.]) .+ .+ .+ .+ .+ [RC]\d+ (.+)\t(.+)$").unwrap()
});

// Unmerged entry: u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>
static UNMERGED_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^u ([A-Z.])([A-Z.]) .+ .+ .+ .+ .+ .+ .+ .+ (.+)$").unwrap());

// Untracked entry: ? <path>
static UNTRACKED_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^\? (.+)$").unwrap());

// Ignored entry: ! <path>
static IGNORED_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^! (.+)$").unwrap());

/// Parse a status code character into FileStatusCode.
fn parse_status_code(c: char) -> Option<FileStatusCode> {
    match c {
        'M' => Some(FileStatusCode::Modified),
        'A' => Some(FileStatusCode::Added),
        'D' => Some(FileStatusCode::Deleted),
        'R' => Some(FileStatusCode::Renamed),
        'C' => Some(FileStatusCode::Copied),
        'T' => Some(FileStatusCode::TypeChanged),
        'U' => Some(FileStatusCode::Unmerged),
        '.' | ' ' => None,
        _ => None,
    }
}

/// Parse git status --porcelain=v2 --branch output.
///
/// Returns (BranchInfo, Vec<FileStatus>).
pub fn parse_status_porcelain_v2(output: &str) -> (Option<BranchInfo>, Vec<FileStatus>) {
    let mut branch_head = String::new();
    let mut branch_upstream = None;
    let mut ahead = 0u32;
    let mut behind = 0u32;
    let mut files = Vec::new();

    for line in output.lines() {
        // Branch info lines
        if BRANCH_OID_PATTERN.is_match(line) {
            // Skip OID line, we get HEAD rev separately
            continue;
        }
        if let Some(caps) = BRANCH_HEAD_PATTERN.captures(line) {
            branch_head = caps[1].to_string();
            continue;
        }
        if let Some(caps) = BRANCH_UPSTREAM_PATTERN.captures(line) {
            branch_upstream = Some(caps[1].to_string());
            continue;
        }
        if let Some(caps) = BRANCH_AB_PATTERN.captures(line) {
            ahead = caps[1].parse().unwrap_or(0);
            behind = caps[2].parse().unwrap_or(0);
            continue;
        }

        // File status lines
        if let Some(caps) = ORDINARY_PATTERN.captures(line) {
            let index_char = caps[1].chars().next().unwrap_or('.');
            let worktree_char = caps[2].chars().next().unwrap_or('.');
            let path = caps[3].to_string();

            files.push(FileStatus {
                path,
                index_status: parse_status_code(index_char),
                worktree_status: parse_status_code(worktree_char),
                original_path: None,
            });
            continue;
        }

        if let Some(caps) = RENAMED_PATTERN.captures(line) {
            let index_char = caps[1].chars().next().unwrap_or('.');
            let worktree_char = caps[2].chars().next().unwrap_or('.');
            let path = caps[3].to_string();
            let orig_path = caps[4].to_string();

            files.push(FileStatus {
                path,
                index_status: parse_status_code(index_char),
                worktree_status: parse_status_code(worktree_char),
                original_path: Some(orig_path),
            });
            continue;
        }

        if let Some(caps) = UNMERGED_PATTERN.captures(line) {
            let path = caps[3].to_string();

            files.push(FileStatus {
                path,
                index_status: Some(FileStatusCode::Unmerged),
                worktree_status: Some(FileStatusCode::Unmerged),
                original_path: None,
            });
            continue;
        }

        if let Some(caps) = UNTRACKED_PATTERN.captures(line) {
            let path = caps[1].to_string();

            files.push(FileStatus {
                path,
                index_status: None,
                worktree_status: Some(FileStatusCode::Untracked),
                original_path: None,
            });
            continue;
        }

        if let Some(caps) = IGNORED_PATTERN.captures(line) {
            let path = caps[1].to_string();

            files.push(FileStatus {
                path,
                index_status: None,
                worktree_status: Some(FileStatusCode::Ignored),
                original_path: None,
            });
            continue;
        }
    }

    let branch_info = if !branch_head.is_empty() {
        Some(BranchInfo {
            name: branch_head,
            upstream: branch_upstream,
            ahead,
            behind,
        })
    } else {
        None
    };

    (branch_info, files)
}

/// Parse git rev-parse output (commit hash).
pub fn parse_rev(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Parse git branch --show-current or rev-parse --abbrev-ref HEAD output.
pub fn parse_branch_name(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Parse git remote get-url output.
pub fn parse_remote_url(output: &str) -> Option<String> {
    let trimmed = output.trim();
    if !trimmed.is_empty() {
        Some(trimmed.to_string())
    } else {
        None
    }
}

/// Check if git output indicates authentication failure.
pub fn is_auth_error(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("permission denied")
        || lower.contains("authentication failed")
        || lower.contains("could not read from remote")
        || lower.contains("host key verification failed")
        || lower.contains("access denied")
}

/// Check if git output indicates network error.
pub fn is_network_error(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("could not resolve host")
        || lower.contains("connection refused")
        || lower.contains("network is unreachable")
        || lower.contains("connection timed out")
        || lower.contains("ssl")
}

/// Check if git output indicates the repository was not found.
pub fn is_repo_not_found(stderr: &str) -> bool {
    let lower = stderr.to_lowercase();
    lower.contains("repository not found")
        || lower.contains("does not exist")
        || lower.contains("not a git repository")
        || lower.contains("fatal: '") && lower.contains("' does not appear to be a git repository")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_clean() {
        let output = "# branch.oid abc123def456\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +0 -0\n";
        let (branch, files) = parse_status_porcelain_v2(output);

        assert!(branch.is_some());
        let branch = branch.unwrap();
        assert_eq!(branch.name, "main");
        assert_eq!(branch.upstream, Some("origin/main".to_string()));
        assert_eq!(branch.ahead, 0);
        assert_eq!(branch.behind, 0);
        assert!(files.is_empty());
    }

    #[test]
    fn test_parse_status_with_changes() {
        let output = r#"# branch.oid abc123
# branch.head main
1 M. N... 100644 100644 100644 abc123 def456 modified.txt
1 .M N... 100644 100644 100644 abc123 def456 unstaged.txt
? untracked.txt
"#;
        let (branch, files) = parse_status_porcelain_v2(output);

        assert!(branch.is_some());
        assert_eq!(files.len(), 3);

        // Modified in index
        assert_eq!(files[0].path, "modified.txt");
        assert_eq!(files[0].index_status, Some(FileStatusCode::Modified));
        assert_eq!(files[0].worktree_status, None);

        // Modified in worktree
        assert_eq!(files[1].path, "unstaged.txt");
        assert_eq!(files[1].index_status, None);
        assert_eq!(files[1].worktree_status, Some(FileStatusCode::Modified));

        // Untracked
        assert_eq!(files[2].path, "untracked.txt");
        assert_eq!(files[2].worktree_status, Some(FileStatusCode::Untracked));
    }

    #[test]
    fn test_parse_rev() {
        assert_eq!(
            parse_rev("abc123def456\n"),
            Some("abc123def456".to_string())
        );
        assert_eq!(parse_rev("  abc123  \n"), Some("abc123".to_string()));
        assert_eq!(parse_rev(""), None);
        assert_eq!(parse_rev("not a hash!"), None);
    }

    #[test]
    fn test_is_auth_error() {
        assert!(is_auth_error("Permission denied (publickey)"));
        assert!(is_auth_error("fatal: Authentication failed for"));
        assert!(!is_auth_error("Already up to date."));
    }

    #[test]
    fn test_is_network_error() {
        assert!(is_network_error("Could not resolve host: github.com"));
        assert!(is_network_error("Connection refused"));
        assert!(!is_network_error("Everything up-to-date"));
    }
}
