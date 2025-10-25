use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use std::{fmt::Debug, ops::Deref};

use crate::{BlobId, interface::BlobStore};
use cryfs_blockstore::RemoveResult;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

#[async_trait]
impl<B: BlobStore + Send + Sync + Debug + AsyncDrop> BlobStore for AsyncDropArc<B> {
    type ConcreteBlob = B::ConcreteBlob;

    async fn create(&self) -> Result<AsyncDropGuard<Self::ConcreteBlob>> {
        self.deref().create().await
    }

    async fn try_create(&self, id: &BlobId) -> Result<Option<AsyncDropGuard<Self::ConcreteBlob>>> {
        self.deref().try_create(id).await
    }

    async fn load(&self, id: &BlobId) -> Result<Option<AsyncDropGuard<Self::ConcreteBlob>>> {
        self.deref().load(id).await
    }

    async fn remove_by_id(&self, id: &BlobId) -> Result<RemoveResult> {
        self.deref().remove_by_id(id).await
    }

    async fn num_nodes(&self) -> Result<u64> {
        self.deref().num_nodes().await
    }

    fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        self.deref().estimate_space_for_num_blocks_left()
    }

    fn logical_block_size_bytes(&self) -> Byte {
        self.deref().logical_block_size_bytes()
    }

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()> {
        self.deref().clear_cache_slow().await
    }
    #[cfg(any(test, feature = "testutils"))]
    async fn clear_unloaded_blocks_from_cache(&self) -> Result<()> {
        self.deref().clear_unloaded_blocks_from_cache().await
    }

    #[cfg(test)]
    async fn all_blobs(&self) -> Result<Vec<BlobId>> {
        self.deref().all_blobs().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BlobStoreOnBlocks, tests::fixture::Fixture};
    use async_trait::async_trait;
    use byte_unit::Byte;
    use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
    use cryfs_utils::async_drop::AsyncDropGuard;

    struct TestFixture;
    #[async_trait]
    impl Fixture for TestFixture {
        type ConcreteBlobStore =
            AsyncDropArc<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlobStore> {
            AsyncDropArc::new(
                BlobStoreOnBlocks::new(
                    LockingBlockStore::new(InMemoryBlockStore::new()),
                    Byte::from_u64(1024),
                )
                .await
                .unwrap(),
            )
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlobStore) {}
    }

    crate::instantiate_tests_for_blobstore!(TestFixture, (flavor = "multi_thread"));
}
