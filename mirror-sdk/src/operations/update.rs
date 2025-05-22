//! Update repository in mirror.toml configuration.

use crate::error::MirrorError;
use crate::models::{MirrorConfig, Repository};

/// Updates an existing repository in the configuration.
/// 
/// The repository is identified by its path. If no repository with the given path exists,
/// an error is returned.
pub fn update_repository(config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
    // Validate the repository
    repo.validate()?;
    
    // Find the repository by path
    let index = config.repositories.iter()
        .position(|r| r.path == repo.path)
        .ok_or_else(|| MirrorError::RepositoryNotFound(
            format!("Repository with path '{}' not found", repo.path)
        ))?;
    
    // Check for ID conflicts if the repository has an ID
    if let Some(id) = &repo.id {
        let id_conflict = config.repositories.iter()
            .position(|r| r.id.as_ref().map_or(false, |r_id| r_id == id))
            .filter(|&pos| pos != index);
        
        if id_conflict.is_some() {
            return Err(MirrorError::RepositoryAlreadyExists(
                format!("Repository with ID '{}' already exists", id)
            ));
        }
    }
    
    // Update the repository
    config.repositories[index] = repo;
    
    Ok(())
}

/// Updates an existing repository in the configuration by ID.
/// 
/// The repository is identified by its ID. If no repository with the given ID exists,
/// an error is returned.
pub fn update_repository_by_id(config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
    // Validate the repository
    repo.validate()?;
    
    // Ensure the repository has an ID
    let id = repo.id.as_ref().ok_or_else(|| MirrorError::InvalidConfiguration(
        "Repository must have an ID to update by ID".to_string()
    ))?;
    
    // Find the repository by ID
    let index = config.repositories.iter()
        .position(|r| r.id.as_ref().map_or(false, |r_id| r_id == id))
        .ok_or_else(|| MirrorError::RepositoryNotFound(
            format!("Repository with ID '{}' not found", id)
        ))?;
    
    // Check for path conflicts
    let path_conflict = config.repositories.iter()
        .position(|r| r.path == repo.path)
        .filter(|&pos| pos != index);
    
    if path_conflict.is_some() {
        return Err(MirrorError::RepositoryAlreadyExists(
            format!("Repository with path '{}' already exists", repo.path)
        ));
    }
    
    // Update the repository
    config.repositories[index] = repo;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::repository::RepositoryBuilder;
    
    #[test]
    fn test_update_repository() {
        let mut config = MirrorConfig::new();
        
        let repo1 = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1.git")
            .branch("main")
            .path("example/repo1")
            .build()
            .unwrap();
        
        config.repositories.push(repo1);
        
        let updated_repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1-updated.git")
            .branch("develop")
            .path("example/repo1")
            .build()
            .unwrap();
        
        update_repository(&mut config, updated_repo).unwrap();
        
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].origin, "git@github.com:example/repo1-updated.git");
        assert_eq!(config.repositories[0].branch, "develop");
    }
    
    #[test]
    fn test_update_repository_not_found() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("nonexistent/path")
            .build()
            .unwrap();
        
        let result = update_repository(&mut config, repo);
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryNotFound(_)) => (),
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }
    
    #[test]
    fn test_update_repository_id_conflict() {
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
        
        let updated_repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1-updated.git")
            .branch("develop")
            .path("example/repo1")
            .id("repo2-id") // Conflict with repo2
            .build()
            .unwrap();
        
        let result = update_repository(&mut config, updated_repo);
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryAlreadyExists(_)) => (),
            _ => panic!("Expected RepositoryAlreadyExists error"),
        }
    }
    
    #[test]
    fn test_update_repository_by_id() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .id("repo-id")
            .build()
            .unwrap();
        
        config.repositories.push(repo);
        
        let updated_repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo-updated.git")
            .branch("develop")
            .path("example/repo-updated")
            .id("repo-id")
            .build()
            .unwrap();
        
        update_repository_by_id(&mut config, updated_repo).unwrap();
        
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].origin, "git@github.com:example/repo-updated.git");
        assert_eq!(config.repositories[0].branch, "develop");
        assert_eq!(config.repositories[0].path, "example/repo-updated");
    }
    
    #[test]
    fn test_update_repository_by_id_no_id() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        let result = update_repository_by_id(&mut config, repo);
        assert!(result.is_err());
        match result {
            Err(MirrorError::InvalidConfiguration(_)) => (),
            _ => panic!("Expected InvalidConfiguration error"),
        }
    }
    
    #[test]
    fn test_update_repository_by_id_not_found() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .id("nonexistent-id")
            .build()
            .unwrap();
        
        let result = update_repository_by_id(&mut config, repo);
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryNotFound(_)) => (),
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }
    
    #[test]
    fn test_update_repository_by_id_path_conflict() {
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
        
        let updated_repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo1-updated.git")
            .branch("develop")
            .path("example/repo2") // Conflict with repo2
            .id("repo1-id")
            .build()
            .unwrap();
        
        let result = update_repository_by_id(&mut config, updated_repo);
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryAlreadyExists(_)) => (),
            _ => panic!("Expected RepositoryAlreadyExists error"),
        }
    }
}