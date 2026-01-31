//! Git URL parsing utilities.

use crate::core::error::{UrlError, UrlResult};
use regex::Regex;
use std::sync::LazyLock;

/// Parsed git URL components.
#[derive(Debug, Clone, PartialEq)]
pub struct GitUrl {
    /// The protocol (ssh, https, git, file)
    pub protocol: Protocol,
    /// The host (e.g., github.com)
    pub host: String,
    /// The owner/organization
    pub owner: String,
    /// The repository name (without .git)
    pub repo: String,
    /// The original URL
    pub original: String,
}

/// Git URL protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    Ssh,
    Https,
    Git,
    File,
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Ssh => write!(f, "ssh"),
            Protocol::Https => write!(f, "https"),
            Protocol::Git => write!(f, "git"),
            Protocol::File => write!(f, "file"),
        }
    }
}

// Regex patterns for different URL formats
static SSH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:ssh://)?(?:[^@]+@)?([^:/]+)[:/]([^/]+)/(.+?)(?:\.git)?$").unwrap()
});

static HTTPS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^https?://([^/]+)/([^/]+)/(.+?)(?:\.git)?$").unwrap()
});

static GIT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^git://([^/]+)/([^/]+)/(.+?)(?:\.git)?$").unwrap()
});

impl GitUrl {
    /// Parse a git URL string into its components.
    pub fn parse(url: &str) -> UrlResult<Self> {
        let url = url.trim();

        // Try SSH format (git@host:owner/repo.git or ssh://...)
        if let Some(caps) = SSH_PATTERN.captures(url) {
            return Ok(GitUrl {
                protocol: Protocol::Ssh,
                host: caps[1].to_string(),
                owner: caps[2].to_string(),
                repo: caps[3].trim_end_matches(".git").to_string(),
                original: url.to_string(),
            });
        }

        // Try HTTPS format
        if let Some(caps) = HTTPS_PATTERN.captures(url) {
            return Ok(GitUrl {
                protocol: Protocol::Https,
                host: caps[1].to_string(),
                owner: caps[2].to_string(),
                repo: caps[3].trim_end_matches(".git").to_string(),
                original: url.to_string(),
            });
        }

        // Try git:// format
        if let Some(caps) = GIT_PATTERN.captures(url) {
            return Ok(GitUrl {
                protocol: Protocol::Git,
                host: caps[1].to_string(),
                owner: caps[2].to_string(),
                repo: caps[3].trim_end_matches(".git").to_string(),
                original: url.to_string(),
            });
        }

        Err(UrlError::InvalidUrl {
            url: url.to_string(),
        })
    }

    /// Get the full repository path (owner/repo).
    pub fn full_path(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    /// Convert to SSH URL format.
    pub fn to_ssh(&self) -> String {
        format!("git@{}:{}/{}.git", self.host, self.owner, self.repo)
    }

    /// Convert to HTTPS URL format.
    pub fn to_https(&self) -> String {
        format!("https://{}/{}/{}.git", self.host, self.owner, self.repo)
    }
}

/// Parse the repository name from a git URL.
///
/// Returns the repository name without the .git extension.
pub fn parse_repo_name(url: &str) -> Option<String> {
    GitUrl::parse(url).ok().map(|u| u.repo)
}

/// Parse the full path (owner/repo) from a git URL.
pub fn parse_full_path(url: &str) -> Option<String> {
    GitUrl::parse(url).ok().map(|u| u.full_path())
}

/// Suggest a local path for a repository based on its URL.
///
/// For example, `git@github.com:owner/repo.git` -> `owner/repo`
pub fn suggest_path(url: &str) -> Option<String> {
    parse_full_path(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_url() {
        let url = GitUrl::parse("git@github.com:owner/repo.git").unwrap();
        assert_eq!(url.protocol, Protocol::Ssh);
        assert_eq!(url.host, "github.com");
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
    }

    #[test]
    fn test_parse_ssh_url_without_git_suffix() {
        let url = GitUrl::parse("git@github.com:owner/repo").unwrap();
        assert_eq!(url.repo, "repo");
    }

    #[test]
    fn test_parse_https_url() {
        let url = GitUrl::parse("https://github.com/owner/repo.git").unwrap();
        assert_eq!(url.protocol, Protocol::Https);
        assert_eq!(url.host, "github.com");
        assert_eq!(url.owner, "owner");
        assert_eq!(url.repo, "repo");
    }

    #[test]
    fn test_parse_repo_name() {
        assert_eq!(
            parse_repo_name("git@github.com:owner/my-repo.git"),
            Some("my-repo".to_string())
        );
    }

    #[test]
    fn test_suggest_path() {
        assert_eq!(
            suggest_path("git@github.com:owner/repo.git"),
            Some("owner/repo".to_string())
        );
    }

    #[test]
    fn test_url_conversion() {
        let url = GitUrl::parse("git@github.com:owner/repo.git").unwrap();
        assert_eq!(url.to_ssh(), "git@github.com:owner/repo.git");
        assert_eq!(url.to_https(), "https://github.com/owner/repo.git");
    }

    #[test]
    fn test_invalid_url() {
        assert!(GitUrl::parse("not a valid url").is_err());
    }
}
