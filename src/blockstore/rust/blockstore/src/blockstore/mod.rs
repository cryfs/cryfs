use anyhow::Result;

pub use cppbridge::{BlockId, BLOCKID_LEN};

pub trait BlockStoreReader {
    fn load(&self, id: &BlockId) -> Result<Option<Vec<u8>>>;
    fn num_blocks(&self) -> Result<u64>;
    fn estimate_num_free_bytes(&self) -> Result<u64>;
    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64>;

    fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>>;
}

pub trait BlockStoreWriter {
    fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool>;
    fn remove(&self, id: &BlockId) -> Result<bool>;
    fn store(&self, id: &BlockId, data: &[u8]) -> Result<()>;
}

pub trait BlockStore: BlockStoreReader + BlockStoreWriter {}

mod cppbridge;
mod encrypted;
mod inmemory;
mod integrity;
mod ondisk;
