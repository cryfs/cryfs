use async_trait::async_trait;
use cryfs_utils::async_drop::AsyncDropGuard;
use std::fmt::Debug;

use super::LLFixture;
use crate::{BlockStore as _, LLBlockStore, LockingBlockStore, tests::high_level::HLFixture};

/// [FixtureAdapterForHLTests] takes a [LLFixture] for a [LLBlockStore] and makes it into
/// a [HLFixture] that creates a [LockingBlockStore] based on that [LLBlockStore].
/// This allows using our high level block store test suite on a low level [LLBlockStore].
pub struct FixtureAdapterForHLTests<F: LLFixture, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}

#[async_trait]
impl<F, const FLUSH_CACHE_ON_YIELD: bool> HLFixture
    for FixtureAdapterForHLTests<F, FLUSH_CACHE_ON_YIELD>
where
    F: LLFixture + Send + Sync,
    F::ConcreteBlockStore: LLBlockStore + Send + Sync + Debug + 'static,
{
    type ConcreteBlockStore = LockingBlockStore<F::ConcreteBlockStore>;

    fn new() -> Self {
        Self { f: F::new() }
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
        let inner = self.f.store().await;
        LockingBlockStore::new(inner)
    }
    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore) {
        if FLUSH_CACHE_ON_YIELD {
            store.clear_cache_slow().await.unwrap();
        }
        self.f.yield_fixture(store.inner_block_store()).await;
    }
}
