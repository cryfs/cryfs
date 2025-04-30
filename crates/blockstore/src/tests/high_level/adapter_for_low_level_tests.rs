use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::{self, Debug};

use crate::{
    BlockId,
    high_level::{Block as _, BlockStore},
    low_level::{
        BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, InvalidBlockSizeError, LLBlockStore,
    },
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

use super::HLFixture;

/// Wrap a [BlockStore] into a [LLBlockStore] so that we can run the low level block store tests on it.
pub struct BlockStoreToLLBlockStoreAdapter<
    B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static,
>(AsyncDropGuard<B>);

impl<B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static>
    BlockStoreToLLBlockStoreAdapter<B>
{
    pub fn new(store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(store))
    }

    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.0.clear_cache_slow().await
    }
}

#[async_trait]
impl<B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static>
    BlockStoreReader for BlockStoreToLLBlockStoreAdapter<B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        Ok(self.0.load(*id).await?.is_some())
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        if let Some(block) = self.0.load(*id).await? {
            Ok(Some(block.data().clone()))
        } else {
            Ok(None)
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.0.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.0.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        self.0.block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.0.all_blocks().await
    }
}

#[async_trait]
impl<B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static>
    BlockStoreDeleter for BlockStoreToLLBlockStoreAdapter<B>
{
    async fn remove(&self, id: &BlockId) -> Result<crate::utils::RemoveResult> {
        self.0.remove_by_id(id).await
    }
}

#[async_trait]
impl<B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static>
    BlockStoreWriter for BlockStoreToLLBlockStoreAdapter<B>
{
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<crate::utils::TryCreateResult> {
        self.0.try_create(id, &data.to_vec().into()).await
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        self.0.overwrite(id, &data.to_vec().into()).await
    }
}

impl<B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static> Debug
    for BlockStoreToLLBlockStoreAdapter<B>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStoreToLLBlockStoreAdapter")
    }
}

#[async_trait]
impl<B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static> AsyncDrop
    for BlockStoreToLLBlockStoreAdapter<B>
{
    type Error = <B as AsyncDrop>::Error;
    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        self.0.async_drop().await
    }
}

impl<B: BlockStore + AsyncDrop<Error = anyhow::Error> + Send + Sync + Debug + 'static> LLBlockStore
    for BlockStoreToLLBlockStoreAdapter<B>
{
}

/// [FixtureAdapterForLLTests] takes a [HLFixture] for a [BlockStore] and makes it into
/// a [LLFixture] that creates a [BlockStoreToLLBlockStoreAdapter] implementing [LLBlockStore] for that [BlockStore].
/// This allows using our low level block store test suite on a high level [BlockStore].
pub struct FixtureAdapterForLLTests<F: HLFixture + Sync, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}
#[async_trait]
impl<
    F: HLFixture<ConcreteBlockStore: AsyncDrop<Error = anyhow::Error>> + Send + Sync,
    const FLUSH_CACHE_ON_YIELD: bool,
> crate::tests::low_level::LLFixture for FixtureAdapterForLLTests<F, FLUSH_CACHE_ON_YIELD>
{
    type ConcreteBlockStore = BlockStoreToLLBlockStoreAdapter<F::ConcreteBlockStore>;
    fn new() -> Self {
        Self { f: F::new() }
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
        let inner: AsyncDropGuard<F::ConcreteBlockStore> = self.f.store().await;
        BlockStoreToLLBlockStoreAdapter::new(inner)
    }
    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore) {
        if FLUSH_CACHE_ON_YIELD {
            store.clear_cache_slow().await.unwrap();
        }
        self.f.yield_fixture(&store.0).await;
    }
}
