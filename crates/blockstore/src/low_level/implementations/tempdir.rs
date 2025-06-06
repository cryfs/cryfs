use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::Debug;
use tempdir::TempDir;

use crate::{
    BlockId, Overhead, RemoveResult, TryCreateResult,
    low_level::{BlockStoreDeleter, BlockStoreReader, LLBlockStore, OptimizedBlockStoreWriter},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

use super::OnDiskBlockStore;

/// Runs a [OnDiskBlockStore] in a temporary directory. Mostly useful because it binds the
/// lifetime of the temporary directory to the lifetime of the block store and automatically
/// deletes the directory when the block store is dropped.
#[derive(Debug)]
pub struct TempDirBlockStore {
    // Order is important, we want to drop the underlying store before the tempdir
    underlying_store: AsyncDropGuard<OnDiskBlockStore>,
    _tempdir: TempDir,
}

impl TempDirBlockStore {
    pub fn new() -> AsyncDropGuard<Self> {
        let tempdir = TempDir::new("cryfs-tempdir-blockstore").expect("Failed to create tempdir");
        let path = tempdir.path().to_owned();
        AsyncDropGuard::new(Self {
            _tempdir: tempdir,
            underlying_store: OnDiskBlockStore::new(path),
        })
    }
}

#[async_trait]
impl BlockStoreReader for TempDirBlockStore {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.underlying_store.exists(id).await
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.underlying_store.load(id).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.underlying_store.estimate_num_free_bytes()
    }

    fn overhead(&self) -> Overhead {
        self.underlying_store.overhead()
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.underlying_store.all_blocks().await
    }
}

#[async_trait]
impl BlockStoreDeleter for TempDirBlockStore {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.underlying_store.remove(id).await
    }
}

#[async_trait]
impl OptimizedBlockStoreWriter for TempDirBlockStore {
    type BlockData = <OnDiskBlockStore as OptimizedBlockStoreWriter>::BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        OnDiskBlockStore::allocate(size)
    }

    async fn try_create_optimized(
        &self,
        id: &BlockId,
        data: Self::BlockData,
    ) -> Result<TryCreateResult> {
        self.underlying_store.try_create_optimized(id, data).await
    }

    async fn store_optimized(&self, id: &BlockId, data: Self::BlockData) -> Result<()> {
        self.underlying_store.store_optimized(id, data).await
    }
}

#[async_trait]
impl AsyncDrop for TempDirBlockStore {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_store.async_drop().await?;
        Ok(())
    }
}

impl LLBlockStore for TempDirBlockStore {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instantiate_blockstore_tests_for_lowlevel_blockstore;
    use crate::tests::low_level::LLFixture;

    struct TestFixture {}
    #[async_trait]
    impl LLFixture for TestFixture {
        type ConcreteBlockStore = TempDirBlockStore;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            TempDirBlockStore::new()
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests_for_lowlevel_blockstore!(TestFixture, (flavor = "multi_thread"));
}
