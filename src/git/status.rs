//! Git status types and parsing.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Status of a file in the working directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatusCode {
    /// File is untracked
    Untracked,
    /// File is ignored
    Ignored,
    /// File is modified
    Modified,
    /// File is added/new
    Added,
    /// File is deleted
    Deleted,
    /// File is renamed
    Renamed,
    /// File is copied
    Copied,
    /// File has been updated but unmerged (conflict)
    Unmerged,
    /// File type changed (e.g., regular file to symlink)
    TypeChanged,
}

impl fmt::Display for FileStatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            FileStatusCode::Untracked => "?",
            FileStatusCode::Ignored => "!",
            FileStatusCode::Modified => "M",
            FileStatusCode::Added => "A",
            FileStatusCode::Deleted => "D",
            FileStatusCode::Renamed => "R",
            FileStatusCode::Copied => "C",
            FileStatusCode::Unmerged => "U",
            FileStatusCode::TypeChanged => "T",
        };
        write!(f, "{}", code)
    }
}

/// Status of a single file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileStatus {
    /// Path of the file
    pub path: String,
    /// Status in the index (staged)
    pub index_status: Option<FileStatusCode>,
    /// Status in the working tree
    pub worktree_status: Option<FileStatusCode>,
    /// Original path (for renames/copies)
    pub original_path: Option<String>,
}

impl FileStatus {
    /// Check if the file is staged (in index).
    pub fn is_staged(&self) -> bool {
        self.index_status.is_some()
    }

    /// Check if the file has unstaged changes.
    pub fn is_unstaged(&self) -> bool {
        self.worktree_status.is_some()
    }

    /// Check if the file is untracked.
    pub fn is_untracked(&self) -> bool {
        self.worktree_status == Some(FileStatusCode::Untracked)
    }

    /// Check if the file has a conflict.
    pub fn is_conflicted(&self) -> bool {
        self.index_status == Some(FileStatusCode::Unmerged)
            || self.worktree_status == Some(FileStatusCode::Unmerged)
    }
}

impl fmt::Display for FileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let index = self
            .index_status
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| " ".to_string());
        let worktree = self
            .worktree_status
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| " ".to_string());

        if let Some(orig) = &self.original_path {
            write!(f, "{}{} {} -> {}", index, worktree, orig, self.path)
        } else {
            write!(f, "{}{} {}", index, worktree, self.path)
        }
    }
}

/// Branch tracking information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchInfo {
    /// Current branch name (or HEAD for detached)
    pub name: String,
    /// Upstream branch name
    pub upstream: Option<String>,
    /// Number of commits ahead of upstream
    pub ahead: u32,
    /// Number of commits behind upstream
    pub behind: u32,
}

impl BranchInfo {
    /// Check if the branch is in sync with upstream.
    pub fn is_synced(&self) -> bool {
        self.upstream.is_some() && self.ahead == 0 && self.behind == 0
    }

    /// Check if this is a detached HEAD state.
    pub fn is_detached(&self) -> bool {
        self.name == "HEAD" || self.name.starts_with("(HEAD detached")
    }
}

/// Overall status of a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryStatus {
    /// Branch information
    pub branch: BranchInfo,
    /// List of file statuses
    pub files: Vec<FileStatus>,
    /// Current HEAD commit hash (short)
    pub head_short: String,
    /// Current HEAD commit hash (full)
    pub head_full: String,
}

impl RepositoryStatus {
    /// Check if the repository is clean (no changes).
    pub fn is_clean(&self) -> bool {
        self.files.is_empty()
    }

    /// Get all staged files.
    pub fn staged_files(&self) -> Vec<&FileStatus> {
        self.files.iter().filter(|f| f.is_staged()).collect()
    }

    /// Get all unstaged files.
    pub fn unstaged_files(&self) -> Vec<&FileStatus> {
        self.files.iter().filter(|f| f.is_unstaged()).collect()
    }

    /// Get all untracked files.
    pub fn untracked_files(&self) -> Vec<&FileStatus> {
        self.files.iter().filter(|f| f.is_untracked()).collect()
    }

    /// Get all conflicted files.
    pub fn conflicted_files(&self) -> Vec<&FileStatus> {
        self.files.iter().filter(|f| f.is_conflicted()).collect()
    }

    /// Check if there are any changes to commit.
    pub fn has_changes_to_commit(&self) -> bool {
        !self.staged_files().is_empty()
    }

    /// Check if there are any uncommitted changes.
    pub fn has_uncommitted_changes(&self) -> bool {
        !self.files.is_empty()
    }

    /// Check if there are merge conflicts.
    pub fn has_conflicts(&self) -> bool {
        !self.conflicted_files().is_empty()
    }

    /// Check if there are unpushed commits.
    pub fn has_unpushed_commits(&self) -> bool {
        self.branch.upstream.is_some() && self.branch.ahead > 0
    }

    /// Check if the repository is fully synced (clean and no unpushed commits).
    pub fn is_fully_synced(&self) -> bool {
        self.is_clean() && !self.has_unpushed_commits()
    }

    /// Get a summary string.
    pub fn summary(&self) -> String {
        if self.is_clean() {
            if self.branch.is_synced() {
                "Clean, up to date".to_string()
            } else if self.branch.upstream.is_some() {
                format!(
                    "Clean (+{} -{} from upstream)",
                    self.branch.ahead, self.branch.behind
                )
            } else {
                "Clean, no upstream".to_string()
            }
        } else {
            let staged = self.staged_files().len();
            let unstaged = self.unstaged_files().len();
            let untracked = self.untracked_files().len();

            let mut parts = Vec::new();
            if staged > 0 {
                parts.push(format!("{} staged", staged));
            }
            if unstaged > 0 {
                parts.push(format!("{} modified", unstaged));
            }
            if untracked > 0 {
                parts.push(format!("{} untracked", untracked));
            }

            parts.join(", ")
        }
    }
}

/// Sync status of a repository compared to expected version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    /// Repository is synced with expected version
    Synced,
    /// Repository is ahead of expected version
    Ahead(u32),
    /// Repository is behind expected version
    Behind(u32),
    /// Repository has diverged from expected version
    Diverged { ahead: u32, behind: u32 },
    /// Repository is not cloned yet
    NotCloned,
    /// Repository has local changes
    Dirty,
    /// Cannot determine sync status
    Unknown,
}

impl fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncStatus::Synced => write!(f, "✓ Synced"),
            SyncStatus::Ahead(n) => write!(f, "↑ Ahead by {} commit(s)", n),
            SyncStatus::Behind(n) => write!(f, "↓ Behind by {} commit(s)", n),
            SyncStatus::Diverged { ahead, behind } => {
                write!(f, "⇅ Diverged (+{} -{})", ahead, behind)
            }
            SyncStatus::NotCloned => write!(f, "✗ Not cloned"),
            SyncStatus::Dirty => write!(f, "● Dirty (uncommitted changes)"),
            SyncStatus::Unknown => write!(f, "? Unknown"),
        }
    }
}
