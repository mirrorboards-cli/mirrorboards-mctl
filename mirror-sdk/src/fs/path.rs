//! Path handling utilities for the Mirror SDK.

use std::path::{Path, PathBuf};
use path_absolutize::Absolutize;

use crate::error::MirrorError;

/// Resolves a path to an absolute path.
/// 
/// If the path is already absolute, it is returned as is.
/// If the path is relative, it is resolved relative to the current working directory.
pub fn resolve_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, MirrorError> {
    let path_ref = path.as_ref();
    
    if path_ref.is_absolute() {
        Ok(path_ref.to_path_buf())
    } else {
        path_ref.absolutize()
            .map(|p| p.to_path_buf())
            .map_err(|e| MirrorError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Failed to resolve path: {}", e)
            )))
    }
}

/// Normalizes a path by resolving it to an absolute path and cleaning it.
/// 
/// This function:
/// 1. Resolves the path to an absolute path
/// 2. Removes any redundant components (like "." and "..")
/// 3. Removes trailing slashes
pub fn normalize_path<P: AsRef<Path>>(path: P) -> Result<PathBuf, MirrorError> {
    let abs_path = resolve_path(path)?;
    
    // Clean the path (remove redundant components like "." and "..")
    let mut components = Vec::new();
    
    for component in abs_path.components() {
        use std::path::Component;
        
        match component {
            Component::Prefix(_) | Component::RootDir => {
                components.push(component);
            },
            Component::CurDir => {
                // Skip "." components
            },
            Component::ParentDir => {
                // Handle ".." by removing the last normal component
                if components.len() > 0 &&
                   !matches!(components.last(), Some(Component::ParentDir) | Some(Component::RootDir)) {
                    components.pop();
                } else {
                    components.push(component);
                }
            },
            Component::Normal(_) => {
                components.push(component);
            },
        }
    }
    
    // Rebuild the path from the normalized components
    let clean_path = components.iter().fold(PathBuf::new(), |mut path, &component| {
        path.push(component);
        path
    });
    
    Ok(clean_path)
}

/// Checks if a path exists.
pub fn path_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

/// Checks if a path is a directory.
pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Checks if a path is a file.
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Creates a directory and all its parent directories if they don't exist.
pub fn create_dir_all<P: AsRef<Path>>(path: P) -> Result<(), MirrorError> {
    std::fs::create_dir_all(path.as_ref())
        .map_err(|e| MirrorError::Io(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_path() {
        // Absolute path
        let abs_path = if cfg!(windows) {
            PathBuf::from(r"C:\path\to\file")
        } else {
            PathBuf::from("/path/to/file")
        };
        
        let resolved = resolve_path(&abs_path).unwrap();
        assert_eq!(resolved, abs_path);
        
        // Relative path (this test is environment-dependent)
        // Just check that it doesn't fail and returns an absolute path
        let rel_path = PathBuf::from("relative/path");
        let resolved = resolve_path(&rel_path).unwrap();
        assert!(resolved.is_absolute());
    }

    #[test]
    fn test_normalize_path() {
        // Create a temporary directory for testing
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        
        // Create a path with redundant components
        let path = base_path.join("a").join("..").join("b").join(".");
        
        let normalized = normalize_path(&path).unwrap();
        let expected = base_path.join("b");
        
        // Compare the string representation because the actual PathBuf might differ
        // in how it's internally represented
        assert_eq!(normalized.to_string_lossy(), expected.to_string_lossy());
    }

    #[test]
    fn test_path_exists() {
        let dir = tempdir().unwrap();
        assert!(path_exists(dir.path()));
        
        let nonexistent = dir.path().join("nonexistent");
        assert!(!path_exists(&nonexistent));
    }

    #[test]
    fn test_is_directory() {
        let dir = tempdir().unwrap();
        assert!(is_directory(dir.path()));
        
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, b"test").unwrap();
        assert!(!is_directory(&file_path));
    }

    #[test]
    fn test_is_file() {
        let dir = tempdir().unwrap();
        assert!(!is_file(dir.path()));
        
        let file_path = dir.path().join("file.txt");
        std::fs::write(&file_path, b"test").unwrap();
        assert!(is_file(&file_path));
    }

    #[test]
    fn test_create_dir_all() {
        let dir = tempdir().unwrap();
        let nested_dir = dir.path().join("a").join("b").join("c");
        
        create_dir_all(&nested_dir).unwrap();
        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());
    }
}