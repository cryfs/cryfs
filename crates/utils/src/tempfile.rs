//! Temporary file management with automatic cleanup.
//!
//! This module provides [`TempFile`], an RAII wrapper that creates a temporary file
//! and automatically deletes it when dropped.

use anyhow::Result;
use std::fs::File;
use std::path::{Path, PathBuf};

/// A temporary file that is automatically deleted when dropped.
///
/// `TempFile` provides RAII-style management for temporary files. When you create
/// a `TempFile`, it creates an empty file at the specified path. When the `TempFile`
/// is dropped, the file is automatically deleted.
///
/// # Panics
///
/// The destructor will panic if the file cannot be deleted (e.g., if it was already
/// deleted or permissions changed).
///
/// # Examples
///
/// ```no_run
/// use cryfs_utils::tempfile::TempFile;
/// use std::path::PathBuf;
///
/// let temp = TempFile::create(PathBuf::from("/tmp/my_temp_file")).unwrap();
/// // Use temp.path() to get the file path
/// // File is automatically deleted when `temp` goes out of scope
/// ```
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    /// Creates a new temporary file at the specified path.
    ///
    /// This creates an empty file at `path`. The file will be automatically
    /// deleted when the `TempFile` is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created (e.g., parent directory
    /// doesn't exist, permission denied).
    pub fn create(path: PathBuf) -> Result<Self> {
        File::create(&path)?;
        Ok(Self { path })
    }

    /// Creates a new temporary file at the specified path asynchronously.
    ///
    /// This is the async version of [`Self::create`].
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created.
    pub async fn create_async(path: PathBuf) -> Result<Self> {
        tokio::fs::File::create(&path).await?;
        Ok(Self { path })
    }

    /// Returns the path to the temporary file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.path).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_create_file_exists() {
        let dir = TempDir::new("tempfile_test").unwrap();
        let path = dir.path().join("test_file");

        let temp = TempFile::create(path.clone()).unwrap();

        assert!(path.exists());
        assert!(temp.path().exists());
    }

    #[tokio::test]
    async fn test_create_async_file_exists() {
        let dir = TempDir::new("tempfile_test_async").unwrap();
        let path = dir.path().join("test_file_async");

        let temp = TempFile::create_async(path.clone()).await.unwrap();

        assert!(path.exists());
        assert!(temp.path().exists());
    }

    #[test]
    fn test_path_returns_correct_path() {
        let dir = TempDir::new("tempfile_test_path").unwrap();
        let path = dir.path().join("test_file");

        let temp = TempFile::create(path.clone()).unwrap();

        assert_eq!(path, temp.path());
    }

    #[test]
    fn test_file_deleted_on_drop() {
        let dir = TempDir::new("tempfile_test_drop").unwrap();
        let path = dir.path().join("test_file");

        {
            let _temp = TempFile::create(path.clone()).unwrap();
            assert!(path.exists());
        }

        // File should be deleted after TempFile is dropped
        assert!(!path.exists());
    }
}
