use anyhow::{anyhow, Result};
use std::collections::hash_map::HashMap;
use std::sync::RwLock;
use sysinfo::{System, SystemExt};

use super::{BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter};

use super::block_data::IBlockData;
use crate::data::Data;

pub struct InMemoryBlockStore {
    blocks: RwLock<HashMap<BlockId, Data>>,
}

impl InMemoryBlockStore {
    pub fn new() -> Self {
        Self {
            blocks: RwLock::new(HashMap::new()),
        }
    }
}

impl BlockStoreReader for InMemoryBlockStore {
    fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        let blocks = self
            .blocks
            .read()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        let load_result = blocks.get(id).cloned();
        Ok(load_result.map(|d| d.into()))
    }

    fn num_blocks(&self) -> Result<u64> {
        let blocks = self
            .blocks
            .read()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        Ok(blocks.len() as u64)
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        let mut sys = System::new();
        sys.refresh_memory();
        Ok(sys.get_available_memory())
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        Ok(block_size)
    }

    fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>> {
        let blocks = self
            .blocks
            .read()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        Ok(Box::new(
            blocks.keys().cloned().collect::<Vec<BlockId>>().into_iter(),
        ))
    }
}

impl BlockStoreDeleter for InMemoryBlockStore {
    fn remove(&self, id: &BlockId) -> Result<bool> {
        let mut blocks = self
            .blocks
            .write()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        let remove_result = blocks.remove(id);
        Ok(remove_result.is_some())
    }
}

create_block_data_wrapper!(BlockData);

impl OptimizedBlockStoreWriter for InMemoryBlockStore {
    type BlockData = BlockData;

    fn allocate(size: usize) -> BlockData {
        BlockData::new(Data::from(vec![0; size]))
    }

    fn try_create_optimized(&self, id: &BlockId, data: BlockData) -> Result<bool> {
        let mut blocks = self
            .blocks
            .write()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        if blocks.contains_key(id) {
            Ok(false)
        } else {
            let insert_result = blocks.insert(id.clone(), data.extract());
            assert!(
                insert_result.is_none(),
                "We just checked above that this key doesn't exist, why does it exist now?"
            );
            Ok(true)
        }
    }

    fn store_optimized(&self, id: &BlockId, data: BlockData) -> Result<()> {
        let mut blocks = self
            .blocks
            .write()
            .map_err(|_| anyhow!("Failed to acquire lock"))?;
        blocks.insert(id.clone(), data.extract());
        Ok(())
    }
}

impl BlockStore for InMemoryBlockStore {}
