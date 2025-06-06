use anyhow::Result;
use async_trait::async_trait;
use binary_layout::Field;
use byte_unit::Byte;
use futures::stream::BoxStream;
use std::fmt::{self, Debug};

use super::super::{DataLeafNode, DataNode, DataNodeStore, layout::node};
use cryfs_blockstore::{
    BlockId, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter, InMemoryBlockStore,
    LLBlockStore, LockingBlockStore, Overhead, RemoveResult, TryCreateResult,
    tests::low_level::LLFixture,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

const MAX_BLOCK_SIZE: Byte = Byte::from_u64(1024 * 1024);

/// Wrap a [DataNodeStore] into a [BlockStore] so that we can run the regular block store tests on it.
/// Each block is stored as a DataLeafNode with the block data.
pub struct BlockStoreAdapter(AsyncDropGuard<DataNodeStore<LockingBlockStore<InMemoryBlockStore>>>);

impl BlockStoreAdapter {
    pub async fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self(
            DataNodeStore::new(
                LockingBlockStore::new(InMemoryBlockStore::new()),
                MAX_BLOCK_SIZE,
            )
            .await
            .unwrap(),
        ))
    }

    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.0.clear_cache_slow().await
    }

    async fn load_leaf(
        &self,
        id: BlockId,
    ) -> Result<Option<DataLeafNode<LockingBlockStore<InMemoryBlockStore>>>> {
        match self.0.load(id).await? {
            Some(DataNode::Leaf(leaf)) => Ok(Some(leaf)),
            Some(DataNode::Inner(_)) => panic!("This node store should only have leaf nodes"),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl BlockStoreReader for BlockStoreAdapter {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        Ok(self.load_leaf(*id).await?.is_some())
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        Ok(self
            .load_leaf(*id)
            .await?
            .map(|leaf| leaf.data().to_vec().into()))
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.0.num_nodes().await
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        Ok(Byte::from_u64(
            self.0.estimate_space_for_num_blocks_left()?
                * self.0.layout().max_bytes_per_leaf() as u64,
        ))
    }

    fn overhead(&self) -> Overhead {
        Overhead::new(Byte::from_u64(node::data::OFFSET as u64))
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        self.0.all_nodes().await
    }
}

#[async_trait]
impl BlockStoreDeleter for BlockStoreAdapter {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        if let Some(leaf) = self.load_leaf(*id).await? {
            self.0.remove(leaf.upcast()).await?;
            Ok(RemoveResult::SuccessfullyRemoved)
        } else {
            Ok(RemoveResult::NotRemovedBecauseItDoesntExist)
        }
    }
}

#[async_trait]
impl BlockStoreWriter for BlockStoreAdapter {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<TryCreateResult> {
        if self.exists(id).await? {
            Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists)
        } else {
            self.store(id, data).await?;
            Ok(TryCreateResult::SuccessfullyCreated)
        }
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        self.0.overwrite_with_leaf_node(id, data).await
    }
}

impl Debug for BlockStoreAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockStoreAdapter")
    }
}

#[async_trait]
impl AsyncDrop for BlockStoreAdapter {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        self.0.async_drop().await
    }
}

impl LLBlockStore for BlockStoreAdapter {}

/// TestFixtureAdapter takes a [Fixture] for a [BlockStore] and makes it into
/// a [Fixture] that creates a [DataNodeStore] based on that [BlockStore].
/// This allows using our block store test suite on [DataNodeStore].
pub struct TestFixtureAdapter<const FLUSH_CACHE_ON_YIELD: bool> {}
#[async_trait]
impl<const FLUSH_CACHE_ON_YIELD: bool> LLFixture for TestFixtureAdapter<FLUSH_CACHE_ON_YIELD> {
    type ConcreteBlockStore = BlockStoreAdapter;
    fn new() -> Self {
        Self {}
    }
    async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
        BlockStoreAdapter::new().await
    }
    async fn yield_fixture(&self, store: &Self::ConcreteBlockStore) {
        if FLUSH_CACHE_ON_YIELD {
            store.clear_cache_slow().await.unwrap();
        }
    }
}
