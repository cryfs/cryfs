use anyhow::Result;
use async_trait::async_trait;

use super::data_tree_store::DataTree;
use crate::blobstore::{Blob, BlobId};
use crate::blockstore::low_level::BlockStore;
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

#[derive(Debug)]
pub struct BlobOnBlocks<B: BlockStore + Send + Sync> {
    // Always Some unless during destruction
    tree: Option<AsyncDropGuard<DataTree<B>>>,
}

impl<B: BlockStore + Send + Sync> BlobOnBlocks<B> {
    pub(super) fn new(tree: AsyncDropGuard<DataTree<B>>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { tree: Some(tree) })
    }

    fn _tree(&self) -> &AsyncDropGuard<DataTree<B>> {
        self.tree.as_ref().expect("BlobOnBlocks.tree is None")
    }

    fn _tree_mut(&mut self) -> &mut AsyncDropGuard<DataTree<B>> {
        self.tree.as_mut().expect("BlobOnBlocks.tree is None")
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync> Blob for BlobOnBlocks<B> {
    fn id(&self) -> BlobId {
        BlobId {
            root: *self._tree().root_node_id(),
        }
    }

    async fn num_bytes(&mut self) -> Result<u64> {
        self._tree_mut().num_bytes().await
    }

    async fn resize(&mut self, new_num_bytes: u64) -> Result<()> {
        self._tree_mut().resize_num_bytes(new_num_bytes).await
    }

    async fn read_all(&mut self) -> Result<Data> {
        self._tree_mut().read_all().await
    }

    async fn read(&mut self, target: &mut [u8], offset: u64) -> Result<()> {
        self._tree_mut().read_bytes(offset, target).await
    }

    async fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize> {
        self._tree_mut().try_read_bytes(offset, target).await
    }

    async fn write(&mut self, source: &[u8], offset: u64) -> Result<()> {
        self._tree_mut().write_bytes(source, offset).await
    }

    async fn flush(&mut self) -> Result<()> {
        self._tree_mut().flush().await
    }

    async fn num_nodes(&mut self) -> Result<u64> {
        self._tree_mut().num_nodes().await
    }

    async fn remove(mut this: AsyncDropGuard<Self>) -> Result<()> {
        let tree = this.tree.take().expect("BlobOnBlocks.tree is None");
        DataTree::remove(tree).await
        // no call to async_drop needed since we moved out of this.tree
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync> AsyncDrop for BlobOnBlocks<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self._tree_mut().async_drop().await
    }
}
