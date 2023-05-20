use async_trait::async_trait;
use cryfs_rustfs::{object_based_api::Symlink, FsResult};
use std::path::PathBuf;

use super::errors::IoResultExt;

pub struct PassthroughSymlink {
    path: PathBuf,
}

impl PassthroughSymlink {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Symlink for PassthroughSymlink {
    async fn target(&self) -> FsResult<PathBuf> {
        let target = tokio::fs::read_link(&self.path).await.map_error()?;
        Ok(target)
    }
}
