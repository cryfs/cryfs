use anyhow::{Context, Result};
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::{self, Debug};

use crate::{
    low_level::{
        interface::block_data::IBlockData, BlockStore, BlockStoreDeleter, BlockStoreReader,
        BlockStoreWriter, InvalidBlockSizeError, OptimizedBlockStoreWriter,
    },
    utils::{RemoveResult, TryCreateResult},
    BlockId,
};

use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

/// This block store compresses blocks before storing them. The implementation isn't really
/// optimized or meant for production code, it's just being used in test cases currently.
pub struct CompressingBlockStore<B: Send + Debug + AsyncDrop<Error = anyhow::Error>> {
    underlying_block_store: AsyncDropGuard<B>,
}

impl<B: Send + Sync + Debug + AsyncDrop<Error = anyhow::Error>> CompressingBlockStore<B> {
    pub fn new(underlying_block_store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            underlying_block_store,
        })
    }
}

impl<B: Send + Debug + AsyncDrop<Error = anyhow::Error>> Debug for CompressingBlockStore<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CompressingBlockStore")
    }
}

#[async_trait]
impl<B: BlockStoreReader + Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>> BlockStoreReader
    for CompressingBlockStore<B>
{
    async fn exists(&self, block_id: &BlockId) -> Result<bool> {
        self.underlying_block_store.exists(block_id).await
    }

    async fn load(&self, block_id: &BlockId) -> Result<Option<Data>> {
        let loaded = self.underlying_block_store.load(block_id).await.context(
            "CompressingBlockStore failed to load the block from the underlying block store",
        )?;
        if let Some(loaded) = loaded {
            Ok(Some(_decompress(loaded).await?))
        } else {
            Ok(None)
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        self.underlying_block_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        //We probably have more since we're compressing, but we don't know exactly how much.
        //The best we can do is ignore the compression step here.
        self.underlying_block_store
            .block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.underlying_block_store.all_blocks().await
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>>
    BlockStoreDeleter for CompressingBlockStore<B>
{
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.underlying_block_store.remove(id).await
    }
}

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>>
    OptimizedBlockStoreWriter for CompressingBlockStore<B>
{
    type BlockData = B::BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        B::allocate(size)
    }

    async fn try_create_optimized(
        &self,
        id: &BlockId,
        data: B::BlockData,
    ) -> Result<TryCreateResult> {
        let compressed = _compress(data.extract()).await?;
        self.underlying_block_store
            // We cannot use try_create_optimized because we may not have enough prefix bytes available
            .try_create(id, &compressed)
            .await
    }

    async fn store_optimized(&self, id: &BlockId, data: B::BlockData) -> Result<()> {
        let compressed = _compress(data.extract()).await?;
        self.underlying_block_store
            // We cannot use store_optimized because we may not have enough prefix bytes available
            .store(id, &compressed)
            .await
    }
}

#[async_trait]
impl<B: Sync + Send + Debug + AsyncDrop<Error = anyhow::Error>> AsyncDrop
    for CompressingBlockStore<B>
{
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.underlying_block_store.async_drop().await?;
        Ok(())
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter + Sync + Send + Debug> BlockStore
    for CompressingBlockStore<B>
{
}

async fn _decompress(data: Data) -> Result<Data> {
    // TODO Is a dedicated thread pool better than spawn_blocking? Similar for EncryptedBlockStore.
    let decompressed = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
        let mut decompressed = Vec::new();
        lzzzz::lz4f::decompress_to_vec(&data, &mut decompressed)?;
        Ok(decompressed)
    })
    .await??;
    Ok(decompressed.into())
}

async fn _compress(data: Data) -> Result<Data> {
    // TODO Is a dedicated thread pool better than spawn_blocking? Similar for EncryptedBlockStore.
    let compressed = tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
        let prefs = lzzzz::lz4f::Preferences::default();
        let mut compressed = Vec::new();
        lzzzz::lz4f::compress_to_vec(&data, &mut compressed, &prefs)?;
        Ok(compressed)
    })
    .await??;
    Ok(compressed.into())
}

#[cfg(test)]
mod generic_tests {
    use super::*;
    use crate::low_level::InMemoryBlockStore;
    use crate::tests::Fixture;

    use crate::instantiate_blockstore_tests;

    struct TestFixture {}
    #[async_trait]
    impl Fixture for TestFixture {
        type ConcreteBlockStore = CompressingBlockStore<InMemoryBlockStore>;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            CompressingBlockStore::new(InMemoryBlockStore::new())
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests!(TestFixture, (flavor = "multi_thread"));

    #[tokio::test]
    async fn test_block_size_from_physical_block_size() {
        let mut fixture = TestFixture::new();
        let mut store = fixture.store().await;
        let expected_overhead = Byte::from_u64(0);

        assert_eq!(
            0u64,
            store
                .block_size_from_physical_block_size(expected_overhead)
                .unwrap()
        );
        assert_eq!(
            20u64,
            store
                .block_size_from_physical_block_size(
                    expected_overhead.add(Byte::from_u64(20)).unwrap()
                )
                .unwrap()
        );

        store.async_drop().await.unwrap();
    }
}
