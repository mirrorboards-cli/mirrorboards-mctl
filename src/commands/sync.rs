use crate::config::Config;
use crate::git::GitHandler;
use crate::output;
use anyhow::Result;
use std::path::PathBuf;

/// Synchronizes repositories defined in the configuration
/// Clones missing repositories with submodules and updates submodules in existing repositories
pub fn sync_repositories(config: Config) -> Result<()> {
    let git_handler = GitHandler::new();
    let mut cloned = 0;
    let mut skipped = 0;
    let total = config.repositories.len();

    for repo in config.repositories {
        let path = PathBuf::from(&repo.path);
        
        if GitHandler::repository_exists(&path) {
            // Repository exists, but we should update its submodules
            if repo.git {
                git_handler.update_submodules(&path)?;
            }
            output::print_skipping(&path);
            skipped += 1;
            continue;
        }

        git_handler.clone_repository(&repo.origin, &path)?;
        
        // If git flag is false, remove the .git directory
        if !repo.git {
            git_handler.remove_git_directory(&path)?;
        }
        
        cloned += 1;
    }

    output::print_summary(total, cloned, skipped);
    Ok(())
}