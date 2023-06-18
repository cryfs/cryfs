use anyhow::Result;
use async_trait::async_trait;
use futures::stream::BoxStream;

use crate::{
    low_level::interface::{BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter},
    BlockId, RemoveResult, TryCreateResult,
};
use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::data::Data;

#[async_trait]
impl BlockStoreReader for Box<dyn BlockStore + Sync + Send> {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        let r = (**self).exists(id);
        r.await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let r = (**self).load(id);
        r.await
    }

    async fn num_blocks(&self) -> Result<u64> {
        let r = (**self).num_blocks();
        r.await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        (**self).estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        (**self).block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        let r = (**self).all_blocks();
        r.await
    }
}

#[async_trait]
impl BlockStoreWriter for Box<dyn BlockStore + Sync + Send> {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult> {
        let r = (**self).try_create(id, data);
        r.await
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        let r = (**self).store(id, data);
        r.await
    }
}

#[async_trait]
impl BlockStoreDeleter for Box<dyn BlockStore + Sync + Send> {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        let r = (**self).remove(id);
        r.await
    }
}

#[async_trait]
impl AsyncDrop for Box<dyn BlockStore + Sync + Send> {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let r = (**self).async_drop_impl();
        let r = r.await?;
        Ok(r)
    }
}

impl BlockStore for Box<dyn BlockStore + Sync + Send> {}
