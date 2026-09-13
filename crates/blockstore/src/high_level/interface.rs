use anyhow::Result;
use byte_unit::Byte;
use futures::stream::BoxStream;

use crate::{BlockId, Overhead, RemoveResult, TryCreateResult};
use cryfs_utils::data::Data;

// TODO Now that we have the interface, go through downstream code and see where we can replace direct use of LockingBlockStore with the BlockStore trait.

pub trait Block {
    fn block_id(&self) -> &BlockId;
    fn data(&self) -> &Data;
    fn data_mut(&mut self) -> &mut Data;
    fn resize(&mut self, new_size: usize) -> impl Future<Output = ()> + Send;
}

pub trait BlockStore {
    type Block: Block;

    fn load(&self, block_id: BlockId) -> impl Future<Output = Result<Option<Self::Block>>> + Send;
    fn try_create(
        &self,
        block_id: &BlockId,
        data: &Data,
    ) -> impl Future<Output = Result<TryCreateResult>> + Send;
    fn overwrite(&self, block_id: &BlockId, data: &Data)
    -> impl Future<Output = Result<()>> + Send;
    fn remove_by_id(&self, block_id: &BlockId)
    -> impl Future<Output = Result<RemoveResult>> + Send;
    fn remove(&self, block: Self::Block) -> impl Future<Output = Result<()>> + Send;

    // Note: for any blocks that are created or removed while the returned stream is running,
    // we don't give any guarantees for whether they're counted or not.
    fn num_blocks(&self) -> impl Future<Output = Result<u64>> + Send;
    fn estimate_num_free_bytes(&self) -> Result<Byte>;
    fn overhead(&self) -> Overhead;

    // Note: for any blocks that are created or removed while the returned stream is running,
    // we don't give any guarantees for whether they'll be part of the stream or not.
    fn all_blocks(
        &self,
    ) -> impl Future<Output = Result<BoxStream<'static, Result<BlockId>>>> + Send;
    fn create(&self, data: &Data) -> impl Future<Output = Result<BlockId>> + Send;
    fn flush_block(&self, block: &mut Self::Block) -> impl Future<Output = Result<()>> + Send;

    /// clear_cache_slow is only used in test cases. Without test cases calling it, they would only
    /// ever test cached blocks and never have to store/reload them to the base store.
    /// This is implemented in a very slow way and shouldn't be used in non-test code.
    #[cfg(any(test, feature = "testutils"))]
    fn clear_cache_slow(&self) -> impl Future<Output = Result<()>> + Send;
    #[cfg(any(test, feature = "testutils"))]
    fn clear_unloaded_blocks_from_cache(&self) -> impl Future<Output = Result<()>> + Send;
}
