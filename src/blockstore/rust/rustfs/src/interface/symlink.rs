use async_trait::async_trait;

use super::error::FsResult;
use std::path::PathBuf;

#[async_trait]
pub trait Symlink {
    async fn target(&self) -> FsResult<PathBuf>;
}
