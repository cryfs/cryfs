use async_trait::async_trait;
use cryfs_rustfs::{AbsolutePathBuf, FsError, FsResult, object_based_api::Symlink};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use super::device::PassthroughDevice;
use super::errors::IoResultExt;
use super::node::PassthroughNode;

#[derive(Debug)]
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

    fn into_node(this: AsyncDropGuard<Self>) -> AsyncDropGuard<PassthroughNode> {
        PassthroughNode::new(this.unsafe_into_inner_dont_drop().path.clone())
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

#[async_trait]
impl AsyncDrop for PassthroughSymlink {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        // Nothing to do
        Ok(())
    }
}
