use async_trait::async_trait;
use std::fmt::Debug;
use std::path::Path;

use super::{device::CryDevice, node::CryNode, open_file::CryOpenFile};
use cryfs_blobstore::BlobStore;
use cryfs_rustfs::{object_based_api::Dir, DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, Uid};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    node: CryNode<'a, B>,
}

impl<'a, B> CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    pub fn new(node: CryNode<'a, B>) -> Self {
        Self { node }
    }
}

#[async_trait]
impl<'a, B> Dir for CryDir<'a, B>
where
    B: BlobStore + AsyncDrop<Error = anyhow::Error> + Debug + Send + Sync + 'static,
    for<'b> <B as BlobStore>::ConcreteBlob<'b>: Send + Sync,
{
    type Device = CryDevice<B>;

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn create_child_dir(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn remove_child_dir(&self, name: &str) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn create_child_symlink(
        &self,
        name: &str,
        target: &Path,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn remove_child_file_or_symlink(&self, name: &str) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn create_and_open_file(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<CryOpenFile<B>>)> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }

    async fn rename_child(&self, old_name: &str, new_path: &Path) -> FsResult<()> {
        // TODO Implement
        Err(FsError::NotImplemented)
    }
}
