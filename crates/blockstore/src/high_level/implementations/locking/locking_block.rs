use anyhow::{Result, bail};
use std::fmt::{self, Debug};

use super::{LockingBlockStore, cache::BlockCacheEntryGuard};
use crate::{BlockId, LLBlockStore, RemoveResult, high_level::Block};
use cryfs_utils::data::Data;

pub struct LockingBlock<B: LLBlockStore + Send + Sync + Debug + 'static> {
    pub(super) cache_entry: BlockCacheEntryGuard<B>,
}

impl<B: crate::low_level::LLBlockStore + Send + Sync + Debug> Block for LockingBlock<B> {
    type BlockStore = LockingBlockStore<B>;

    #[inline]
    fn block_id(&self) -> &BlockId {
        self.cache_entry.key()
    }

    #[inline]
    fn data(&self) -> &Data {
        self.cache_entry
            .value()
            .expect("An existing block cannot have a None cache entry")
            .data()
    }

    #[inline]
    fn data_mut(&mut self) -> &mut Data {
        self.cache_entry
            .value_mut()
            .expect("An existing block cannot have a None cache entry")
            .data_mut()
    }

    async fn resize(&mut self, new_size: usize) {
        self.cache_entry
            .value_mut()
            .expect("An existing block cannot have a None cache entry")
            .resize(new_size)
            .await;
    }

    async fn remove(self, block_store: &LockingBlockStore<B>) -> Result<()> {
        // TODO Keep cache entry locked until removal is finished
        let block_id = *self.block_id();
        match block_store._remove(&block_id, self.cache_entry).await? {
            RemoveResult::SuccessfullyRemoved => Ok(()),
            RemoveResult::NotRemovedBecauseItDoesntExist => {
                bail!(
                    "Tried to remove a loaded block {:?} but didn't find it",
                    &block_id,
                );
            }
        }
    }
}

impl<B: crate::low_level::LLBlockStore + Send + Sync + Debug + 'static> fmt::Debug
    for LockingBlock<B>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("block_id", self.block_id())
            .field("cache_entry", &self.cache_entry)
            .finish()
    }
}
