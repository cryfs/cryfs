use anyhow::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use std::fmt::Debug;
use std::pin::Pin;

use crate::{
    low_level::{BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter},
    BlockId, RemoveResult, TryCreateResult,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

/// Wraps a BlockStore into a block store that only lets through read calls and panics on each write call.
/// TODO Can we solve this with the traits instead by having read-only code like the stats tool only rely on the BlockStoreReader trait?
#[derive(Debug)]
pub struct ReadOnlyBlockStore<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> {
    underlying_store: AsyncDropGuard<B>,
}

impl<B: Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> ReadOnlyBlockStore<B> {
    pub fn new(underlying: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_store: underlying,
        })
    }
}

#[async_trait]
impl<B: BlockStoreReader + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>> BlockStoreReader
    for ReadOnlyBlockStore<B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.underlying_store.exists(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.underlying_store.load(id).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.underlying_store
            .block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        self.underlying_store.all_blocks().await
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    BlockStoreDeleter for ReadOnlyBlockStore<B>
{
    async fn remove(&self, _id: &BlockId) -> Result<RemoveResult> {
        panic!("ReadOnlyBlockStore::remove blocked");
    }
}

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Debug + Sync + Send + AsyncDrop<Error = anyhow::Error>>
    OptimizedBlockStoreWriter for ReadOnlyBlockStore<B>
{
    type BlockData = B::BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        B::allocate(size)
    }

    async fn try_create_optimized(
        &self,
        _id: &BlockId,
        _data: Self::BlockData,
    ) -> Result<TryCreateResult> {
        panic!("ReadOnlyBlockStore::try_create_optimized blocked");
    }

    async fn store_optimized(&self, _id: &BlockId, _data: Self::BlockData) -> Result<()> {
        panic!("ReadOnlyBlockStore::store_optimized blocked");
    }
}

#[async_trait]
impl<B: Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>> AsyncDrop
    for ReadOnlyBlockStore<B>
{
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_store.async_drop().await?;
        Ok(())
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter + Sync + Send + Debug> BlockStore
    for ReadOnlyBlockStore<B>
{
}
