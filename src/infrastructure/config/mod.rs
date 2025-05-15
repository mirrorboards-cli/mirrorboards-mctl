//! # Configuration Provider Module
//!
//! This module is responsible for loading and parsing the TOML configuration file.
//! It implements path expansion, environment variable substitution, and configuration 
//! discovery according to the defined search paths.

use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context, anyhow};
use toml;
use dirs;
use home;

use crate::domain::configuration::{Config, PathExpander, ConfigValidator};
use crate::domain::error::ConfigError;

/// Configuration provider for loading and parsing TOML configuration
pub struct ConfigProvider {
    /// Explicitly provided config path (if any)
    config_path: Option<PathBuf>,
    /// Path expander for resolving paths with home dir expansion and env vars
    path_expander: Box<dyn PathExpander>,
    /// Config validator for checking configuration validity
    validator: Box<dyn ConfigValidator>,
}

impl ConfigProvider {
    /// Create a new configuration provider with an optional explicit config path
    pub fn new(config_path: Option<PathBuf>) -> Self {
        Self {
            config_path,
            path_expander: Box::new(DefaultPathExpander {}),
            validator: Box::new(DefaultConfigValidator {}),
        }
    }
    
    /// Load the configuration file with layered approach (system, user, local)
    pub fn load_config(&self) -> Result<Config> {
        // Create a base config with defaults
        let mut config = Config::default();
        
        // Load configuration files in order of increasing precedence
        self.load_system_config(&mut config)?;
        self.load_user_config(&mut config)?;
        self.load_local_config(&mut config)?;
        self.load_explicit_config(&mut config)?;
        
        // Expand paths after all configurations are merged
        self.path_expander.expand_paths(&mut config);
        
        // Validate configuration
        self.validator.validate(&config)
            .map_err(|msg| anyhow!(ConfigError::ConfigValidationError { message: msg }))?;
        
        Ok(config)
    }
    
    /// Find the configuration file according to search paths
    pub fn find_config_file(&self) -> Result<PathBuf> {
        // Check explicitly provided path first
        if let Some(path) = &self.config_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }
        
        // Check current directory
        let current_dir_config = PathBuf::from("./mirror.toml");
        if current_dir_config.exists() {
            return Ok(current_dir_config);
        }
        
        // Check user config directory
        if let Some(user_config_dir) = dirs::config_dir() {
            let user_config_path = user_config_dir.join("mctl").join("mirror.toml");
            if user_config_path.exists() {
                return Ok(user_config_path);
            }
        }
        
        // Check system-wide config
        let system_config = PathBuf::from("/etc/mctl/mirror.toml");
        if system_config.exists() {
            return Ok(system_config);
        }
        
        // No config file found
        let mut search_paths = Vec::new();
        if let Some(path) = &self.config_path {
            search_paths.push(path.clone());
        }
        search_paths.push(current_dir_config);
        if let Some(user_config_dir) = dirs::config_dir() {
            search_paths.push(user_config_dir.join("mctl").join("mirror.toml"));
        }
        search_paths.push(system_config);
        
