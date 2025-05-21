use crate::config::{load_config, Repository};
use crate::error::{MctlError, MctlResult};
use log::{debug, info, warn};
use std::path::Path;

/// Execute the add command
pub fn execute(
    config_path: &Path,
    git_url: Option<String>,
    path: Option<String>,
    branch: Option<String>,
    message: Option<String>,
) -> MctlResult<()> {
    // Validate required parameters
    let git_url = match git_url {
        Some(url) => url,
        None => return Err(MctlError::MissingParameter("git-url".to_string())),
    };

    let path = match path {
        Some(p) => p,
        None => return Err(MctlError::MissingParameter("path".to_string())),
    };

    // Load the configuration
    let mut config = load_config(config_path)?;

    // Create a new repository entry
    let repository = Repository::new(git_url.clone(), path.clone(), branch.clone());

    // Validate the repository
    repository.validate()?;

    // Add the repository to the configuration
    config.add_repository(repository)?;

    // Save the configuration
    config.save(config_path)?;

    // Print success message
    if let Some(msg) = message {
        info!("{}", msg);
    } else {
        info!(
            "Added repository {} to {} in configuration",
            git_url, path
        );
    }

    Ok(())
}