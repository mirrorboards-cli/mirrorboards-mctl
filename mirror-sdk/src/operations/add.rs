//! Add repository to mirror.toml configuration.

use crate::error::MirrorError;
use crate::models::{MirrorConfig, Repository};

/// Adds a repository to the configuration.
/// 
/// If a repository with the same path already exists, an error is returned.
/// If a repository with the same ID already exists, an error is returned.
pub fn add_repository(config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
    // Validate the repository
    repo.validate()?;
    
    // Check for duplicate path
    if config.find_by_path(&repo.path).is_some() {
        return Err(MirrorError::RepositoryAlreadyExists(
            format!("Repository with path '{}' already exists", repo.path)
        ));
    }
    
    // Check for duplicate ID if the repository has an ID
    if let Some(id) = &repo.id {
        if config.find_by_id(id).is_some() {
            return Err(MirrorError::RepositoryAlreadyExists(
                format!("Repository with ID '{}' already exists", id)
            ));
        }
    }
    
    // Add the repository
    config.repositories.push(repo);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::repository::RepositoryBuilder;
    
    #[test]
    fn test_add_repository() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        add_repository(&mut config, repo).unwrap();
        
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].path, "example/repo");
    }
    
    #[test]
    fn test_add_repository_duplicate_path() {
        let mut config = MirrorConfig::new();
        
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        let repo2 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo2.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        add_repository(&mut config, repo1).unwrap();
        
        let result = add_repository(&mut config, repo2);
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryAlreadyExists(_)) => (),
            _ => panic!("Expected RepositoryAlreadyExists error"),
        }
    }
    
    #[test]
    fn test_add_repository_duplicate_id() {
        let mut config = MirrorConfig::new();
        
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo1")
            .id("duplicate-id")
            .build()
            .unwrap();
        
        let repo2 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo2.git")
            .branch("main")
            .path("example/repo2")
            .id("duplicate-id")
            .build()
            .unwrap();
        
        add_repository(&mut config, repo1).unwrap();
        
        let result = add_repository(&mut config, repo2);
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryAlreadyExists(_)) => (),
            _ => panic!("Expected RepositoryAlreadyExists error"),
        }
    }
}