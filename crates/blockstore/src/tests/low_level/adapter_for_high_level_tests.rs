use async_trait::async_trait;
use cryfs_utils::async_drop::AsyncDropGuard;
use std::fmt::Debug;

use super::LLFixture;
use crate::{BlockStore as _, LockingBlockStore, tests::high_level::HLFixture};

/// Take a [LLFixture] meant to set up a [LLBlockStore] and create a [HLFixture] that sets up a [BlockStore] by wrapping the [LLBlockStore] into a [LockingBlockStore].
/// This way, we can run the high level block store tests on it.
pub struct LockingBlockStoreFixture<F: LLFixture, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}

#[async_trait]
impl<F, const FLUSH_CACHE_ON_YIELD: bool> HLFixture
    for LockingBlockStoreFixture<F, FLUSH_CACHE_ON_YIELD>
where
    F: LLFixture + Send + Sync,
    F::ConcreteBlockStore: Send + Sync + Debug + 'static,
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
