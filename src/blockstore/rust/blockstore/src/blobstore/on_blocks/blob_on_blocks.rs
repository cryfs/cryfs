use anyhow::Result;
use async_trait::async_trait;

use super::data_tree_store::DataTree;
use crate::blobstore::{Blob, BlobId};
use crate::blockstore::low_level::BlockStore;
use crate::data::Data;

pub struct BlobOnBlocks<B: BlockStore + Send + Sync> {
    tree: DataTree<B>,
}

#[async_trait]
impl<B: BlockStore + Send + Sync> Blob for BlobOnBlocks<B> {
    fn id(&self) -> BlobId {
        BlobId {
            root: *self.tree.root_node_id(),
        }
    }

    async fn num_bytes(&mut self) -> Result<u64> {
        self.tree.num_bytes().await
    }

    async fn resize(&mut self, new_num_bytes: u64) -> Result<()> {
        self.tree.resize_num_bytes(new_num_bytes).await
    }

    async fn read_all(&mut self) -> Result<Data> {
        self.tree.read_all().await
    }

    async fn read(&mut self, target: &mut [u8], offset: u64) -> Result<()> {
        self.tree.read_bytes(offset, target).await
    }

    async fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize> {
        self.tree.try_read_bytes(offset, target).await
    }

    async fn write(&mut self, source: &[u8], offset: u64) -> Result<()> {
        self.tree.write_bytes(source, offset).await
    }

    async fn flush(&mut self) -> Result<()> {
        self.tree.flush().await
    }

    async fn num_nodes(&mut self) -> Result<u64> {
        self.tree.num_nodes().await
    }

    async fn remove(self) -> Result<()> {
        todo!()
    }
}
