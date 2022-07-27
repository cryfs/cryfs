use anyhow::Result;

use super::blob_on_blocks::BlobOnBlocks;
use super::data_node_store::DataNodeStore;
use crate::blobstore::{BlobId, BlobStore};
use crate::blockstore::low_level::BlockStore;

pub struct BlobStoreOnBlocks<B: BlockStore + Send + Sync> {
    node_store: DataNodeStore<B>,
}

impl<B: BlockStore + Send + Sync> BlobStore for BlobStoreOnBlocks<B> {
    type ConcreteBlob = BlobOnBlocks<B>;

    // fn create(&self) -> Result<Self::ConcreteBlob> {
    //     todo!()
    // }

    fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob>> {
        todo!()
    }

    // fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult> {
    //     todo!()
    // }

    // fn num_blocks(&self) -> Result<u64> {
    //     todo!()
    // }

    // fn estimate_space_for_num_blocks_left() -> Result<u64> {
    //     todo!()
    // }

    // //virtual means "space we can use" as opposed to "space it takes on the disk" (i.e. virtual is without headers, checksums, ...)
    // fn virtual_blocksize_bytes() -> Result<u64> {
    //     todo!()
    // }
}
