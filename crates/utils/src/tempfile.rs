use anyhow::Result;
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn create(path: PathBuf) -> Result<Self> {
        File::create(&path)?;
        Ok(Self { path })
    }

    pub async fn create_async(path: PathBuf) -> Result<Self> {
        tokio::fs::File::create(&path).await?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.path).unwrap();
    }
}
