use crate::error::{RepositoryError, RepositoryResult};
use regex::Regex;
use url::Url;
use std::sync::OnceLock;

/// Extract the organization/repository path from a git URL
pub fn extract_path_from_url(git_url: &str) -> RepositoryResult<String> {
    let trimmed_url = git_url.trim();
    
    if trimmed_url.is_empty() {
        return Err(RepositoryError::InvalidUrl {
            url: git_url.to_string(),
        });
    }
    
    // Try SSH format first
    if let Ok(path) = extract_ssh_path(trimmed_url) {
        return Ok(path);
    }
    
    // Try HTTPS format
    if let Ok(path) = extract_https_path(trimmed_url) {
        return Ok(path);
    }
    
    // If neither format works, return an error
    Err(RepositoryError::PathExtractionFailed {
        url: git_url.to_string(),
    })
}

/// Extract path from SSH format: git@host:org/repo.git
fn extract_ssh_path(git_url: &str) -> RepositoryResult<String> {
    static SSH_REGEX: OnceLock<Regex> = OnceLock::new();
    let regex = SSH_REGEX.get_or_init(|| {
        Regex::new(r"^[^@]+@[^:]+:(.+?)(?:\.git)?/?$").unwrap()
    });
    
    if let Some(captures) = regex.captures(git_url) {
        let path = captures.get(1).unwrap().as_str();
        let normalized_path = normalize_path(path);
        
        if normalized_path.is_empty() {
            return Err(RepositoryError::PathExtractionFailed {
                url: git_url.to_string(),
            });
        }
        
        Ok(normalized_path)
    } else {
        Err(RepositoryError::PathExtractionFailed {
            url: git_url.to_string(),
        })
    }
}

/// Extract path from HTTPS format: https://host/org/repo.git
fn extract_https_path(git_url: &str) -> RepositoryResult<String> {
    let parsed_url = Url::parse(git_url).map_err(|_| RepositoryError::InvalidUrl {
        url: git_url.to_string(),
    })?;
    
    // Validate scheme
    match parsed_url.scheme() {
        "https" | "http" => {},
        scheme => return Err(RepositoryError::UnsupportedScheme {
            scheme: scheme.to_string(),
        }),
    }
    
    let path = parsed_url.path();
    
    // Remove leading slash and .git suffix
    let cleaned_path = path
        .strip_prefix('/')
        .unwrap_or(path)
        .strip_suffix(".git")
        .unwrap_or(path.strip_prefix('/').unwrap_or(path));
    
    let normalized_path = normalize_path(cleaned_path);
    
    if normalized_path.is_empty() {
        return Err(RepositoryError::PathExtractionFailed {
            url: git_url.to_string(),
        });
    }
    
    Ok(normalized_path)
}

/// Normalize a path by removing extra slashes and empty segments
fn normalize_path(path: &str) -> String {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}

/// Validate that a git URL is in a supported format
pub fn validate_git_url(git_url: &str) -> RepositoryResult<()> {
    let trimmed_url = git_url.trim();
    
    if trimmed_url.is_empty() {
        return Err(RepositoryError::InvalidUrl {
            url: git_url.to_string(),
        });
    }
    
    // Check if it's a valid SSH or HTTPS URL
    if is_ssh_url(trimmed_url) || is_https_url(trimmed_url) {
        // Try to extract path to ensure the URL is parseable
        extract_path_from_url(trimmed_url)?;
        Ok(())
    } else {
        Err(RepositoryError::InvalidUrl {
            url: git_url.to_string(),
        })
    }
}

/// Check if URL is in SSH format
fn is_ssh_url(url: &str) -> bool {
    static SSH_PATTERN: OnceLock<Regex> = OnceLock::new();
    let regex = SSH_PATTERN.get_or_init(|| {
        Regex::new(r"^[^@]+@[^:]+:.+").unwrap()
    });
    regex.is_match(url)
}

/// Check if URL is in HTTPS format
fn is_https_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

