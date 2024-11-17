use anyhow::{anyhow, Result};
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::{BoxStream, StreamExt};
use std::collections::hash_map::HashMap;
use std::fmt::{self, Debug};
use std::sync::RwLock;
use sysinfo::System;

use crate::low_level::InvalidBlockSizeError;
use crate::{
    low_level::{
        interface::block_data::IBlockData, BlockStore, BlockStoreDeleter, BlockStoreReader,
        OptimizedBlockStoreWriter,
    },
    BlockId, RemoveResult, TryCreateResult,
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
};

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
            .map_err(|err| anyhow!("Failed to acquire lock: {}", err))?;
        Ok(blocks.contains_key(id))
    }

    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let blocks = self
            .blocks
            .read()
            .map_err(|err| anyhow!("Failed to acquire lock: {}", err))?;
        let load_result = blocks.get(id).cloned();
        Ok(load_result)
    }

    async fn num_blocks(&self) -> Result<u64> {
        let blocks = self
            .blocks
            .read()
            .map_err(|err| anyhow!("Failed to acquire lock: {}", err))?;
        Ok(blocks.len() as u64)
    }

    fn estimate_num_free_bytes(&self) -> Result<Byte> {
        let mut sys = System::new();
        sys.refresh_memory();
        Ok(Byte::from_u64(sys.available_memory()))
    }

    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        Ok(block_size)
    }

    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        let blocks = self
            .blocks
            .read()
            .map_err(|err| anyhow!("Failed to acquire lock: {}", err))?;
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
            .map_err(|err| anyhow!("Failed to acquire lock: {}", err))?;
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
            .map_err(|err| anyhow!("Failed to acquire lock: {}", err))?;
        if blocks.contains_key(id) {
            Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists)
        } else {
            let insert_result = blocks.insert(*id, data.extract());
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
            .map_err(|err| anyhow!("Failed to acquire lock: {}", err))?;
        blocks.insert(*id, data.extract());
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
    use crate::instantiate_blockstore_tests;
    use crate::tests::Fixture;

    struct TestFixture {}
    #[async_trait]
    impl Fixture for TestFixture {
        type ConcreteBlockStore = InMemoryBlockStore;
        fn new() -> Self {
            Self {}
        }
        async fn store(&mut self) -> AsyncDropGuard<Self::ConcreteBlockStore> {
            InMemoryBlockStore::new()
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
