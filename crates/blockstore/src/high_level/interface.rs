use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use futures::stream::BoxStream;

use crate::{BlockId, InvalidBlockSizeError, RemoveResult, TryCreateResult};
use cryfs_utils::data::Data;

// TODO Now that we have the interface, go through downstream code and see where we can replace direct use of LockingBlockStore with the BlockStore trait.

#[async_trait]
pub trait Block {
    fn block_id(&self) -> &BlockId;
    fn data(&self) -> &Data;
    fn data_mut(&mut self) -> &mut Data;
    async fn resize(&mut self, new_size: usize);
}

#[async_trait]
pub trait BlockStore {
    type Block: Block;

    async fn load(&self, block_id: BlockId) -> Result<Option<Self::Block>>;
    async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult>;
    async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()>;
    async fn remove_by_id(&self, block_id: &BlockId) -> Result<RemoveResult>;
    async fn remove(&self, block: Self::Block) -> Result<()>;

    // Note: for any blocks that are created or removed while the returned stream is running,
    // we don't give any guarantees for whether they're counted or not.
    async fn num_blocks(&self) -> Result<u64>;
    fn estimate_num_free_bytes(&self) -> Result<Byte>;
    fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError>;

    // Note: for any blocks that are created or removed while the returned stream is running,
    // we don't give any guarantees for whether they'll be part of the stream or not.
    async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>>;
    async fn create(&self, data: &Data) -> Result<BlockId>;
    async fn flush_block(&self, block: &mut Self::Block) -> Result<()>;

    /// clear_cache_slow is only used in test cases. Without test cases calling it, they would only
    /// ever test cached blocks and never have to store/reload them to the base store.
    /// This is implemented in a very slow way and shouldn't be used in non-test code.
    #[cfg(any(test, feature = "testutils"))]
    async fn clear_cache_slow(&self) -> Result<()>;
    #[cfg(any(test, feature = "testutils"))]
    async fn clear_unloaded_blocks_from_cache(&self) -> Result<()>;
}
