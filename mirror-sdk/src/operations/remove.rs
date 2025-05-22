//! Remove repository from mirror.toml configuration.

use crate::error::MirrorError;
use crate::models::MirrorConfig;

/// Removes a repository from the configuration by path.
/// 
/// If no repository with the given path exists, an error is returned.
pub fn remove_repository_by_path(config: &mut MirrorConfig, path: &str) -> Result<(), MirrorError> {
    let index = config.repositories.iter()
        .position(|repo| repo.path == path)
        .ok_or_else(|| MirrorError::RepositoryNotFound(
            format!("Repository with path '{}' not found", path)
        ))?;
    
    config.repositories.remove(index);
    
    Ok(())
}

/// Removes a repository from the configuration by ID.
/// 
/// If no repository with the given ID exists, an error is returned.
pub fn remove_repository_by_id(config: &mut MirrorConfig, id: &str) -> Result<(), MirrorError> {
    let index = config.repositories.iter()
        .position(|repo| repo.id.as_ref().map_or(false, |repo_id| repo_id == id))
        .ok_or_else(|| MirrorError::RepositoryNotFound(
            format!("Repository with ID '{}' not found", id)
        ))?;
    
    config.repositories.remove(index);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::repository::RepositoryBuilder;
    
    #[test]
    fn test_remove_repository_by_path() {
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
        
        assert_eq!(config.repositories.len(), 2);
        
        remove_repository_by_path(&mut config, "example/repo1").unwrap();
        
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].path, "example/repo2");
    }
    
    #[test]
    fn test_remove_repository_by_path_not_found() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .build()
            .unwrap();
        
        config.repositories.push(repo);
        
        let result = remove_repository_by_path(&mut config, "nonexistent/path");
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryNotFound(_)) => (),
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }
    
    #[test]
    fn test_remove_repository_by_id() {
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
        
        assert_eq!(config.repositories.len(), 2);
        
        remove_repository_by_id(&mut config, "repo1-id").unwrap();
        
        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].id, Some("repo2-id".to_string()));
    }
    
    #[test]
    fn test_remove_repository_by_id_not_found() {
        let mut config = MirrorConfig::new();
        
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .id("repo-id")
            .build()
            .unwrap();
        
        config.repositories.push(repo);
        
        let result = remove_repository_by_id(&mut config, "nonexistent-id");
        assert!(result.is_err());
        match result {
            Err(MirrorError::RepositoryNotFound(_)) => (),
            _ => panic!("Expected RepositoryNotFound error"),
        }
    }
}