use anyhow::Result;
use std::fmt::{self, Debug};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::AsyncDropGuard;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CacheEntryState {
    Dirty,
    Clean,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockBaseStoreState {
    ExistsInBaseStore,
    DoesntExistInBaseStore,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum FlushResult {
    FlushingAddedANewBlockToTheBaseStore,
    FlushingDidntAddANewBlockToTheBaseStoreBecauseItAlreadyExistedInTheBaseStore,
    FlushingDidntAddANewBlockToTheBaseStoreBecauseCacheEntryWasntDirty,
}

pub struct BlockCacheEntry<
    B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static,
> {
    // TODO Do we really need to store the base_store in each cache entry? It's only used in flush().
    base_store: Arc<AsyncDropGuard<B>>,
    dirty: CacheEntryState,
    data: Data,
    block_exists_in_base_store: BlockBaseStoreState,
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static>
    BlockCacheEntry<B>
{
    #[inline]
    pub fn new(
        base_store: Arc<AsyncDropGuard<B>>,
        data: Data,
        dirty: CacheEntryState,
        block_exists_in_base_store: BlockBaseStoreState,
    ) -> Self {
        Self {
            base_store,
            dirty,
            data,
            block_exists_in_base_store,
        }
    }

    #[inline]
    pub fn block_exists_in_base_store(&self) -> BlockBaseStoreState {
        self.block_exists_in_base_store
    }

    #[inline]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut Data {
        self.dirty = CacheEntryState::Dirty;
        &mut self.data
    }

    // Warning: _flush_to_base_store doesn't update BlockCacheImpl.num_blocks_in_cache_but_not_in_base_store.
    // It shouldn't be used directly but we should use BlockCacheImpl.flush_entry() instead,
    // unless we're certain that the bad state in BlockCacheImpl doesn't matter (e.g. because the BlockCacheImpl
    // instance is already dead)
    pub(super) async fn _flush_to_base_store(&mut self, block_id: &BlockId) -> Result<FlushResult> {
        if self.dirty == CacheEntryState::Dirty {
            // TODO self.base_store.optimized_store() ?
            self.base_store.store(block_id, &self.data).await?;
            self.dirty = CacheEntryState::Clean;
            match self.block_exists_in_base_store {
                BlockBaseStoreState::ExistsInBaseStore => {
                    Ok(FlushResult::FlushingDidntAddANewBlockToTheBaseStoreBecauseItAlreadyExistedInTheBaseStore)
                },
                BlockBaseStoreState::DoesntExistInBaseStore => {
                    self.block_exists_in_base_store = BlockBaseStoreState::ExistsInBaseStore;
                    Ok(FlushResult::FlushingAddedANewBlockToTheBaseStore)
                }
            }
        } else {
            Ok(FlushResult::FlushingDidntAddANewBlockToTheBaseStoreBecauseCacheEntryWasntDirty)
        }
    }

    #[inline]
    pub async fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size);
        self.dirty = CacheEntryState::Dirty;
    }

    #[inline]
    pub(super) fn discard(mut self) {
        self.dirty = CacheEntryState::Clean;
        // now that dirty is false, the value can be safely dropped
    }
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> fmt::Debug
    for BlockCacheEntry<B>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockCacheEntry")
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> Drop
    for BlockCacheEntry<B>
{
    fn drop(&mut self) {
        // User code never gets access to BlockCacheEntry by value, so they can't do this mistake.
        // If a dirty block is really dropped, it is our mistake.
        if self.dirty != CacheEntryState::Clean {
            if std::thread::panicking() {
                // We're already panicking, double panic wouldn't show a good error message anyways. Let's just log instead.
                // A common scenario for this to happen is a failing test case.
                log::error!("Tried to drop a dirty block. Please call flush() first");
            } else {
                panic!("Tried to drop a dirty block. Please call flush() first");
            }
        }
    }
}
