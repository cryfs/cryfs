use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::stream::{Stream, StreamExt};
use std::collections::hash_map::HashMap;
use std::fmt::{self, Debug};
use std::pin::Pin;
use std::sync::RwLock;
use sysinfo::{System, SystemExt};

use super::block_data::IBlockData;
use super::{
    BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter,
    RemoveResult, TryCreateResult,
};
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct InMemoryBlockStore {
    blocks: RwLock<HashMap<BlockId, Data>>,
}

impl InMemoryBlockStore {
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            blocks: RwLock::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl BlockStoreReader for InMemoryBlockStore {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        let blocks = self
            .blocks
            .read()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        Ok(blocks.contains_key(id))
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let blocks = self
            .blocks
            .read()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        let load_result = blocks.get(id).cloned();
        Ok(load_result.map(|d| d.into()))
    }

    async fn num_blocks(&self) -> Result<u64> {
        let blocks = self
            .blocks
            .read()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        Ok(blocks.len() as u64)
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        let mut sys = System::new();
        sys.refresh_memory();
        Ok(sys.available_memory())
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        Ok(block_size)
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        let blocks = self
            .blocks
            .read()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        Ok(
            // TODO Do we still need to collect here after having switched to a stream?
            futures::stream::iter(
                blocks
                    .keys()
                    .cloned()
                    .map(Ok)
                    .collect::<Vec<Result<BlockId>>>(),
            )
            .boxed(),
        )
    }
}

#[async_trait]
impl BlockStoreDeleter for InMemoryBlockStore {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        let mut blocks = self
            .blocks
            .write()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        let remove_result = blocks.remove(id);
        match remove_result {
            Some(_) => Ok(RemoveResult::SuccessfullyRemoved),
            None => Ok(RemoveResult::NotRemovedBecauseItDoesntExist),
        }
    }
}

create_block_data_wrapper!(BlockData);

#[async_trait]
impl OptimizedBlockStoreWriter for InMemoryBlockStore {
    type BlockData = BlockData;

    fn allocate(size: usize) -> BlockData {
        BlockData::new(Data::from(vec![0; size]))
    }

    async fn try_create_optimized(&self, id: &BlockId, data: BlockData) -> Result<TryCreateResult> {
        let mut blocks = self
            .blocks
            .write()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        if blocks.contains_key(id) {
            Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists)
        } else {
            let insert_result = blocks.insert(id.clone(), data.extract());
            assert!(
                insert_result.is_none(),
                "We just checked above that this key doesn't exist, why does it exist now?"
            );
            Ok(TryCreateResult::SuccessfullyCreated)
        }
    }

    async fn store_optimized(&self, id: &BlockId, data: BlockData) -> Result<()> {
        let mut blocks = self
            .blocks
            .write()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        blocks.insert(id.clone(), data.extract());
        Ok(())
    }
}

impl Debug for InMemoryBlockStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InMemoryBlockStore")
    }
}

#[async_trait]
impl AsyncDrop for InMemoryBlockStore {
    type Error = anyhow::Error;
    async fn async_drop_impl(&mut self) -> Result<()> {
        Ok(())
    }
}

impl BlockStore for InMemoryBlockStore {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockstore::tests::Fixture;
    use crate::instantiate_blockstore_tests;
    use crate::utils::async_drop::SyncDrop;

    struct TestFixture {}
    #[async_trait]
    impl Fixture for TestFixture {
        type ConcreteBlockStore = InMemoryBlockStore;
        fn new() -> Self {
            Self {}
        }
        fn store(&mut self) -> SyncDrop<Self::ConcreteBlockStore> {
            SyncDrop::new(InMemoryBlockStore::new())
        }
        async fn yield_fixture(&self, _store: &Self::ConcreteBlockStore) {}
    }

    instantiate_blockstore_tests!(TestFixture, (flavor = "multi_thread"));

    #[test]
    fn test_block_size_from_physical_block_size() {
        let mut fixture = TestFixture::new();
        let store = fixture.store();
        let expected_overhead: u64 = 0u64;

        assert_eq!(
            0u64,
            store
                .block_size_from_physical_block_size(expected_overhead)
                .unwrap()
        );
        assert_eq!(
            20u64,
            store
                .block_size_from_physical_block_size(expected_overhead + 20u64)
                .unwrap()
        );
    }
}
