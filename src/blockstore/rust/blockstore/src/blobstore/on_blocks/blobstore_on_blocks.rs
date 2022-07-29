use anyhow::Result;
use async_trait::async_trait;
use std::fmt;

use super::blob_on_blocks::BlobOnBlocks;
use super::data_node_store::DataNodeStore;
use crate::blobstore::{BlobId, BlobStore, RemoveResult};
use crate::blockstore::{high_level::LockingBlockStore, low_level::BlockStore};
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct BlobStoreOnBlocks<B: BlockStore + Send + Sync> {
    node_store: AsyncDropGuard<DataNodeStore<B>>,
}

impl<B: BlockStore + Send + Sync> BlobStoreOnBlocks<B> {
    pub fn new(
        blockstore: AsyncDropGuard<LockingBlockStore<B>>,
        block_size_bytes: u32,
    ) -> Result<AsyncDropGuard<Self>> {
        Ok(AsyncDropGuard::new(Self {
            node_store: DataNodeStore::new(blockstore, block_size_bytes)?,
        }))
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync> BlobStore for BlobStoreOnBlocks<B> {
    type ConcreteBlob = BlobOnBlocks<B>;

    async fn create(&self) -> Result<Self::ConcreteBlob> {
        todo!()
    }

    async fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob>> {
        todo!()
    }

    async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult> {
        todo!()
    }

    async fn num_nodes(&self) -> Result<u64> {
        todo!()
    }

    fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        todo!()
    }

    fn virtual_block_size_bytes(&self) -> Result<u64> {
        todo!()
    }
}

impl<B: BlockStore + Send + Sync> fmt::Debug for BlobStoreOnBlocks<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStoreOnBlocks")
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync> AsyncDrop for BlobStoreOnBlocks<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.node_store.async_drop().await
    }
}
