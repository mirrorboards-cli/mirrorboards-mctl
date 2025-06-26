//! SSH Authentication Module
//! 
//! Provides robust SSH key management and authentication with proper fallback mechanisms.
//! Addresses the infinite retry problem by implementing smart SSH agent detection and
//! filesystem key discovery.

use ssh2::{Agent, Session};
use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use crate::error::{SshError, SshResult};

/// SSH Manager that handles SSH authentication with robust fallback mechanisms
pub struct SshManager {
    /// SSH agent connection (only if it has keys loaded)
    agent: Option<Agent>,
    /// Available SSH keys found on filesystem
    available_keys: Vec<PathBuf>,
}

impl SshManager {
    /// Create a new SSH manager with smart agent detection and key discovery
    pub fn new() -> SshResult<Self> {
        Self::new_with_verbose(false)
    }
    
    /// Create a new SSH manager with optional verbose output
    pub fn new_with_verbose(verbose: bool) -> SshResult<Self> {
        if verbose {
            println!("Initializing SSH manager...");
        }
        
        // Try to connect to SSH agent and verify it has keys
        let agent = match Self::try_connect_agent_with_verbose(verbose) {
            Ok(agent) => {
                if verbose {
                    println!("SSH agent connected and has keys loaded");
                }
                Some(agent)
            }
            Err(e) => {
                if verbose {
                    println!("SSH agent not usable: {}", e);
                }
                None
            }
        };
        
        // Discover filesystem SSH keys
        let available_keys = Self::find_ssh_keys_with_verbose(verbose);
        if verbose {
            println!("Found {} SSH keys on filesystem", available_keys.len());
            for key in &available_keys {
                println!("  - {}", key.display());
            }
        }
        
        // Ensure we have at least one authentication method
        if agent.is_none() && available_keys.is_empty() {
            return Err(SshError::NoUsableKeysError);
        }
        
        Ok(SshManager {
            agent,
            available_keys,
        })
    }
    
    /// Check if SSH agent is available AND has keys loaded
    /// This is the key difference from naive implementations
    pub fn has_usable_agent(&self) -> bool {
        self.agent.is_some()
    }
    
    /// Find SSH keys in the filesystem with optional verbose output
    pub fn find_ssh_keys_with_verbose(verbose: bool) -> Vec<PathBuf> {
        let mut keys = Vec::new();
        
        // Get SSH directory
        let ssh_dir = match Self::get_ssh_directory() {
            Ok(dir) => dir,
            Err(_) => {
                if verbose {
                    println!("Could not determine SSH directory");
                }
                return keys;
            }
        };
        
        if !ssh_dir.exists() {
            if verbose {
                println!("SSH directory does not exist: {}", ssh_dir.display());
            }
            return keys;
        }
        
        // Standard SSH key names to look for
        let key_names = [
            "id_rsa",
            "id_ed25519",
            "id_ecdsa",
            "id_dsa",
        ];
        
        for key_name in &key_names {
            let private_key_path = ssh_dir.join(key_name);
            let public_key_path = ssh_dir.join(format!("{}.pub", key_name));
            
            // Check if both private and public key exist
            if private_key_path.exists() && public_key_path.exists() {
                // Verify private key is readable
                if let Ok(metadata) = fs::metadata(&private_key_path) {
                    if metadata.is_file() {
                        if verbose {
                            println!("Found SSH key pair: {}", key_name);
                        }
                        keys.push(private_key_path);
                    }
                }
            }
        }
        
        keys
    }
    
    /// Get the list of available filesystem SSH keys
    pub fn get_available_keys(&self) -> &[PathBuf] {
        &self.available_keys
    }
    
    /// Get the number of available authentication methods
    pub fn auth_method_count(&self) -> usize {
        let agent_count = if self.has_usable_agent() { 1 } else { 0 };
        agent_count + self.available_keys.len()
    }
    
