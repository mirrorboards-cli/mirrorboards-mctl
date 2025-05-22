//! SDK settings and configuration.

use std::path::{Path, PathBuf};

/// SDK configuration settings.
#[derive(Debug, Clone)]
pub struct ConfigSettings {
    /// Default path for mirror.toml.
    pub default_config_path: Option<PathBuf>,
    
    /// Whether to validate repository paths.
    pub validate_paths: bool,
    
    /// Whether to validate repository origins.
    pub validate_origins: bool,
}

impl ConfigSettings {
    /// Creates a new settings instance with custom values.
    pub fn new(
        default_config_path: Option<PathBuf>,
        validate_paths: bool,
        validate_origins: bool,
    ) -> Self {
        Self {
            default_config_path,
            validate_paths,
            validate_origins,
        }
    }
    
    /// Sets the default config path.
    pub fn with_default_config_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.default_config_path = Some(path.as_ref().to_path_buf());
        self
    }
    
    /// Sets whether to validate repository paths.
    pub fn with_validate_paths(mut self, validate: bool) -> Self {
        self.validate_paths = validate;
        self
    }
    
    /// Sets whether to validate repository origins.
    pub fn with_validate_origins(mut self, validate: bool) -> Self {
        self.validate_origins = validate;
        self
    }
}

impl Default for ConfigSettings {
    fn default() -> Self {
        Self {
            default_config_path: None,
            validate_paths: true,
            validate_origins: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_settings() {
        let settings = ConfigSettings::default();
        
        assert_eq!(settings.default_config_path, None);
        assert_eq!(settings.validate_paths, true);
        assert_eq!(settings.validate_origins, true);
    }
    
    #[test]
    fn test_custom_settings() {
        let path = PathBuf::from("/path/to/mirror.toml");
        let settings = ConfigSettings::new(
            Some(path.clone()),
            false,
            false,
        );
        
        assert_eq!(settings.default_config_path, Some(path));
        assert_eq!(settings.validate_paths, false);
        assert_eq!(settings.validate_origins, false);
    }
    
    #[test]
    fn test_builder_pattern() {
        let path = PathBuf::from("/path/to/mirror.toml");
        let settings = ConfigSettings::default()
            .with_default_config_path(&path)
            .with_validate_paths(false)
            .with_validate_origins(true);
        
        assert_eq!(settings.default_config_path, Some(path));
        assert_eq!(settings.validate_paths, false);
        assert_eq!(settings.validate_origins, true);
    }
}