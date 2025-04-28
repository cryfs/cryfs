use anyhow::Result;
use byte_unit::Byte;
use futures::stream::BoxStream;

use crate::{BlockId, InvalidBlockSizeError, RemoveResult, TryCreateResult};
use cryfs_utils::{async_drop::AsyncDropGuard, data::Data};

pub trait Block {
    type BlockStore: BlockStore<Block = Self>;

    fn block_id(&self) -> &BlockId;
    fn data(&self) -> &Data;
    fn data_mut(&mut self) -> &mut Data;
    async fn resize(&mut self, new_size: usize);
    // TODO This is a weird API. We should probably change this so that code calls block_store.remove(block) instead. Then we can remove the `type BlockStore` above.
    async fn remove(self, block_store: &Self::BlockStore) -> Result<()>;
}

pub trait BlockStore {
    type Block: Block<BlockStore = Self>;

    async fn load(&self, block_id: BlockId) -> Result<Option<Self::Block>>;
    async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult>;
    async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()>;
    async fn remove_by_id(&self, block_id: &BlockId) -> Result<RemoveResult>;

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
