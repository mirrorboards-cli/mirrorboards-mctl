//! Repository model and version specification.

use serde::{Deserialize, Serialize};

/// Version specification for a repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionSpec {
    /// Use repository's default branch
    DefaultBranch,
    /// Track a specific branch
    Branch(String),
    /// Specific commit hash
    Rev(String),
    /// Specific tag
    Tag(String),
}

impl Default for VersionSpec {
    fn default() -> Self {
        VersionSpec::DefaultBranch
    }
}

impl std::fmt::Display for VersionSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionSpec::DefaultBranch => write!(f, "default"),
            VersionSpec::Branch(b) => write!(f, "branch:{}", b),
            VersionSpec::Rev(r) => write!(f, "rev:{}", r),
            VersionSpec::Tag(t) => write!(f, "tag:{}", t),
        }
    }
}

/// Helper function for serde skip_serializing_if
fn is_false(value: &bool) -> bool {
    !*value
}

/// A repository configuration entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    /// Git URL of the repository
    pub git: String,

    /// Local path where the repository will be cloned
    pub path: String,

    /// Branch to track (mutually exclusive with rev and tag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Specific revision/commit hash (mutually exclusive with branch and tag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,

    /// Specific tag (mutually exclusive with branch and rev)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Skip push operations for this repository
    #[serde(default, skip_serializing_if = "is_false", rename = "skip-push")]
    pub skip_push: bool,

    /// Workspaces this repository belongs to
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workspaces: Vec<String>,
}

impl Repository {
    /// Create a new repository with default settings.
    pub fn new(git: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            git: git.into(),
            path: path.into(),
            branch: None,
            rev: None,
            tag: None,
            skip_push: false,
            workspaces: Vec::new(),
        }
    }

    /// Set the branch for this repository.
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self.rev = None;
        self.tag = None;
        self
    }

    /// Set a specific revision for this repository.
    pub fn with_rev(mut self, rev: impl Into<String>) -> Self {
        self.rev = Some(rev.into());
        self.branch = None;
        self.tag = None;
        self
    }

    /// Set a specific tag for this repository.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self.branch = None;
        self.rev = None;
        self
    }

    /// Set skip-push flag.
    pub fn with_skip_push(mut self, skip: bool) -> Self {
        self.skip_push = skip;
        self
    }

    /// Add workspaces.
    pub fn with_workspaces(mut self, workspaces: Vec<String>) -> Self {
        self.workspaces = workspaces;
        self
    }

    /// Get the version specification for this repository.
    pub fn version_spec(&self) -> VersionSpec {
        if let Some(rev) = &self.rev {
            VersionSpec::Rev(rev.clone())
        } else if let Some(tag) = &self.tag {
            VersionSpec::Tag(tag.clone())
        } else if let Some(branch) = &self.branch {
            VersionSpec::Branch(branch.clone())
        } else {
            VersionSpec::DefaultBranch
        }
    }

    /// Check if this repository is in a given workspace.
    pub fn is_in_workspace(&self, workspace: &str) -> bool {
        self.workspaces.iter().any(|w| w == workspace)
    }

    /// Validate the repository configuration.
    pub fn validate(&self) -> Result<(), String> {
        // Check that only one version spec is set
        let specs_count = [
            self.branch.is_some(),
            self.rev.is_some(),
            self.tag.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        if specs_count > 1 {
            return Err(
                "Repository can only have one of: branch, rev, or tag".to_string()
            );
        }

        // Validate git URL
        if self.git.is_empty() {
            return Err("Git URL cannot be empty".to_string());
        }

        // Validate path
        if self.path.is_empty() {
            return Err("Path cannot be empty".to_string());
        }

        Ok(())
    }

    /// Get the repository name from the git URL.
    pub fn name(&self) -> String {
        crate::core::url::parse_repo_name(&self.git).unwrap_or_else(|| self.path.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_spec_default() {
        let repo = Repository::new("git@github.com:test/repo.git", "test/repo");
        assert_eq!(repo.version_spec(), VersionSpec::DefaultBranch);
    }

    #[test]
    fn test_version_spec_branch() {
        let repo = Repository::new("git@github.com:test/repo.git", "test/repo")
            .with_branch("develop");
        assert_eq!(repo.version_spec(), VersionSpec::Branch("develop".to_string()));
    }

    #[test]
    fn test_version_spec_rev() {
        let repo = Repository::new("git@github.com:test/repo.git", "test/repo")
            .with_rev("abc123");
        assert_eq!(repo.version_spec(), VersionSpec::Rev("abc123".to_string()));
    }

    #[test]
    fn test_version_spec_tag() {
        let repo = Repository::new("git@github.com:test/repo.git", "test/repo")
            .with_tag("v1.0.0");
        assert_eq!(repo.version_spec(), VersionSpec::Tag("v1.0.0".to_string()));
    }

    #[test]
    fn test_workspace_membership() {
        let repo = Repository::new("git@github.com:test/repo.git", "test/repo")
            .with_workspaces(vec!["api".to_string(), "core".to_string()]);

        assert!(repo.is_in_workspace("api"));
        assert!(repo.is_in_workspace("core"));
        assert!(!repo.is_in_workspace("frontend"));
    }

    #[test]
    fn test_serialization() {
        let repo = Repository::new("git@github.com:test/repo.git", "test/repo")
            .with_branch("main")
            .with_workspaces(vec!["api".to_string()]);

        let toml = toml::to_string(&repo).unwrap();
        assert!(toml.contains("git = "));
        assert!(toml.contains("path = "));
    }
}
