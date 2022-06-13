use anyhow::Result;
use std::fmt::{self, Debug};
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

pub struct BlockCacheEntry<
    B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static,
> {
    // TODO Do we really need to store the base_store in each cache entry?
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

    pub async fn flush(&mut self, block_id: &BlockId) -> Result<()> {
        if self.dirty == CacheEntryState::Dirty {
            // TODO self.base_store.optimized_store() ?
            self.base_store.store(block_id, &self.data).await?;
            self.dirty = CacheEntryState::Clean;
        }
        Ok(())
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
