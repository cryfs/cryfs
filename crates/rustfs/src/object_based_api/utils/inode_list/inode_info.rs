use async_trait::async_trait;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};
use std::fmt::Debug;

use crate::FsResult;
use crate::{FsError, InodeNumber, object_based_api::Device};

#[derive(Debug)]
pub struct InodeInfo<Fs>
where
    Fs: Device + Debug,
{
    node: AsyncDropGuard<AsyncDropArc<Fs::Node>>,
    parent_inode: InodeNumber,
}

impl<Fs> InodeInfo<Fs>
where
    Fs: Device + Debug,
{
    pub fn new(
        node: AsyncDropGuard<AsyncDropArc<Fs::Node>>,
        parent_inode: InodeNumber,
    ) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { node, parent_inode })
    }

    pub fn parent_inode(&self) -> InodeNumber {
        self.parent_inode
    }

    pub fn node(&self) -> AsyncDropGuard<AsyncDropArc<Fs::Node>> {
        AsyncDropArc::clone(&self.node)
    }

    #[cfg(feature = "testutils")]
    pub async fn fsync(&self) -> FsResult<()> {
        use crate::object_based_api::Node as _;

        self.node.fsync(false).await
    }
}

#[async_trait]
impl<Fs> AsyncDrop for InodeInfo<Fs>
where
    Fs: Device + Debug,
{
    type Error = FsError;
    async fn async_drop_impl(&mut self) -> FsResult<()> {
        self.node.async_drop().await?;
        Ok(())
    }
}
