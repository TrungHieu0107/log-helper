//! File system utility functions.
//! 
//! Provides simple wrappers around std::fs for common file operations.

use std::path::Path;

/// Check if a file exists at the given path.
pub fn file_exists(path: &str) -> bool {
    let p = Path::new(path);
    p.exists() && p.is_file()
}

/// Check if a directory exists at the given path.
pub fn directory_exists(path: &str) -> bool {
    let p = Path::new(path);
    p.exists() && p.is_dir()
}

/// Create a directory and all parent directories if they don't exist.
pub fn create_directory(path: &str) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}

/// Get the filename from a path.
pub fn get_file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
}

/// Get the parent directory from a path.
pub fn get_directory(path: &str) -> Option<String> {
    Path::new(path)
        .parent()
        .map(|p| p.to_string_lossy().into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_file_name() {
        assert_eq!(get_file_name("C:\\path\\to\\file.txt"), Some("file.txt".to_string()));
        assert_eq!(get_file_name("/path/to/file.txt"), Some("file.txt".to_string()));
    }

    #[test]
    fn test_get_directory() {
        assert_eq!(get_directory("C:\\path\\to\\file.txt"), Some("C:\\path\\to".to_string()));
    }
}
