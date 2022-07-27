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

    async fn size(&mut self) -> Result<u64> {
        self.tree.num_bytes().await
    }

    // fn read_all(&self) -> Data {
    //     todo!()
    // }

    // fn read(&self, target: &mut [u8], offset: u64, size: u64) -> Result<()> {
    //     todo!()
    // }

    // fn try_read(&self, target: &mut [u8], offset: u64, size: u64) -> Result<()> {
    //     todo!()
    // }

    // fn write(&self, source: &[u8], offset: u64, size: u64) -> Result<()> {
    //     todo!()
    // }

    // fn flush(&self) -> Result<()> {
    //     todo!()
    // }

    // fn num_nodes(&self) -> Result<()> {
    //     todo!()
    // }
}
