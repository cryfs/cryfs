use anyhow::Result;

pub use cppbridge::{BLOCKID_LEN, BlockId};

pub trait BlockStore2 {
  fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool>;
  fn remove(&self, id: &BlockId) -> Result<bool>;
  fn load(&self, id: &BlockId) -> Result<Option<Vec<u8>>>;
  fn store(&self, id: &BlockId, data: &[u8]) -> Result<()>;
  fn num_blocks(&self) -> Result<u64>;
  fn estimate_num_free_bytes(&self) -> Result<u64>;
  fn block_size_from_physical_block_size(&self, block_size: u64) -> u64;

  fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>>;
}

mod cppbridge;
mod encrypted;
mod inmemory;
mod ondisk;