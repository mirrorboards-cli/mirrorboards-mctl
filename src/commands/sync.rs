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

/// Saves (commits and pushes) changes in all repositories
/// For each repository with changes, commits and pushes those changes
/// Uses the provided commit message or generates one from the repo name
pub fn save_repositories(config: Config, message: Option<String>) -> Result<()> {
    let git_handler = GitHandler::new();
    let mut pushed = 0;
    let mut unchanged = 0;
    let total = config.repositories.len();

    for repo in config.repositories {
        // Skip repositories without git
        if !repo.git {
            continue;
        }

        let path = PathBuf::from(&repo.path);
        
        // Skip repositories that don't exist or aren't git repositories
        if !GitHandler::repository_exists(&path) {
            continue;
        }

        // Check if repository has changes
        if git_handler.has_changes(&path)? {
            // Generate commit message if not provided
            let commit_message = match &message {
                Some(msg) => msg.clone(),
                None => {
                    // Try to get repository name from remote URL
                    let origin_url = match git_handler.get_origin_url(&path) {
                        Ok(url) => url,
                        Err(_) => String::from("repository"),
                    };
                    
                    let repo_name = git_handler.extract_repo_name_from_url(&origin_url);
                    format!("Update {}", repo_name)
                }
            };

            // Display repository name with color
            println!("\n{}", output::colorize(&format!("Repository: {}", path.display()), "bold blue"));
            
            // Commit changes
            println!("{}", output::colorize(&format!("Committing with message: {}", commit_message), "cyan"));
            git_handler.commit_changes(&path, &commit_message)?;
            
            // Push changes
            println!("{}", output::colorize("Pushing changes...", "yellow"));
            git_handler.push_changes(&path)?;
            
            println!("{}", output::colorize("Changes saved successfully", "green"));
            pushed += 1;
        } else {
            unchanged += 1;
        }
    }

    // Print summary
    println!("\n{}", output::colorize("Summary:", "bold"));
    println!("Total repositories: {}", output::colorize(&total.to_string(), "bold"));
    
    // Color based on whether changes were pushed
    let push_color = if pushed > 0 { "green" } else { "white" };
    println!("Changes pushed: {}", output::colorize(&pushed.to_string(), push_color));
    
    println!("No changes: {}", output::colorize(&unchanged.to_string(), "white"));
    
    Ok(())
}