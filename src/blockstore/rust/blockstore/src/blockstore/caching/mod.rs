use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use std::num::NonZeroUsize;
use std::pin::Pin;

use super::{BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter};

use super::block_data::IBlockData;
use crate::data::Data;

mod cache;
use cache::Cache;

const MAX_NUM_CACHE_ENTRIES: NonZeroUsize = unsafe {
    // TODO Use ::new().unwrap() instead of unsafe once that is const
    NonZeroUsize::new_unchecked(1000)
};

pub struct CachingBlockStore<C, B> {
    underlying_block_store: B,
    cache: C,
}

impl<C: Cache<BlockId, Data>, B> CachingBlockStore<C, B> {
    pub fn new(underlying_block_store: B) -> Self {
        Self {
            underlying_block_store,
            cache: C::new(MAX_NUM_CACHE_ENTRIES, todo!()),
        }
    }
}

#[async_trait]
impl<C: Cache<BlockId, Data> + Send + Sync, B: BlockStoreReader + Send + Sync> BlockStoreReader
    for CachingBlockStore<C, B>
{
    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        todo!()
        // let loaded = self.underlying_block_store.load(id).await?;
        // match loaded {
        //     None => Ok(None),
        //     Some(data) => Ok(Some(self._decrypt(data).await?)),
        // }
    }

    async fn num_blocks(&self) -> Result<u64> {
        todo!()
        //self.underlying_block_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        todo!()
        // self.underlying_block_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        todo!()
        // let ciphertext_size = block_size.checked_sub(FORMAT_VERSION_HEADER.len() as u64)
        //     .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, FORMAT_VERSION_HEADER.len()))?;
        // ciphertext_size
        //     .checked_sub(C::CIPHERTEXT_OVERHEAD as u64)
        //     .with_context(|| anyhow!("Physical block size of {} is too small.", block_size))
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        todo!()
        // self.underlying_block_store.all_blocks().await
    }
}

#[async_trait]
impl<C: Cache<BlockId, Data> + Send + Sync, B: BlockStoreDeleter + Send + Sync> BlockStoreDeleter
    for CachingBlockStore<C, B>
{
    async fn remove(&self, id: &BlockId) -> Result<bool> {
        todo!()
        // self.underlying_block_store.remove(id).await
    }
}

create_block_data_wrapper!(BlockData);

#[async_trait]
impl<C: Cache<BlockId, Data> + Send + Sync, B: OptimizedBlockStoreWriter + Send + Sync>
    OptimizedBlockStoreWriter for CachingBlockStore<C, B>
{
    type BlockData = BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        todo!()
        // let data = B::allocate(FORMAT_VERSION_HEADER.len() + C::CIPHERTEXT_OVERHEAD + size)
        //     .extract()
        //     .into_subregion((FORMAT_VERSION_HEADER.len() + C::CIPHERTEXT_OVERHEAD)..);
        // BlockData::new(data)
    }

    async fn try_create_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<bool> {
        todo!()
        // let ciphertext = self._encrypt(data.extract()).await?;
        // self.underlying_block_store
        //     .try_create_optimized(id, B::BlockData::new(ciphertext))
        //     .await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        todo!()
        // let ciphertext = self._encrypt(data.extract()).await?;
        // self.underlying_block_store
        //     .store_optimized(id, B::BlockData::new(ciphertext))
        //     .await
    }
}

impl<
        C: Cache<BlockId, Data> + Send + Sync,
        B: BlockStore + OptimizedBlockStoreWriter + Send + Sync,
    > BlockStore for CachingBlockStore<C, B>
{
}
