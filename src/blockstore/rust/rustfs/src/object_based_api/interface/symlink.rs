use async_trait::async_trait;
use std::path::PathBuf;

use crate::common::FsResult;

#[async_trait]
pub trait Symlink {
    async fn target(&self) -> FsResult<PathBuf>;
}
