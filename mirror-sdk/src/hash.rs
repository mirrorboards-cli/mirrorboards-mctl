use crate::models::Repository;
use crate::error::{HashError, HashResult};
use sha2::{Sha256, Digest};
use std::collections::HashMap;

/// Generate a unique hash for a repository based on all its metadata
pub fn generate_hash(repo: &Repository) -> String {
    let mut hasher = Sha256::new();
    
    // Normalize and concatenate all fields in a deterministic order
    let normalized_data = format!(
        "{}|{}|{}|{}",
        repo.git.trim(),
        repo.path.trim(),
        repo.branch.trim(),
        repo.skip_push
    );
    
    hasher.update(normalized_data.as_bytes());
    let result = hasher.finalize();
    
    // Convert to hex string and take first 8 characters for user-friendly IDs
    format!("{:x}", result)[..8].to_string()
}

/// Generate a longer hash if collision is detected
pub fn generate_extended_hash(repo: &Repository, length: usize) -> String {
    let mut hasher = Sha256::new();
    
    let normalized_data = format!(
        "{}|{}|{}|{}",
        repo.git.trim(),
        repo.path.trim(),
        repo.branch.trim(),
        repo.skip_push
    );
    
    hasher.update(normalized_data.as_bytes());
    let result = hasher.finalize();
    let hex_string = format!("{:x}", result);
    
    // Take the requested length, up to the full hash length
    let actual_length = std::cmp::min(length, hex_string.len());
    hex_string[..actual_length].to_string()
}

/// Verify that a hash matches a repository
pub fn verify_hash(repo: &Repository, hash: &str) -> bool {
    let computed_hash = generate_hash(repo);
    computed_hash.starts_with(hash)
}

/// Check for hash collisions in a collection of repositories
pub fn check_collisions(repositories: &[Repository]) -> HashResult<()> {
    let mut hash_map: HashMap<String, &Repository> = HashMap::new();
    
    for repo in repositories {
        let hash = generate_hash(repo);
        
        if let Some(existing_repo) = hash_map.get(&hash) {
            // We have a collision - this is very unlikely with SHA256 but we should handle it
            if existing_repo.git != repo.git {
                return Err(HashError::Collision { 
                    git: repo.git.clone() 
                });
            }
        } else {
            hash_map.insert(hash, repo);
        }
    }
    
    Ok(())
}

/// Find the minimum hash length needed to uniquely identify all repositories
pub fn find_minimum_hash_length(repositories: &[Repository]) -> usize {
    if repositories.len() <= 1 {
        return 4; // Minimum reasonable length
    }
    
    for length in 4..=64 {
        let mut hash_set = std::collections::HashSet::new();
        let mut unique = true;
        
        for repo in repositories {
            let hash = generate_extended_hash(repo, length);
            if !hash_set.insert(hash) {
                unique = false;
                break;
            }
        }
        
        if unique {
            return length;
        }
    }
    
    64 // Fallback to full hash length
}

/// Validate hash format (hex string of appropriate length)
pub fn validate_hash_format(hash: &str) -> HashResult<()> {
    if hash.is_empty() {
        return Err(HashError::InvalidFormat { 
            hash: hash.to_string() 
        });
    }
    
    if hash.len() < 4 || hash.len() > 64 {
        return Err(HashError::InvalidFormat { 
            hash: hash.to_string() 
        });
    }
    
    // Check if all characters are valid hex
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(HashError::InvalidFormat { 
            hash: hash.to_string() 
        });
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Repository;

    fn create_test_repo(git: &str, path: &str) -> Repository {
        Repository::new(
            git.to_string(),
            path.to_string(),
            Some("main".to_string()),
            Some(false),
        )
    }

    #[test]
    fn test_generate_hash() {
        let repo1 = create_test_repo("git@github.com:org/repo1.git", "org/repo1");
        let repo2 = create_test_repo("git@github.com:org/repo2.git", "org/repo2");
        
        let hash1 = generate_hash(&repo1);
        let hash2 = generate_hash(&repo2);
        
        assert_eq!(hash1.len(), 8);
        assert_eq!(hash2.len(), 8);
        assert_ne!(hash1, hash2);
        
        // Same repo should generate same hash
        let hash1_again = generate_hash(&repo1);
        assert_eq!(hash1, hash1_again);
    }
    
    #[test]
    fn test_verify_hash() {
        let repo = create_test_repo("git@github.com:org/repo.git", "org/repo");
        let hash = generate_hash(&repo);
        
        assert!(verify_hash(&repo, &hash));
        assert!(verify_hash(&repo, &hash[..4])); // Partial match
        assert!(!verify_hash(&repo, "invalid"));
    }
    
    #[test]
    fn test_hash_consistency() {
        let repo1 = create_test_repo("git@github.com:org/repo.git", "org/repo");
        let mut repo2 = repo1.clone();
        
        // Same repos should have same hash
        assert_eq!(generate_hash(&repo1), generate_hash(&repo2));
        
        // Different branch should produce different hash
        repo2.branch = "develop".to_string();
        assert_ne!(generate_hash(&repo1), generate_hash(&repo2));
        
        // Different skip_push should produce different hash
        repo2 = repo1.clone();
        repo2.skip_push = true;
        assert_ne!(generate_hash(&repo1), generate_hash(&repo2));
    }
    
    #[test]
    fn test_minimum_hash_length() {
        let repos = vec![
            create_test_repo("git@github.com:org/repo1.git", "org/repo1"),
            create_test_repo("git@github.com:org/repo2.git", "org/repo2"),
            create_test_repo("git@github.com:org/repo3.git", "org/repo3"),
        ];
        
        let min_length = find_minimum_hash_length(&repos);
        assert!(min_length >= 4);
        assert!(min_length <= 64);
    }
    
    #[test]
    fn test_validate_hash_format() {
        assert!(validate_hash_format("abcd1234").is_ok());
        assert!(validate_hash_format("abc").is_err()); // Too short
        assert!(validate_hash_format("").is_err()); // Empty
        assert!(validate_hash_format("ghijk").is_err()); // Invalid hex
        assert!(validate_hash_format("abcd efgh").is_err()); // Contains space
    }
    
    #[test]
    fn test_extended_hash() {
        let repo = create_test_repo("git@github.com:org/repo.git", "org/repo");
        
        let hash_8 = generate_extended_hash(&repo, 8);
        let hash_16 = generate_extended_hash(&repo, 16);
        
        assert_eq!(hash_8.len(), 8);
        assert_eq!(hash_16.len(), 16);
        assert!(hash_16.starts_with(&hash_8));
    }
}