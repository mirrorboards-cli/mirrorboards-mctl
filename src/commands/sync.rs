use crate::config::Config;
use crate::git::GitHandler;
use crate::output;
use anyhow::Result;
use std::path::PathBuf;

pub fn sync_repositories(config: Config) -> Result<()> {
    let git_handler = GitHandler::new();
    let mut cloned = 0;
    let mut skipped = 0;
    let total = config.repositories.len();

    for repo in config.repositories {
        let path = PathBuf::from(&repo.path);
        
        if GitHandler::repository_exists(&path) {
            output::print_skipping(&path);
            skipped += 1;
            continue;
        }

        git_handler.clone_repository(&repo.origin, &path)?;
        cloned += 1;
    }

    output::print_summary(total, cloned, skipped);
    Ok(())
}