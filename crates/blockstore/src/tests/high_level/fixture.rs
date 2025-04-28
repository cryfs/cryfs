use async_trait::async_trait;
use std::fmt::Debug;

use crate::{BlockStore as _, LLBlockStore, LockingBlockStore, tests::low_level::LLFixture};
use cryfs_utils::async_drop::AsyncDropGuard;

/// Based on a [crate::tests::Fixture], we define a [LockingBlockStoreFixture]
/// that uses the underlying fixture and wraps its blockstore into a [LockingBlockStore]
/// to run LockingBlockStore tests on it.
#[async_trait]
pub trait LockingBlockStoreFixture {
    type UnderlyingBlockStore: LLBlockStore + Send + Sync + Debug + 'static;

    fn new() -> Self;
    async fn store(&mut self) -> AsyncDropGuard<LockingBlockStore<Self::UnderlyingBlockStore>>;
    async fn yield_fixture(&self, store: &LockingBlockStore<Self::UnderlyingBlockStore>);
}

pub struct LockingBlockStoreFixtureImpl<F: LLFixture, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}

#[async_trait]
impl<F, const FLUSH_CACHE_ON_YIELD: bool> LockingBlockStoreFixture
    for LockingBlockStoreFixtureImpl<F, FLUSH_CACHE_ON_YIELD>
where
    F: LLFixture + Send + Sync,
    F::ConcreteBlockStore: Send + Sync + Debug + 'static,
{
    type UnderlyingBlockStore = F::ConcreteBlockStore;
    fn new() -> Self {
        Self { f: F::new() }
    }
    async fn store(&mut self) -> AsyncDropGuard<LockingBlockStore<Self::UnderlyingBlockStore>> {
        let inner = self.f.store().await;
        LockingBlockStore::new(inner)
    }
    async fn yield_fixture(&self, store: &LockingBlockStore<Self::UnderlyingBlockStore>) {
        if FLUSH_CACHE_ON_YIELD {
            store.clear_cache_slow().await.unwrap();
        }
    }
}
