//! Default paths and environment variables for the Mirror SDK.

use std::env;
use std::path::{Path, PathBuf};

use crate::error::MirrorError;
use crate::fs::path::resolve_path;

/// Environment variable for the mirror.toml path.
pub const ENV_MIRROR_CONFIG: &str = "MIRROR_CONFIG";

/// Default filename for mirror.toml.
pub const DEFAULT_CONFIG_FILENAME: &str = "mirror.toml";

/// Gets the path to the mirror.toml file.
/// 
/// The path is determined in the following order:
/// 1. From the MIRROR_CONFIG environment variable if set
/// 2. From the provided default path if not None
/// 3. From the current working directory with the default filename
pub fn get_config_path(default_path: Option<&Path>) -> Result<PathBuf, MirrorError> {
    // Check environment variable
    if let Ok(path) = env::var(ENV_MIRROR_CONFIG) {
        return resolve_path(path);
    }
    
    // Use provided default path
    if let Some(path) = default_path {
        return resolve_path(path);
    }
    
    // Use current working directory with default filename
    let current_dir = env::current_dir()
        .map_err(|e| MirrorError::Io(e))?;
    
    Ok(current_dir.join(DEFAULT_CONFIG_FILENAME))
}

/// Gets the user's home directory.
pub fn get_home_dir() -> Result<PathBuf, MirrorError> {
    dirs::home_dir()
        .ok_or_else(|| MirrorError::Environment("Could not determine home directory".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_get_config_path_with_env() {
        let temp_path = tempdir().unwrap();
        let config_path = temp_path.path().join("custom-mirror.toml");
        
        // Set environment variable
        env::set_var(ENV_MIRROR_CONFIG, config_path.to_string_lossy().to_string());
        
        let result = get_config_path(None).unwrap();
        assert_eq!(result, config_path);
        
        // Clean up
        env::remove_var(ENV_MIRROR_CONFIG);
    }
    
    #[test]
    fn test_get_config_path_with_default() {
        // Ensure environment variable is not set
        env::remove_var(ENV_MIRROR_CONFIG);
        
        let temp_path = tempdir().unwrap();
        let default_path = temp_path.path().join("default-mirror.toml");
        
        let result = get_config_path(Some(&default_path)).unwrap();
        assert_eq!(result, default_path);
    }
    
    #[test]
    fn test_get_config_path_with_current_dir() {
        // Ensure environment variable is not set
        env::remove_var(ENV_MIRROR_CONFIG);
        
        let result = get_config_path(None).unwrap();
        let expected = env::current_dir().unwrap().join(DEFAULT_CONFIG_FILENAME);
        
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_get_home_dir() {
        let result = get_home_dir();
        assert!(result.is_ok());
        assert!(result.unwrap().exists());
    }
}