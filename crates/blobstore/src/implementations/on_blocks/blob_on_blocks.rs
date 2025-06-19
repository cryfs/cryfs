use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;
use std::fmt::Debug;

use super::data_tree_store::DataTree;
use crate::{Blob, BlobId};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

#[derive(Debug)]
pub struct BlobOnBlocks<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> {
    // Always Some unless during destruction
    tree: AsyncDropGuard<DataTree<B>>,
}

impl<'a, B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> BlobOnBlocks<B> {
    pub(super) fn new(tree: AsyncDropGuard<DataTree<B>>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { tree })
    }

    fn _tree(&self) -> &DataTree<B> {
        &*self.tree
    }

    fn _tree_mut(&mut self) -> &mut DataTree<B> {
        &mut *self.tree
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn into_data_tree(this: AsyncDropGuard<Self>) -> AsyncDropGuard<DataTree<B>> {
        this.unsafe_into_inner_dont_drop().tree
    }
}

#[async_trait]
impl<B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync> Blob for BlobOnBlocks<B> {
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

    async fn remove(this: AsyncDropGuard<Self>) -> Result<()> {
        let tree = this.unsafe_into_inner_dont_drop().tree;
        DataTree::remove(tree).await
        // no call to async_drop needed since we moved out of this
    }

    fn all_blocks(&self) -> Result<BoxStream<'_, Result<BlockId>>> {
        self._tree().all_blocks()
    }
}

#[async_trait]
impl<B> AsyncDrop for BlobOnBlocks<B>
where
    B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync,
{
    type Error = <B as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.tree.async_drop().await
    }
}
