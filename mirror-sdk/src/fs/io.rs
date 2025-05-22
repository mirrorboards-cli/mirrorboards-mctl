//! File I/O operations for the Mirror SDK.

use std::fs;
use std::path::Path;

use crate::error::MirrorError;
use crate::models::MirrorConfig;

/// Reads a mirror.toml file and parses it into a MirrorConfig.
pub fn read_config<P: AsRef<Path>>(path: P) -> Result<MirrorConfig, MirrorError> {
    let content = fs::read_to_string(path.as_ref())
        .map_err(|e| MirrorError::Io(e))?;
    
    parse_config(&content)
}

/// Parses a string containing TOML into a MirrorConfig.
pub fn parse_config(content: &str) -> Result<MirrorConfig, MirrorError> {
    toml::from_str(content)
        .map_err(|e| MirrorError::TomlParse(e))
}

/// Writes a MirrorConfig to a mirror.toml file.
pub fn write_config<P: AsRef<Path>>(config: &MirrorConfig, path: P) -> Result<(), MirrorError> {
    let content = serialize_config(config)?;
    
    // Create parent directories if they don't exist
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)
            .map_err(|e| MirrorError::Io(e))?;
    }
    
    fs::write(path.as_ref(), content)
        .map_err(|e| MirrorError::Io(e))
}

/// Serializes a MirrorConfig to a TOML string.
pub fn serialize_config(config: &MirrorConfig) -> Result<String, MirrorError> {
    toml::to_string(config)
        .map_err(|e| MirrorError::TomlSerialize(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::repository::RepositoryBuilder;
    use tempfile::tempdir;

    #[test]
    fn test_serialize_deserialize() {
        let mut config = MirrorConfig::new();
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .tag("example")
            .build()
            .unwrap();
        config.repositories.push(repo);

        let toml_str = serialize_config(&config).unwrap();
        let parsed_config = parse_config(&toml_str).unwrap();

        assert_eq!(config, parsed_config);
    }

    #[test]
    fn test_write_read_config() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("mirror.toml");

        let mut config = MirrorConfig::new();
        let repo = RepositoryBuilder::new()
            .origin("git@github.com:example/repo.git")
            .branch("main")
            .path("example/repo")
            .tag("example")
            .build()
            .unwrap();
        config.repositories.push(repo);

        write_config(&config, &file_path).unwrap();
        let read_config = read_config(&file_path).unwrap();

        assert_eq!(config, read_config);
    }
}