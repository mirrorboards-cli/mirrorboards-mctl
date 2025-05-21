//! Git credentials module for MCTL
//!
//! This module handles secure credential management for git operations.

use git2::{Cred, CredentialType};
use log::{debug, warn};
use secrecy::{ExposeSecret, Secret};
use std::env;

/// Git credentials for authentication
#[derive(Debug, Clone)]
pub struct GitCredentials {
    /// Username for authentication
    username: Option<String>,

    /// Password for HTTPS authentication
    password: Option<Secret<String>>,

    /// SSH key path for SSH authentication
    ssh_key_path: Option<String>,

    /// SSH key passphrase
    ssh_passphrase: Option<Secret<String>>,
}

impl GitCredentials {
    /// Create a new empty GitCredentials instance
    pub fn new() -> Self {
        Self {
            username: None,
            password: None,
            ssh_key_path: None,
            ssh_passphrase: None,
        }
    }

    /// Create a new GitCredentials instance with HTTPS authentication
    pub fn with_https(username: String, password: String) -> Self {
        Self {
            username: Some(username),
            password: Some(Secret::new(password)),
            ssh_key_path: None,
            ssh_passphrase: None,
        }
    }

    /// Create a new GitCredentials instance with SSH authentication
    pub fn with_ssh(
        username: String,
        ssh_key_path: String,
        ssh_passphrase: Option<String>,
    ) -> Self {
        Self {
            username: Some(username),
            password: None,
            ssh_key_path: Some(ssh_key_path),
            ssh_passphrase: ssh_passphrase.map(Secret::new),
        }
    }

    /// Set username
    pub fn set_username(&mut self, username: String) {
        self.username = Some(username);
    }

    /// Set password for HTTPS authentication
    pub fn set_password(&mut self, password: String) {
        self.password = Some(Secret::new(password));
    }

    /// Set SSH key path for SSH authentication
    pub fn set_ssh_key_path(&mut self, ssh_key_path: String) {
        self.ssh_key_path = Some(ssh_key_path);
    }

    /// Set SSH key passphrase
    pub fn set_ssh_passphrase(&mut self, ssh_passphrase: String) {
        self.ssh_passphrase = Some(Secret::new(ssh_passphrase));
    }

    /// Get credentials from environment variables
    pub fn from_env() -> Self {
        let mut credentials = Self::new();

        // Try to get username from environment
        if let Ok(username) = env::var("GIT_USERNAME") {
            credentials.set_username(username);
        }

        // Try to get password from environment
        if let Ok(password) = env::var("GIT_PASSWORD") {
            credentials.set_password(password);
        }

        // Try to get SSH key path from environment
        if let Ok(ssh_key_path) = env::var("GIT_SSH_KEY_PATH") {
            credentials.set_ssh_key_path(ssh_key_path);
        }

        // Try to get SSH passphrase from environment
        if let Ok(ssh_passphrase) = env::var("GIT_SSH_PASSPHRASE") {
            credentials.set_ssh_passphrase(ssh_passphrase);
        }

        credentials
    }

    /// Get credentials callback for git2
    pub fn get_cred(
        &self,
        url: &str,
        username_from_url: Option<&str>,
        allowed_types: CredentialType,
    ) -> Result<Cred, git2::Error> {
        debug!("Getting credentials for URL: {}", url);

        // Use username from URL, or from credentials, or default to "git"
        let username = username_from_url
            .map(|s| s.to_string())
            .or_else(|| self.username.clone())
            .unwrap_or_else(|| "git".to_string());

        // Try SSH key authentication
        if allowed_types.contains(CredentialType::SSH_KEY) && self.ssh_key_path.is_some() {
            debug!("Trying SSH key authentication");
            let ssh_key_path = self.ssh_key_path.as_ref().unwrap();

            // Try with passphrase if available
            if let Some(passphrase) = &self.ssh_passphrase {
                return Cred::ssh_key(
                    &username,
                    None,
                    ssh_key_path.as_ref(),
                    Some(passphrase.expose_secret()),
                );
            }

            // Try without passphrase
            return Cred::ssh_key(&username, None, ssh_key_path.as_ref(), None);
        }

        // Try username/password authentication
        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) && self.password.is_some() {
            debug!("Trying username/password authentication");
            return Cred::userpass_plaintext(
                &username,
                self.password.as_ref().unwrap().expose_secret(),
            );
        }

        // Try SSH agent authentication
        if allowed_types.contains(CredentialType::SSH_KEY) {
            debug!("Trying SSH agent authentication");
            return Cred::ssh_key_from_agent(&username);
        }

        // Try default credentials
        debug!("Trying default credentials");
        Cred::default()
    }
}

impl Default for GitCredentials {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_credentials() {
        let credentials = GitCredentials::new();
        assert!(credentials.username.is_none());
        assert!(credentials.password.is_none());
        assert!(credentials.ssh_key_path.is_none());
        assert!(credentials.ssh_passphrase.is_none());
    }

    #[test]
    fn test_with_https() {
        let credentials = GitCredentials::with_https("user".to_string(), "pass".to_string());
        assert_eq!(credentials.username, Some("user".to_string()));
        assert_eq!(
            credentials.password.as_ref().unwrap().expose_secret(),
            "pass"
        );
        assert!(credentials.ssh_key_path.is_none());
        assert!(credentials.ssh_passphrase.is_none());
    }

    #[test]
    fn test_with_ssh() {
        let credentials = GitCredentials::with_ssh(
            "git".to_string(),
            "/path/to/key".to_string(),
            Some("passphrase".to_string()),
        );
        assert_eq!(credentials.username, Some("git".to_string()));
        assert!(credentials.password.is_none());
        assert_eq!(credentials.ssh_key_path, Some("/path/to/key".to_string()));
        assert_eq!(
            credentials.ssh_passphrase.as_ref().unwrap().expose_secret(),
            "passphrase"
        );
    }

    #[test]
    fn test_from_env() {
        // Set environment variables
        env::set_var("GIT_USERNAME", "env_user");
        env::set_var("GIT_PASSWORD", "env_pass");
        env::set_var("GIT_SSH_KEY_PATH", "/env/path/to/key");
        env::set_var("GIT_SSH_PASSPHRASE", "env_passphrase");

        let credentials = GitCredentials::from_env();
        assert_eq!(credentials.username, Some("env_user".to_string()));
        assert_eq!(
            credentials.password.as_ref().unwrap().expose_secret(),
            "env_pass"
        );
        assert_eq!(
            credentials.ssh_key_path,
            Some("/env/path/to/key".to_string())
        );
        assert_eq!(
            credentials.ssh_passphrase.as_ref().unwrap().expose_secret(),
            "env_passphrase"
        );

        // Clean up
        env::remove_var("GIT_USERNAME");
        env::remove_var("GIT_PASSWORD");
        env::remove_var("GIT_SSH_KEY_PATH");
        env::remove_var("GIT_SSH_PASSPHRASE");
    }
}
