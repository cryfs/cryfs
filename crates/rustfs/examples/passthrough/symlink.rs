use async_trait::async_trait;
use cryfs_rustfs::{AbsolutePathBuf, FsError, FsResult, object_based_api::Symlink};
use cryfs_utils::async_drop::AsyncDropGuard;

use super::device::PassthroughDevice;
use super::errors::IoResultExt;
use super::node::PassthroughNode;

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
    type Device = PassthroughDevice;

    fn as_node(&self) -> AsyncDropGuard<PassthroughNode> {
        PassthroughNode::new(self.path.clone())
    }

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
