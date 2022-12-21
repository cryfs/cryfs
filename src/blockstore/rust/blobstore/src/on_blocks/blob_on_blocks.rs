use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;

use super::data_tree_store::DataTree;
use crate::{Blob, BlobId};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_utils::data::Data;

#[derive(Debug)]
pub struct BlobOnBlocks<'a, B: BlockStore + Send + Sync> {
    // Always Some unless during destruction
    tree: Option<DataTree<'a, B>>,
}

impl<'a, B: BlockStore + Send + Sync> BlobOnBlocks<'a, B> {
    pub(super) fn new(tree: DataTree<'a, B>) -> Self {
        Self { tree: Some(tree) }
    }

    fn _tree(&self) -> &DataTree<'a, B> {
        self.tree.as_ref().expect("BlobOnBlocks.tree is None")
    }

    fn _tree_mut(&mut self) -> &mut DataTree<'a, B> {
        self.tree.as_mut().expect("BlobOnBlocks.tree is None")
    }
}

#[async_trait]
impl<'a, B: BlockStore + Send + Sync> Blob<'a> for BlobOnBlocks<'a, B> {
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

    async fn remove(mut self) -> Result<()> {
        let tree = self.tree.take().expect("BlobOnBlocks.tree is None");
        DataTree::remove(tree).await
        // no call to async_drop needed since we moved out of this
    }

    async fn all_blocks(&self) -> Result<Box<dyn Stream<Item = Result<BlockId>> + Unpin + '_>> {
        self._tree().all_blocks().await
    }
}