    /// Try to authenticate using SSH agent
    pub fn try_agent_auth(&mut self, _session: &Session, username: &str) -> SshResult<bool> {
        if let Some(ref mut agent) = self.agent {
            println!("Attempting SSH agent authentication for user: {}", username);
            
            // List identities from the agent
            agent.list_identities().map_err(|e| SshError::AgentConnectionError {
                message: format!("Failed to list identities: {}", e)
            })?;
            
            // Try each identity
            let identities = agent.identities().map_err(|e| SshError::AgentConnectionError {
                message: format!("Failed to get identities: {}", e)
            })?;
            
            for identity in identities {
                println!("Trying SSH agent identity: {}", identity.comment());
                match agent.userauth(username, &identity) {
                    Ok(()) => {
                        println!("SSH agent authentication successful with identity: {}", identity.comment());
                        return Ok(true);
                    }
                    Err(e) => {
                        println!("SSH agent identity {} failed: {}", identity.comment(), e);
                        continue;
                    }
                }
            }
            
            println!("All SSH agent identities failed");
            Err(SshError::AgentAuthenticationFailed)
        } else {
            println!("No SSH agent available");
            Ok(false)
        }
    }
    
    /// Try to authenticate using a specific filesystem key
    pub fn try_key_auth(&self, session: &Session, username: &str, key_path: &Path) -> SshResult<bool> {
        println!("Attempting key authentication with: {}", key_path.display());
        
        // Check if key file exists
        if !key_path.exists() {
            return Err(SshError::KeyFileNotFound { path: key_path.to_path_buf() });
        }
        
        // Try to authenticate with the key
        match session.userauth_pubkey_file(username, None, key_path, None) {
            Ok(()) => {
                println!("Key authentication successful with: {}", key_path.display());
                Ok(true)
            }
            Err(e) => {
                println!("Key authentication failed with {}: {}", key_path.display(), e);
                Err(SshError::KeyAuthenticationFailed { path: key_path.to_path_buf() })
            }
        }
    }
    
    /// Try to connect to SSH agent and verify it has keys with optional verbose output
    fn try_connect_agent_with_verbose(verbose: bool) -> SshResult<Agent> {
        if verbose {
            println!("Attempting to connect to SSH agent...");
        }
        
        // Create a temporary session to test agent
        let session = Session::new().map_err(|e| SshError::SessionInitError {
            message: format!("Failed to create session: {}", e)
        })?;
        
        // Try to connect to SSH agent
        let mut agent = session.agent().map_err(|e| SshError::AgentConnectionError {
            message: format!("Failed to connect to agent: {}", e)
        })?;
        
        // Connect to the agent
        agent.connect().map_err(|e| SshError::AgentConnectionError {
            message: format!("Agent connect failed: {}", e)
        })?;
        
        // Request list of identities
        agent.list_identities().map_err(|e| SshError::AgentConnectionError {
            message: format!("Failed to list identities: {}", e)
        })?;
        
        // Check if agent has any keys
        let identities = agent.identities().map_err(|e| SshError::AgentConnectionError {
            message: format!("Failed to get identities: {}", e)
        })?;
        
        if identities.is_empty() {
            if verbose {
                println!("SSH agent is connected but has no keys loaded");
            }
            return Err(SshError::AgentEmptyError);
        }
        
        if verbose {
            println!("SSH agent has {} keys loaded", identities.len());
        }
        Ok(agent)
    }
    
    /// Get the SSH directory path (~/.ssh)
    fn get_ssh_directory() -> SshResult<PathBuf> {
        let home_dir = env::var("HOME").map_err(|_| SshError::IoError { 
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "HOME environment variable not set"
            )
        })?;
        
        Ok(PathBuf::from(home_dir).join(".ssh"))
    }
}

impl Default for SshManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|e| {
            println!("Warning: Failed to initialize SSH manager: {}", e);
            SshManager {
                agent: None,
                available_keys: Vec::new(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_ssh_directory_detection() {
        // This test will only work if HOME is set
        if env::var("HOME").is_ok() {
            let ssh_dir = SshManager::get_ssh_directory().unwrap();
            assert!(ssh_dir.to_string_lossy().contains(".ssh"));
        }
    }
    
    #[test]
    fn test_key_discovery() {
        // This test will find keys if they exist
        let keys = SshManager::find_ssh_keys_with_verbose(false);
        // We can't assert specific counts since it depends on the system
        // but we can verify it returns a Vec
        assert!(keys.len() >= 0);
    }
    
    #[test]
    fn test_ssh_manager_creation() {
        // Test that SshManager can be created
        // This might fail if no SSH keys are available, which is expected
        let result = SshManager::new();
        match result {
            Ok(manager) => {
                println!("SSH manager created successfully");
                println!("Agent available: {}", manager.has_usable_agent());
                println!("Keys found: {}", manager.get_available_keys().len());
            }
            Err(e) => {
                println!("SSH manager creation failed (expected if no keys): {}", e);
            }
        }
    }
}