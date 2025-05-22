//! Validation utilities for the Mirror SDK.

use crate::error::ValidationError;
use crate::models::{MirrorConfig, Repository};

/// Validates a repository configuration.
pub fn validate_repository(repo: &Repository) -> Result<(), ValidationError> {
    repo.validate()
}

/// Validates the entire mirror.toml configuration.
pub fn validate_config(config: &MirrorConfig) -> Result<(), ValidationError> {
    config.validate()
}

/// Checks for path conflicts between repositories.
pub fn check_path_conflicts(config: &MirrorConfig) -> Result<(), ValidationError> {
    for (i, repo1) in config.repositories.iter().enumerate() {
        for (j, repo2) in config.repositories.iter().enumerate() {
            if i != j && repo1.path == repo2.path {
                return Err(ValidationError::PathConflict(
                    repo1.path.clone(),
                    repo2.path.clone(),
                ));
            }
            
            // Check for path prefix conflicts (one path is a prefix of another)
            if i != j && (repo1.path.starts_with(&repo2.path) || repo2.path.starts_with(&repo1.path)) {
                // Only report if one path is a direct parent of another (with a trailing slash)
                let path1 = if !repo1.path.ends_with('/') {
                    format!("{}/", repo1.path)
                } else {
                    repo1.path.clone()
                };
                
                let path2 = if !repo2.path.ends_with('/') {
                    format!("{}/", repo2.path)
                } else {
                    repo2.path.clone()
                };
                
                if path1.starts_with(&path2) || path2.starts_with(&path1) {
                    return Err(ValidationError::PathConflict(
                        repo1.path.clone(),
                        repo2.path.clone(),
                    ));
                }
            }
        }
    }
    
    Ok(())
}

/// Checks for duplicate IDs in the configuration.
pub fn check_duplicate_ids(config: &MirrorConfig) -> Result<(), ValidationError> {
    let mut ids = Vec::new();
    
    for repo in &config.repositories {
        if let Some(id) = &repo.id {
            if ids.contains(id) {
                return Err(ValidationError::DuplicateId(id.clone()));
            }
            ids.push(id.clone());
        }
    }
    
    Ok(())
}

/// Validates a repository origin URL.
/// 
/// This is a basic validation that checks if the origin contains a colon,
/// which is required for Git URLs (e.g., git@github.com:user/repo.git or https://github.com/user/repo.git).
pub fn validate_origin(origin: &str) -> Result<(), ValidationError> {
    if !origin.contains(':') {
        return Err(ValidationError::InvalidOrigin(origin.to_string()));
    }
    
    Ok(())
}

/// Validates a repository path.
/// 
/// This is a basic validation that checks if the path contains "..".
pub fn validate_path(path: &str) -> Result<(), ValidationError> {
    if path.contains("..") {
        return Err(ValidationError::InvalidPath(path.to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::repository::RepositoryBuilder;
    
    #[test]
    fn test_validate_repository() {
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        let result = validate_repository(&repo);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_validate_config() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        config.repositories.push(repo);
        
        let result = validate_config(&config);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_check_path_conflicts() {
        let mut config = MirrorConfig::new();
        
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo1")
            .build()
            .unwrap();
        
        let repo2 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo2.git")
            .branch("main")
            .path("example/repo2")
            .build()
            .unwrap();
        
        config.repositories.push(repo1);
        config.repositories.push(repo2);
        
        let result = check_path_conflicts(&config);
        assert!(result.is_ok());
        
        // Add a repository with a conflicting path
        let repo3 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo3.git")
            .branch("main")
            .path("example/repo1")
            .build()
            .unwrap();
        
        config.repositories.push(repo3);
        
        let result = check_path_conflicts(&config);
        assert!(result.is_err());
        match result {
            Err(ValidationError::PathConflict(_, _)) => (),
            _ => panic!("Expected PathConflict error"),
        }
    }
    
    #[test]
    fn test_check_duplicate_ids() {
        let mut config = MirrorConfig::new();
        
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo1")
            .id("repo1-id")
            .build()
            .unwrap();
        
        let repo2 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo2.git")
            .branch("main")
            .path("example/repo2")
            .id("repo2-id")
            .build()
            .unwrap();
        
        config.repositories.push(repo1);
        config.repositories.push(repo2);
        
        let result = check_duplicate_ids(&config);
        assert!(result.is_ok());
        
        // Add a repository with a duplicate ID
        let repo3 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo3.git")
            .branch("main")
            .path("example/repo3")
            .id("repo1-id")
            .build()
            .unwrap();
        
        config.repositories.push(repo3);
        
        let result = check_duplicate_ids(&config);
        assert!(result.is_err());
        match result {
            Err(ValidationError::DuplicateId(_)) => (),
            _ => panic!("Expected DuplicateId error"),
        }
    }
    
    #[test]
    fn test_validate_origin() {
        let result = validate_origin("git@github.com:example/repo.git");
        assert!(result.is_ok());
        
        let result = validate_origin("https://github.com/example/repo.git");
        assert!(result.is_ok());
        
        let result = validate_origin("invalid-origin");
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidOrigin(_)) => (),
            _ => panic!("Expected InvalidOrigin error"),
        }
    }
    
    #[test]
    fn test_validate_path() {
        let result = validate_path("example/repo");
        assert!(result.is_ok());
        
        let result = validate_path("../example/repo");
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidPath(_)) => (),
            _ => panic!("Expected InvalidPath error"),
        }
    }
}