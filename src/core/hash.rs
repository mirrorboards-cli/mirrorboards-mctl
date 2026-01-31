//! SHA256 hash generation for repository identification.

use sha2::{Digest, Sha256};

/// Generate a SHA256 hash for a git URL.
///
/// This is used to create unique identifiers for repositories
/// that are filesystem-safe.
pub fn generate_repo_hash(git_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(git_url.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

/// Generate a short hash (first 8 characters) for display purposes.
pub fn generate_short_hash(git_url: &str) -> String {
    let full_hash = generate_repo_hash(git_url);
    full_hash[..8].to_string()
}

/// Hex encoding without external crate
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_repo_hash() {
        let hash = generate_repo_hash("git@github.com:test/repo.git");
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_hash_consistency() {
        let url = "git@github.com:test/repo.git";
        let hash1 = generate_repo_hash(url);
        let hash2 = generate_repo_hash(url);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_urls_different_hashes() {
        let hash1 = generate_repo_hash("git@github.com:test/repo1.git");
        let hash2 = generate_repo_hash("git@github.com:test/repo2.git");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_short_hash() {
        let short = generate_short_hash("git@github.com:test/repo.git");
        assert_eq!(short.len(), 8);
    }
}