/// Get the hostname from a git URL
pub fn extract_hostname(git_url: &str) -> RepositoryResult<String> {
    let trimmed_url = git_url.trim();
    
    if is_ssh_url(trimmed_url) {
        static SSH_HOST_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = SSH_HOST_REGEX.get_or_init(|| {
            Regex::new(r"^[^@]+@([^:]+):").unwrap()
        });
        
        if let Some(captures) = regex.captures(trimmed_url) {
            Ok(captures.get(1).unwrap().as_str().to_string())
        } else {
            Err(RepositoryError::InvalidUrl {
                url: git_url.to_string(),
            })
        }
    } else if is_https_url(trimmed_url) {
        let parsed_url = Url::parse(trimmed_url).map_err(|_| RepositoryError::InvalidUrl {
            url: git_url.to_string(),
        })?;
        
        Ok(parsed_url.host_str().unwrap_or("").to_string())
    } else {
        Err(RepositoryError::InvalidUrl {
            url: git_url.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_ssh_path() {
        // Standard GitHub SSH format
        assert_eq!(
            extract_path_from_url("git@github.com:org/repo.git").unwrap(),
            "org/repo"
        );
        
        // SSH without .git suffix
        assert_eq!(
            extract_path_from_url("git@github.com:org/repo").unwrap(),
            "org/repo"
        );
        
        // SSH with trailing slash
        assert_eq!(
            extract_path_from_url("git@github.com:org/repo/").unwrap(),
            "org/repo"
        );
        
        // SSH with nested path
        assert_eq!(
            extract_path_from_url("git@gitlab.com:group/subgroup/repo.git").unwrap(),
            "group/subgroup/repo"
        );
        
        // Custom host
        assert_eq!(
            extract_path_from_url("git@git.example.com:org/repo.git").unwrap(),
            "org/repo"
        );
    }
    
    #[test]
    fn test_extract_https_path() {
        // Standard GitHub HTTPS format
        assert_eq!(
            extract_path_from_url("https://github.com/org/repo.git").unwrap(),
            "org/repo"
        );
        
        // HTTPS without .git suffix
        assert_eq!(
            extract_path_from_url("https://github.com/org/repo").unwrap(),
            "org/repo"
        );
        
        // HTTPS with trailing slash
        assert_eq!(
            extract_path_from_url("https://github.com/org/repo/").unwrap(),
            "org/repo"
        );
        
        // HTTPS with nested path
        assert_eq!(
            extract_path_from_url("https://gitlab.com/group/subgroup/repo.git").unwrap(),
            "group/subgroup/repo"
        );
        
        // Custom host
        assert_eq!(
            extract_path_from_url("https://git.example.com/org/repo.git").unwrap(),
            "org/repo"
        );
        
        // HTTP (less secure but should work)
        assert_eq!(
            extract_path_from_url("http://git.example.com/org/repo.git").unwrap(),
            "org/repo"
        );
    }
    
    #[test]
    fn test_invalid_urls() {
        // Empty URL
        assert!(extract_path_from_url("").is_err());
        
        // Invalid SSH format
        assert!(extract_path_from_url("git@github.com").is_err());
        
        // Invalid HTTPS format
        assert!(extract_path_from_url("https://github.com/").is_err());
        
        // Unsupported scheme
        assert!(extract_path_from_url("ftp://example.com/repo.git").is_err());
        
        // Malformed URL
        assert!(extract_path_from_url("not-a-url").is_err());
    }
    
    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("org/repo"), "org/repo");
        assert_eq!(normalize_path("org//repo"), "org/repo");
        assert_eq!(normalize_path("/org/repo/"), "org/repo");
        assert_eq!(normalize_path("group/subgroup/repo"), "group/subgroup/repo");
        assert_eq!(normalize_path("///org///repo///"), "org/repo");
    }
    
    #[test]
    fn test_validate_git_url() {
        // Valid URLs
        assert!(validate_git_url("git@github.com:org/repo.git").is_ok());
        assert!(validate_git_url("https://github.com/org/repo.git").is_ok());
        
        // Invalid URLs
        assert!(validate_git_url("").is_err());
        assert!(validate_git_url("not-a-url").is_err());
        assert!(validate_git_url("ftp://example.com/repo.git").is_err());
    }
    
    #[test]
    fn test_extract_hostname() {
        assert_eq!(
            extract_hostname("git@github.com:org/repo.git").unwrap(),
            "github.com"
        );
        assert_eq!(
            extract_hostname("https://github.com/org/repo.git").unwrap(),
            "github.com"
        );
        assert_eq!(
            extract_hostname("git@git.example.com:org/repo.git").unwrap(),
            "git.example.com"
        );
        
        // Invalid URLs should fail
        assert!(extract_hostname("not-a-url").is_err());
    }
    
    #[test]
    fn test_url_format_detection() {
        assert!(is_ssh_url("git@github.com:org/repo.git"));
        assert!(!is_ssh_url("https://github.com/org/repo.git"));
        
        assert!(is_https_url("https://github.com/org/repo.git"));
        assert!(is_https_url("http://git.example.com/org/repo.git"));
        assert!(!is_https_url("git@github.com:org/repo.git"));
    }
    
    #[test]
    fn test_organization_specific_ssh_urls() {
        // Test organization-specific SSH URLs with various username formats
        assert_eq!(
            extract_path_from_url("org-25111032@github.com:smartcontractkit/chainlink.git").unwrap(),
            "smartcontractkit/chainlink"
        );
        
        assert_eq!(
            extract_path_from_url("deploy-key-123@gitlab.com:myorg/myrepo.git").unwrap(),
            "myorg/myrepo"
        );
        
        assert_eq!(
            extract_path_from_url("user.name@bitbucket.org:company/project.git").unwrap(),
            "company/project"
        );
        
        // Test hostname extraction for organization-specific URLs
        assert_eq!(
            extract_hostname("org-25111032@github.com:smartcontractkit/chainlink.git").unwrap(),
            "github.com"
        );
        
        assert_eq!(
            extract_hostname("deploy-key-123@gitlab.com:myorg/myrepo.git").unwrap(),
            "gitlab.com"
        );
        
        // Test URL format detection for organization-specific URLs
        assert!(is_ssh_url("org-25111032@github.com:smartcontractkit/chainlink.git"));
        assert!(is_ssh_url("deploy-key-123@gitlab.com:myorg/myrepo.git"));
        assert!(is_ssh_url("user.name@bitbucket.org:company/project.git"));
        
        // Test validation for organization-specific URLs
        assert!(validate_git_url("org-25111032@github.com:smartcontractkit/chainlink.git").is_ok());
        assert!(validate_git_url("deploy-key-123@gitlab.com:myorg/myrepo.git").is_ok());
        assert!(validate_git_url("user.name@bitbucket.org:company/project.git").is_ok());
        
        // Ensure backward compatibility with standard git@ URLs
        assert_eq!(
            extract_path_from_url("git@github.com:org/repo.git").unwrap(),
            "org/repo"
        );
        assert!(is_ssh_url("git@github.com:org/repo.git"));
        assert!(validate_git_url("git@github.com:org/repo.git").is_ok());
    }
}