        Err(anyhow!(ConfigError::ConfigFileNotFound {
            search_paths,
        }))
    }
    
    /// Load system-wide configuration from /etc/mctl/mirror.toml
    fn load_system_config(&self, config: &mut Config) -> Result<()> {
        let system_config_path = PathBuf::from("/etc/mctl/mirror.toml");
        if system_config_path.exists() {
            let system_config = self.load_config_from_file(&system_config_path)?;
            self.merge_configs(config, &system_config);
            log::debug!("Loaded system configuration from {}", system_config_path.display());
        }
        Ok(())
    }
    
    /// Load user-specific configuration from ~/.config/mctl/mirror.toml
    pub fn load_user_config(&self, config: &mut Config) -> Result<()> {
        if let Some(user_config_dir) = dirs::config_dir() {
            let user_config_path = user_config_dir.join("mctl").join("mirror.toml");
            if user_config_path.exists() {
                let user_config = self.load_config_from_file(&user_config_path)?;
                self.merge_configs(config, &user_config);
                log::debug!("Loaded user configuration from {}", user_config_path.display());
            }
        }
        Ok(())
    }
    
    /// Load local configuration from ./mirror.toml
    pub fn load_local_config(&self, config: &mut Config) -> Result<()> {
        let local_config_path = PathBuf::from("./mirror.toml");
        if local_config_path.exists() {
            let local_config = self.load_config_from_file(&local_config_path)?;
            self.merge_configs(config, &local_config);
            log::debug!("Loaded local configuration from {}", local_config_path.display());
        }
        Ok(())
    }
    
    /// Load configuration from explicitly provided path (if any)
    pub fn load_explicit_config(&self, config: &mut Config) -> Result<()> {
        if let Some(explicit_path) = &self.config_path {
            if explicit_path.exists() {
                let explicit_config = self.load_config_from_file(explicit_path)?;
                self.merge_configs(config, &explicit_config);
                log::debug!("Loaded explicit configuration from {}", explicit_path.display());
            }
        }
        Ok(())
    }
    
    /// Load and parse a TOML configuration file
    pub fn load_config_from_file(&self, path: &Path) -> Result<Config> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read configuration file at {}", path.display()))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML configuration at {}", path.display()))?;
        
        Ok(config)
    }
    
    /// Merge source configuration into target configuration
    fn merge_configs(&self, target: &mut Config, source: &Config) {
        // Merge global settings
        if source.global.parallel != target.global.parallel {
            target.global.parallel = source.global.parallel;
        }
        if source.global.max_threads != target.global.max_threads {
            target.global.max_threads = source.global.max_threads;
        }
        
        // Merge SSH authentication settings
        if source.auth.ssh.key_path.is_some() {
            target.auth.ssh.key_path = source.auth.ssh.key_path.clone();
        }
        if source.auth.ssh.passphrase_command.is_some() {
            target.auth.ssh.passphrase_command = source.auth.ssh.passphrase_command.clone();
        }
        if source.auth.ssh.known_hosts_path.is_some() {
            target.auth.ssh.known_hosts_path = source.auth.ssh.known_hosts_path.clone();
        }
        
        // Merge logging configuration
        if source.logging.level != target.logging.level {
            target.logging.level = source.logging.level.clone();
        }
        if source.logging.file.is_some() {
            target.logging.file = source.logging.file.clone();
        }
        if source.logging.format != target.logging.format {
            target.logging.format = source.logging.format.clone();
        }
        
        // Merge command settings
        // Sync command
        if source.commands.sync.recursive != target.commands.sync.recursive {
            target.commands.sync.recursive = source.commands.sync.recursive;
        }
        if source.commands.sync.depth != target.commands.sync.depth {
            target.commands.sync.depth = source.commands.sync.depth;
        }
        if source.commands.sync.timeout_seconds != target.commands.sync.timeout_seconds {
            target.commands.sync.timeout_seconds = source.commands.sync.timeout_seconds;
        }
        
        // Status command
        if source.commands.status.include_untracked != target.commands.status.include_untracked {
            target.commands.status.include_untracked = source.commands.status.include_untracked;
        }
        if source.commands.status.timeout_seconds != target.commands.status.timeout_seconds {
            target.commands.status.timeout_seconds = source.commands.status.timeout_seconds;
        }
        
        // Save command
        if source.commands.save.push != target.commands.save.push {
            target.commands.save.push = source.commands.save.push;
        }
        if source.commands.save.sign_commits != target.commands.save.sign_commits {
            target.commands.save.sign_commits = source.commands.save.sign_commits;
        }
        
        // Merge repositories (append new ones and update existing ones)
        for source_repo in &source.repositories {
            // Check if repository already exists in target by path
            let existing_repo = target.repositories.iter_mut().find(|r| r.path == source_repo.path);
            
            match existing_repo {
                Some(repo) => {
                    // Update existing repository
                    if !source_repo.origin.is_empty() {
                        repo.origin = source_repo.origin.clone();
                    }
                    if source_repo.branch.is_some() {
                        repo.branch = source_repo.branch.clone();
                    }
                    if source_repo.is_git != repo.is_git {
                        repo.is_git = source_repo.is_git;
                    }
                    if source_repo.enabled != repo.enabled {
                        repo.enabled = source_repo.enabled;
                    }
                    if !source_repo.tags.is_empty() {
                        repo.tags = source_repo.tags.clone();
                    }
                    
                    // Merge repository-specific overrides
                    if let Some(source_overrides) = &source_repo.config_overrides {
                        if repo.config_overrides.is_none() {
                            repo.config_overrides = Some(source_overrides.clone());
                        } else if let Some(repo_overrides) = &mut repo.config_overrides {
                            // Merge SSH configuration
                            if let Some(source_ssh) = &source_overrides.ssh {
                                if repo_overrides.ssh.is_none() {
                                    repo_overrides.ssh = Some(source_ssh.clone());
                                } else if let Some(repo_ssh) = &mut repo_overrides.ssh {
                                    if source_ssh.key_path.is_some() {
                                        repo_ssh.key_path = source_ssh.key_path.clone();
                                    }
                                    if source_ssh.known_hosts_path.is_some() {
                                        repo_ssh.known_hosts_path = source_ssh.known_hosts_path.clone();
                                    }
                                    if source_ssh.passphrase_command.is_some() {
                                        repo_ssh.passphrase_command = source_ssh.passphrase_command.clone();
                                    }
                                }
                            }
                            
                            // Merge command configuration
                            if let Some(source_commands) = &source_overrides.commands {
                                if repo_overrides.commands.is_none() {
                                    repo_overrides.commands = Some(source_commands.clone());
                                } else if let Some(repo_commands) = &mut repo_overrides.commands {
                                    // Merge sync command settings
                                    if let Some(source_sync) = &source_commands.sync {
                                        if repo_commands.sync.is_none() {
                                            repo_commands.sync = Some(source_sync.clone());
                                        } else if let Some(repo_sync) = &mut repo_commands.sync {
                                            if source_sync.recursive.is_some() {
                                                repo_sync.recursive = source_sync.recursive;
                                            }
                                            if source_sync.depth.is_some() {
                                                repo_sync.depth = source_sync.depth;
                                            }
                                            if source_sync.timeout_seconds.is_some() {
                                                repo_sync.timeout_seconds = source_sync.timeout_seconds;
                                            }
                                        }
                                    }
                                    
                                    // Merge status command settings
                                    if let Some(source_status) = &source_commands.status {
                                        if repo_commands.status.is_none() {
                                            repo_commands.status = Some(source_status.clone());
                                        } else if let Some(repo_status) = &mut repo_commands.status {
                                            if source_status.include_untracked.is_some() {
                                                repo_status.include_untracked = source_status.include_untracked;
                                            }
                                            if source_status.timeout_seconds.is_some() {
                                                repo_status.timeout_seconds = source_status.timeout_seconds;
                                            }
                                        }
                                    }
                                    
                                    // Merge save command settings
                                    if let Some(source_save) = &source_commands.save {
                                        if repo_commands.save.is_none() {
                                            repo_commands.save = Some(source_save.clone());
                                        } else if let Some(repo_save) = &mut repo_commands.save {
                                            if source_save.push.is_some() {
                                                repo_save.push = source_save.push;
                                            }
                                            if source_save.sign_commits.is_some() {
                                                repo_save.sign_commits = source_save.sign_commits;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                None => {
                    // Add new repository
                    target.repositories.push(source_repo.clone());
                }
            }
        }
    }
    
    /// Get default configuration
    pub fn get_default_config(&self) -> Config {
        Config::default()
    }
}

/// Default implementation of path expander
struct DefaultPathExpander {}

impl PathExpander for DefaultPathExpander {
    /// Expand all paths in the configuration
    fn expand_paths(&self, config: &mut Config) {
        // Expand paths in SSH configuration
        if let Some(key_path) = &config.auth.ssh.key_path {
            config.auth.ssh.key_path = Some(self.expand_path(key_path).to_string_lossy().to_string());
        }
        
        if let Some(known_hosts_path) = &config.auth.ssh.known_hosts_path {
            config.auth.ssh.known_hosts_path = Some(self.expand_path(known_hosts_path).to_string_lossy().to_string());
        }
        
        // Expand paths in logging configuration
        if let Some(file_path) = &config.logging.file {
            config.logging.file = Some(self.expand_path(file_path).to_string_lossy().to_string());
        }
        
        // Expand paths in repositories
        for repo in &mut config.repositories {
            repo.path = self.expand_path(&repo.path.to_string_lossy());
            
            // Expand paths in repository-specific SSH configuration
            if let Some(repo_config) = &mut repo.config_overrides {
                if let Some(ssh_config) = &mut repo_config.ssh {
                    if let Some(key_path) = &ssh_config.key_path {
                        ssh_config.key_path = Some(self.expand_path(&key_path.to_string_lossy()));
                    }
                    
                    if let Some(known_hosts_path) = &ssh_config.known_hosts_path {
                        ssh_config.known_hosts_path = Some(self.expand_path(&known_hosts_path.to_string_lossy()));
                    }
                }
            }
        }
    }
    
    /// Expand a single path with environment variables and ~ substitution
    fn expand_path(&self, path: &str) -> PathBuf {
        let path_with_env = self.expand_env_vars(path);
        
        if path_with_env.starts_with("~/") || path_with_env == "~" {
            if let Some(home_dir) = home::home_dir() {
                let home_path = home_dir.to_string_lossy();
                return PathBuf::from(path_with_env.replacen("~", &home_path, 1));
            }
        }
        
        PathBuf::from(path_with_env)
    }
    
    /// Expand environment variables in a string using ${VAR} or ${VAR:-default} syntax
    fn expand_env_vars(&self, input: &str) -> String {
        let mut result = String::new();
        let mut chars = input.chars().peekable();
        
        while let Some(c) = chars.next() {
            if c == '$' && chars.peek() == Some(&'{') {
                chars.next(); // Consume '{'
                let mut var_name = String::new();
                let mut default_value = None;
                
                // Extract variable name and optional default value
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some(':') if chars.peek() == Some(&'-') => {
                            chars.next(); // Consume '-'
                            default_value = Some(String::new());
                        },
                        Some(c) if default_value.is_some() => {
                            default_value.as_mut().unwrap().push(c);
                        },
                        Some(c) => var_name.push(c),
                        None => break,
                    }
                }
                
                // Resolve environment variable
                match std::env::var(&var_name) {
                    Ok(value) => result.push_str(&value),
                    Err(_) => {
                        if let Some(default) = default_value {
                            result.push_str(&default);
                        }
                    }
                }
            } else {
                result.push(c);
            }
        }
        
        result
    }
}

/// Default implementation of config validator
struct DefaultConfigValidator {}

impl ConfigValidator for DefaultConfigValidator {
    /// Validate the configuration
    fn validate(&self, config: &Config) -> Result<(), String> {
        // Validate global settings
        if config.global.max_threads == 0 {
            return Err("Global max_threads must be greater than 0".to_string());
        }
        
        // Validate SSH configuration
        if let Some(key_path) = &config.auth.ssh.key_path {
            let path = Path::new(key_path);
            if !path.exists() && !key_path.contains("${") && !key_path.starts_with("~") {
                return Err(format!("SSH key path does not exist: {}", key_path));
            }
        }
        
        if let Some(known_hosts_path) = &config.auth.ssh.known_hosts_path {
            let path = Path::new(known_hosts_path);
            if !path.exists() && !known_hosts_path.contains("${") && !known_hosts_path.starts_with("~") {
                return Err(format!("SSH known hosts path does not exist: {}", known_hosts_path));
            }
        }
        
        // Validate logging configuration
        match config.logging.level.as_str() {
            "debug" | "info" | "warn" | "error" => {},
            level => return Err(format!("Invalid log level: {}. Must be one of: debug, info, warn, error", level)),
        }
        
        match config.logging.format.as_str() {
            "text" | "json" => {},
            format => return Err(format!("Invalid log format: {}. Must be one of: text, json", format)),
        }
        
        if let Some(file_path) = &config.logging.file {
            let log_dir = Path::new(file_path).parent();
            if let Some(dir) = log_dir {
                if !dir.exists() && !file_path.contains("${") && !file_path.starts_with("~") {
                    return Err(format!("Log directory does not exist: {}", dir.display()));
                }
            }
        }
        
        // Command configuration validation
        if config.commands.sync.timeout_seconds == 0 {
            return Err("Sync command timeout must be greater than 0 seconds".to_string());
        }
        
        if config.commands.status.timeout_seconds == 0 {
            return Err("Status command timeout must be greater than 0 seconds".to_string());
        }
        
        // Validate repositories
        for (index, repo) in config.repositories.iter().enumerate() {
            // Required fields
            if repo.path.as_os_str().is_empty() {
                return Err(format!("Repository at index {} is missing a path", index));
            }
            
            if repo.origin.is_empty() {
                return Err(format!("Repository at index {} is missing an origin URL", index));
            }
            
            // Repository-specific overrides validation
            if let Some(overrides) = &repo.config_overrides {
                // SSH config validation
                if let Some(ssh_config) = &overrides.ssh {
                    if let Some(key_path) = &ssh_config.key_path {
                        if !key_path.exists() && !key_path.to_string_lossy().contains("${") && !key_path.to_string_lossy().starts_with("~") {
                            return Err(format!("Repository SSH key path does not exist: {}", key_path.display()));
                        }
                    }
                }
                
                // Command config validation
                if let Some(commands) = &overrides.commands {
                    if let Some(sync) = &commands.sync {
                        if let Some(timeout) = sync.timeout_seconds {
                            if timeout == 0 {
                                return Err(format!("Repository '{}' sync timeout must be greater than 0", repo.path.display()));
                            }
                        }
                    }
                    
                    if let Some(status) = &commands.status {
                        if let Some(timeout) = status.timeout_seconds {
                            if timeout == 0 {
                                return Err(format!("Repository '{}' status timeout must be greater than 0", repo.path.display()));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_expand_env_vars() {
        let expander = DefaultPathExpander {};
        
        // Set test environment variable
        std::env::set_var("MCTL_TEST_VAR", "test_value");
        
        // Test simple expansion
        assert_eq!(expander.expand_env_vars("${MCTL_TEST_VAR}"), "test_value");
        
        // Test with default value when var exists
        assert_eq!(expander.expand_env_vars("${MCTL_TEST_VAR:-default}"), "test_value");
        
        // Test with default value when var doesn't exist
        assert_eq!(expander.expand_env_vars("${NONEXISTENT_VAR:-default}"), "default");
        
        // Test with text around variables
        assert_eq!(expander.expand_env_vars("prefix_${MCTL_TEST_VAR}_suffix"), "prefix_test_value_suffix");
    }
    
    #[test]
    fn test_expand_path() {
        let expander = DefaultPathExpander {};
        
        // Test home expansion
        let home_expanded = expander.expand_path("~/test");
        assert!(home_expanded.to_string_lossy().contains("/test"));
        assert!(!home_expanded.to_string_lossy().contains("~"));
        
        // Test no expansion needed
        let no_expansion = expander.expand_path("/absolute/path");
        assert_eq!(no_expansion, PathBuf::from("/absolute/path"));
        
        // Test environment variable in path
        std::env::set_var("MCTL_TEST_PATH", "/custom/path");
        let env_expanded = expander.expand_path("${MCTL_TEST_PATH}/file.txt");
        assert_eq!(env_expanded, PathBuf::from("/custom/path/file.txt"));
    }
    
    #[test]
    fn test_basic_config_validation() {
        let validator = DefaultConfigValidator {};
        
        // Create a minimal valid configuration
        let mut valid_config = Config::default();
        
        // Invalid log level
        let mut invalid_log_level = Config::default();
        invalid_log_level.logging.level = "invalid".to_string();
        assert!(validator.validate(&invalid_log_level).is_err());
        
        // Invalid log format
        let mut invalid_log_format = Config::default();
        invalid_log_format.logging.format = "invalid".to_string();
        assert!(validator.validate(&invalid_log_format).is_err());
        
        // Invalid max_threads
        let mut invalid_max_threads = Config::default();
        invalid_max_threads.global.max_threads = 0; // Must be > 0
        assert!(validator.validate(&invalid_max_threads).is_err());
    }
    
    #[test]
    fn test_load_basic_config_from_file() -> Result<()> {
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new()?;
        let config_content = r#"
            [global]
            max_threads = 16
            
            [logging]
            level = "debug"
        "#;
        temp_file.write_all(config_content.as_bytes())?;
        
        // Create provider and load the config
        let provider = ConfigProvider::new(None);
        let config = provider.load_config_from_file(temp_file.path())?;
        
        // Verify config was loaded correctly
        assert_eq!(config.global.max_threads, 16);
        assert_eq!(config.logging.level, "debug");
        
        Ok(())
    }
    
    #[test]
    fn test_basic_merge_configs() {
        // Create a provider for testing
        let provider = ConfigProvider::new(None);
        
        // Create base config
        let mut base_config = Config::default();
        base_config.global.max_threads = 4;
        base_config.logging.level = "debug".to_string();
        
        // Create overlay config
        let mut overlay_config = Config::default();
        overlay_config.global.max_threads = 8; // Different value
        overlay_config.logging.level = "info".to_string(); // Different value
        overlay_config.auth.ssh.key_path = Some("~/.ssh/custom_key".to_string()); // New value
        
        // Merge configs
        provider.merge_configs(&mut base_config, &overlay_config);
        
        // Check merged values
        assert_eq!(base_config.global.max_threads, 8); // From overlay
        assert_eq!(base_config.logging.level, "info"); // From overlay
        assert_eq!(base_config.auth.ssh.key_path, Some("~/.ssh/custom_key".to_string())); // From overlay
    }
}