use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use std::fmt::{self, Debug};
use std::pin::Pin;

use crate::{
    high_level::LockingBlockStore,
    low_level::{BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter},
    tests::Fixture,
    utils::{RemoveResult, TryCreateResult},
    BlockId,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard, SyncDrop},
    data::Data,
};

/// Wrap a [LockingBlockStore] into a [BlockStore] so that we can run the regular block store tests on it.
pub struct BlockStoreAdapter<B: BlockStore + Send + Sync + Debug + 'static>(
    AsyncDropGuard<LockingBlockStore<B>>,
);

impl<B: BlockStore + Send + Sync + Debug + 'static> BlockStoreAdapter<B> {
    pub fn new(store: AsyncDropGuard<LockingBlockStore<B>>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(store))
    }

    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.0.clear_cache_slow().await
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync + Debug + 'static> BlockStoreReader for BlockStoreAdapter<B> {
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

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.0.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.0.block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        self.0.all_blocks().await
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync + Debug + 'static> BlockStoreDeleter for BlockStoreAdapter<B> {
    async fn remove(&self, id: &BlockId) -> Result<crate::utils::RemoveResult> {
        self.0.remove(id).await.map(|r| match r {
            RemoveResult::SuccessfullyRemoved => crate::utils::RemoveResult::SuccessfullyRemoved,
            RemoveResult::NotRemovedBecauseItDoesntExist => {
                crate::utils::RemoveResult::NotRemovedBecauseItDoesntExist
            }
        })
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync + Debug + 'static> BlockStoreWriter for BlockStoreAdapter<B> {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<crate::utils::TryCreateResult> {
        self.0
            .try_create(id, &data.to_vec().into())
            .await
            .map(|r| match r {
                TryCreateResult::SuccessfullyCreated => {
                    crate::utils::TryCreateResult::SuccessfullyCreated
                }
                TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => {
                    crate::utils::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists
                }
            })
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        self.0.overwrite(id, &data.to_vec().into()).await
    }
}

impl<B: BlockStore + Send + Sync + Debug + 'static> Debug for BlockStoreAdapter<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStoreAdapter")
    }
}

#[async_trait]
impl<B: BlockStore + Send + Sync + Debug + 'static> AsyncDrop for BlockStoreAdapter<B> {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.0.async_drop().await
    }
}

impl<B: BlockStore + Send + Sync + Debug + 'static> BlockStore for BlockStoreAdapter<B> {}

/// TestFixtureAdapter takes a [Fixture] for a [BlockStore] and makes it into
/// a [Fixture] that creates a [LockingBlockStore] based on that [BlockStore].
/// This allows using our block store test suite on [LockingBlockStore].
pub struct TestFixtureAdapter<F: Fixture + Sync, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}
#[async_trait]
impl<F: Fixture + Send + Sync, const FLUSH_CACHE_ON_YIELD: bool> crate::tests::Fixture
    for TestFixtureAdapter<F, FLUSH_CACHE_ON_YIELD>
{
    type ConcreteBlockStore = BlockStoreAdapter<F::ConcreteBlockStore>;
    fn new() -> Self {
        Self { f: F::new() }
    }
    async fn store(&mut self) -> SyncDrop<Self::ConcreteBlockStore> {
        let inner: AsyncDropGuard<F::ConcreteBlockStore> =
            self.f.store().await.into_inner_dont_drop();
        SyncDrop::new(BlockStoreAdapter::new(LockingBlockStore::new(inner)))
    }
    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore) {
        if FLUSH_CACHE_ON_YIELD {
            store.clear_cache_slow().await.unwrap();
        }
    }
}
