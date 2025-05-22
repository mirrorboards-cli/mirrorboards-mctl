//! Initialize new mirror.toml configuration.

use std::path::Path;

use crate::error::MirrorError;
use crate::fs;
use crate::models::MirrorConfig;

/// Initializes a new empty mirror.toml configuration file.
/// 
/// If the file already exists, an error is returned unless `force` is true.
pub fn init_config<P: AsRef<Path>>(path: P, force: bool) -> Result<MirrorConfig, MirrorError> {
    let path_ref = path.as_ref();
    
    // Check if the file already exists
    if path_ref.exists() && !force {
        return Err(MirrorError::InvalidConfiguration(
            format!("Configuration file already exists at {}", path_ref.display())
        ));
    }
    
    // Create a new empty configuration
    let config = MirrorConfig::new();
    
    // Write the configuration to the file
    fs::write_config(&config, path_ref)?;
    
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_init_config_new_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mirror.toml");
        
        let config = init_config(&path, false).unwrap();
        assert!(config.repositories.is_empty());
        assert!(path.exists());
    }
    
    #[test]
    fn test_init_config_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("mirror.toml");
        
        // Create the file
        std::fs::write(&path, "").unwrap();
        
        // Try to initialize without force
        let result = init_config(&path, false);
        assert!(result.is_err());
        
        // Try to initialize with force
        let config = init_config(&path, true).unwrap();
        assert!(config.repositories.is_empty());
    }
}