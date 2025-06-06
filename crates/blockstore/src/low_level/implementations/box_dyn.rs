use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;

use crate::{
    BlockId, RemoveResult, TryCreateResult,
    low_level::{
        InvalidBlockSizeError,
        interface::{BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, LLBlockStore},
    },
};
use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::data::Data;

#[derive(Debug)]
pub struct DynBlockStore(pub Box<dyn LLBlockStore + Sync + Send>);

#[async_trait]
impl BlockStoreReader for DynBlockStore {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        let r = (*self.0).exists(id);
        r.await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let r = (*self.0).load(id);
        r.await
    }

    async fn num_blocks(&self) -> Result<u64> {
        let r = (*self.0).num_blocks();
        r.await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        (*self.0).estimate_num_free_bytes()
    }

    fn usable_block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        (*self.0).usable_block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        let r = (*self.0).all_blocks();
        r.await
    }
}

#[async_trait]
impl BlockStoreWriter for DynBlockStore {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult> {
        let r = (*self.0).try_create(id, data);
        r.await
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        let r = (*self.0).store(id, data);
        r.await
    }
}

#[async_trait]
impl BlockStoreDeleter for DynBlockStore {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        let r = (*self.0).remove(id);
        r.await
    }
}

#[async_trait]
impl AsyncDrop for DynBlockStore {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let r = (*self.0).async_drop_impl();
        let r = r.await?;
        Ok(r)
    }
}

impl LLBlockStore for DynBlockStore {}
