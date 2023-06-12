use async_trait::async_trait;
use cryfs_rustfs::{object_based_api::Symlink, AbsolutePathBuf, FsError, FsResult};

use super::errors::IoResultExt;

pub struct PassthroughSymlink {
    path: AbsolutePathBuf,
}

impl PassthroughSymlink {
    pub fn new(path: AbsolutePathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Symlink for PassthroughSymlink {
    async fn target(&self) -> FsResult<String> {
        let target = tokio::fs::read_link(&self.path).await.map_error()?;
        let target = target
            .as_os_str()
            .to_str()
            .ok_or_else(|| {
                log::error!("Symlink target is not valid UTF-8: {:?}", target);
                FsError::InvalidPath
            })?
            .to_owned();
        Ok(target)
    }
}
