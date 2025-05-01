use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use std::{fmt::Debug, ops::Deref};

use super::interface::BlobStore;
use crate::BlobId;
use cryfs_blockstore::{BlockId, RemoveResult};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc};

#[async_trait]
impl<B: BlobStore + Send + Sync + Debug + AsyncDrop> BlobStore for AsyncDropArc<B> {
    type ConcreteBlob<'a>
        = B::ConcreteBlob<'a>
    where
        Self: 'a;

    async fn create(&self) -> Result<Self::ConcreteBlob<'_>> {
        self.deref().create().await
    }

    async fn try_create(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>> {
        self.deref().try_create(id).await
    }

    async fn load(&self, id: &BlobId) -> Result<Option<Self::ConcreteBlob<'_>>> {
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

    fn virtual_block_size_bytes(&self) -> Byte {
        self.deref().virtual_block_size_bytes()
    }

    async fn load_block_depth(&self, id: &BlockId) -> Result<Option<u8>> {
        self.deref().load_block_depth(id).await
    }

    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()> {
        self.deref().clear_cache_slow().await
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

    struct TestFixture<const BLOCK_SIZE_BYTES: u64>;
    #[async_trait]
    impl<const BLOCK_SIZE_BYTES: u64> Fixture for TestFixture<BLOCK_SIZE_BYTES> {
        type ConcreteBlobStore =
            AsyncDropArc<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlobStore> {
            AsyncDropArc::new(
                BlobStoreOnBlocks::new(
                    LockingBlockStore::new(InMemoryBlockStore::new()),
                    Byte::from_u64(BLOCK_SIZE_BYTES),
                )
                .await
                .unwrap(),
            )
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlobStore) {}
    }

    crate::instantiate_blobstore_tests!(TestFixture<1024>, (flavor = "multi_thread"));
}
