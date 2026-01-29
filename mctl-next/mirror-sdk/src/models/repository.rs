use serde::{Deserialize, Serialize};

use crate::error::{MirrorError, Result};

/// Reference specification for a repository.
/// Only one of branch, tag, or rev can be specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefSpec {
    Branch(String),
    Tag(String),
    Rev(String),
}

impl Default for RefSpec {
    fn default() -> Self {
        RefSpec::Branch("main".to_string())
    }
}

impl RefSpec {
    /// Returns the reference string for git operations
    pub fn as_ref_str(&self) -> &str {
        match self {
            RefSpec::Branch(s) | RefSpec::Tag(s) | RefSpec::Rev(s) => s,
        }
    }

    /// Returns true if this is a branch reference
    pub fn is_branch(&self) -> bool {
        matches!(self, RefSpec::Branch(_))
    }

    /// Returns true if this is a tag reference
    pub fn is_tag(&self) -> bool {
        matches!(self, RefSpec::Tag(_))
    }

    /// Returns true if this is a revision reference
    pub fn is_rev(&self) -> bool {
        matches!(self, RefSpec::Rev(_))
    }
}

/// Raw repository data as parsed from TOML
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawRepository {
    pub git: String,
    pub path: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub tag: Option<String>,
    #[serde(default)]
    pub rev: Option<String>,
    #[serde(default)]
    pub workspaces: Vec<String>,
}

impl RawRepository {
    /// Convert to Repository with validated RefSpec
    pub fn into_repository(self) -> Result<Repository> {
        let ref_spec = match (self.branch, self.tag, self.rev) {
            (None, None, None) => RefSpec::default(),
            (Some(branch), None, None) => RefSpec::Branch(branch),
            (None, Some(tag), None) => RefSpec::Tag(tag),
            (None, None, Some(rev)) => RefSpec::Rev(rev),
            _ => return Err(MirrorError::InvalidRefSpec),
        };

        Ok(Repository {
            git: self.git,
            path: self.path,
            ref_spec,
            workspaces: self.workspaces,
        })
    }
}

/// Repository configuration
#[derive(Debug, Clone)]
pub struct Repository {
    pub git: String,
    pub path: String,
    pub ref_spec: RefSpec,
    pub workspaces: Vec<String>,
}

impl Repository {
    /// Convert to RawRepository for serialization
    pub fn to_raw(&self) -> RawRepository {
        let (branch, tag, rev) = match &self.ref_spec {
            RefSpec::Branch(b) if b == "main" => (None, None, None),
            RefSpec::Branch(b) => (Some(b.clone()), None, None),
            RefSpec::Tag(t) => (None, Some(t.clone()), None),
            RefSpec::Rev(r) => (None, None, Some(r.clone())),
        };

        RawRepository {
            git: self.git.clone(),
            path: self.path.clone(),
            branch,
            tag,
            rev,
            workspaces: self.workspaces.clone(),
        }
    }

    /// Check if repository belongs to given workspace
    pub fn in_workspace(&self, workspace: &str) -> bool {
        self.workspaces.iter().any(|w| w == workspace)
    }

    /// Extract repository name from git URL
    pub fn name(&self) -> &str {
        self.git
            .rsplit('/')
            .next()
            .and_then(|s| s.strip_suffix(".git"))
            .unwrap_or(&self.git)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refspec_default() {
        assert_eq!(RefSpec::default(), RefSpec::Branch("main".to_string()));
    }

    #[test]
    fn test_raw_repository_conversion() {
        let raw = RawRepository {
            git: "git@github.com:test/repo.git".to_string(),
            path: "test/repo".to_string(),
            branch: Some("develop".to_string()),
            tag: None,
            rev: None,
            workspaces: vec!["ws1".to_string()],
        };

        let repo = raw.into_repository().unwrap();
        assert_eq!(repo.ref_spec, RefSpec::Branch("develop".to_string()));
    }

    #[test]
    fn test_invalid_refspec() {
        let raw = RawRepository {
            git: "git@github.com:test/repo.git".to_string(),
            path: "test/repo".to_string(),
            branch: Some("main".to_string()),
            tag: Some("v1.0".to_string()),
            rev: None,
            workspaces: vec![],
        };

        assert!(raw.into_repository().is_err());
    }

    #[test]
    fn test_repository_name() {
        let repo = Repository {
            git: "git@github.com:test/my-repo.git".to_string(),
            path: "test/repo".to_string(),
            ref_spec: RefSpec::default(),
            workspaces: vec![],
        };

        assert_eq!(repo.name(), "my-repo");
    }
}
