use async_trait::async_trait;
use cryfs_blockstore::tests::low_level::LLFixture;

use super::block_store_adapter::BlockStoreAdapter;
use crate::tests::fixture::Fixture;
use cryfs_utils::async_drop::AsyncDropGuard;

/// TestFixtureAdapter takes a [Fixture] for a [crate::BlobStore] and makes it into
/// a [Fixture] that creates a [cryfs_blockstore::BlockStore] by storing each block as a blob.
/// This allows using our block store test suite on a [crate::BlobStore].
pub struct TestFixtureAdapter<F: Fixture + Send + Sync, const FLUSH_CACHE_ON_YIELD: bool> {
    f: F,
}

#[async_trait]
impl<F: Fixture + Send + Sync, const FLUSH_CACHE_ON_YIELD: bool> LLFixture
    for TestFixtureAdapter<F, FLUSH_CACHE_ON_YIELD>
{
    type ConcreteBlockStore = BlockStoreAdapter<F::ConcreteBlobStore>;
    fn new() -> Self {
        Self { f: Fixture::new() }
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
        BlockStoreAdapter::new(self.f.store().await).await
    }

    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore) {
        if FLUSH_CACHE_ON_YIELD {
            store.clear_cache_slow().await.unwrap();
        }
        self.f.yield_fixture(store.inner()).await;
    }
}
