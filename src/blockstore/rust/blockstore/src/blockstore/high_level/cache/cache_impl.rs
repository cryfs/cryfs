use anyhow::Result;
use futures::{
    Stream,
};
use std::sync::atomic::{Ordering, AtomicU64};
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;
use tokio::{
    time::Duration,
};
use std::future::Future;

use super::lockable_cache::{Guard, LockableCache, GuardImpl};
use super::entry::{BlockCacheEntry, BlockBaseStoreState, CacheEntryState};
use super::guard::BlockCacheEntryGuard;
use crate::blockstore::BlockId;
use crate::data::Data;

pub struct BlockCacheImpl<B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static> {
    // Only None while it is being dropped
    cache: Option<Arc<LockableCache<BlockId, BlockCacheEntry<B>>>>,

    // This variable counts how many blocks in the cache are not in the base store.
    // Since this isn't protected by the same mutex as cache, it is only eventually consistent.
    // While operations are adding or removing entries from the cache or the base store,
    // this may temporarily have the wrong value.
    num_blocks_in_cache_but_not_in_base_store: AtomicU64,
}

impl <B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static> BlockCacheImpl<B> {
    pub fn new() -> Arc<Self> {
        Arc::new(BlockCacheImpl {
            cache: Some(Arc::new(LockableCache::new())),
            num_blocks_in_cache_but_not_in_base_store: 0.into(),
        })
    }

    fn _cache(&self) -> &Arc<LockableCache<BlockId, BlockCacheEntry<B>>> {
        &self.cache.as_ref().expect("Instance is currently being dropped")
    }

    pub fn keys(&self) -> Vec<BlockId> {
        self._cache().keys()
    }

    pub async fn async_lock(&self, block_id: BlockId) -> BlockCacheEntryGuard<B> {
        let guard = self._cache().async_lock_owned(block_id).await;
        BlockCacheEntryGuard {guard}
    }

    pub fn delete_entry_from_cache<'a>(&self, entry: &mut Guard<'a, BlockId, BlockCacheEntry<B>>) {
        let entry = entry.take().expect("Tried to delete an entry that wasn't set");
        if entry.block_exists_in_base_store() == BlockBaseStoreState::DoesntExistInBaseStore {
            let prev = self.num_blocks_in_cache_but_not_in_base_store.fetch_sub(1, Ordering::SeqCst);
            assert!(prev > 0, "Underflow in num_blocks_in_cache_but_not_in_base_store");
        }

        // This will cause BlockCacheEntry to get destructed and that'll trigger a panic if it was dirty.
        // Since entry is now None, when the Guard is dropped and calls LockableCache::_unlock, it will remove the entry from the cache.
    }

    pub fn delete_entry_from_cache_even_if_dirty<C: Deref<Target=LockableCache<BlockId, BlockCacheEntry<B>>>>(&self, entry: &mut GuardImpl<BlockId, BlockCacheEntry<B>, C>) {
        let old_entry = std::mem::replace(&mut **entry, None);

        let old_entry = old_entry.expect("Tried to delete an entry that wasn't set");

        if old_entry.block_exists_in_base_store() == BlockBaseStoreState::DoesntExistInBaseStore {
            let prev = self.num_blocks_in_cache_but_not_in_base_store.fetch_sub(1, Ordering::SeqCst);
            assert!(prev > 0, "Underflow in num_blocks_in_cache_but_not_in_base_store");
        }

        // Now the old cache entry is in the old_entry variable and we need to discard it
        // so we don't trigger a panic when it gets destructed and is dirty.
        old_entry.discard();

        // Since entry is now None, when the Guard is dropped and calls LockableCache::_unlock, it will remove the entry from the cache.
    }

    pub fn set_entry(&self, base_store: &Arc<B>, entry: &mut BlockCacheEntryGuard<B>, new_value: Data, dirty: CacheEntryState, base_store_state: BlockBaseStoreState) {
        assert!(entry.is_none(), "Can only set an entry if it wasn't set beforehand. Otherwise, use overwrite_entry");
        if base_store_state == BlockBaseStoreState::DoesntExistInBaseStore {
            assert!(dirty == CacheEntryState::Dirty, "If it doesn't exist in the base store, it must be dirty");
            self.num_blocks_in_cache_but_not_in_base_store.fetch_add(1, Ordering::SeqCst);
        }
        **entry = Some(BlockCacheEntry::new(Arc::clone(base_store), new_value, dirty, base_store_state));
    }

    pub async fn set_or_overwrite_entry_even_if_dirty<F>(&self, base_store: &Arc<B>, entry: &mut BlockCacheEntryGuard<B>, new_value: Data, dirty: CacheEntryState, base_store_state: impl FnOnce() -> F) -> Result<()> where F: Future<Output = Result<BlockBaseStoreState>> {
        let base_store_state = if let Some(entry) = &**entry {
            entry.block_exists_in_base_store()
        } else {
            let base_store_state = base_store_state().await?;
            if base_store_state == BlockBaseStoreState::DoesntExistInBaseStore {
                assert!(dirty == CacheEntryState::Dirty, "If it doesn't exist in the base store, it must be dirty");
                self.num_blocks_in_cache_but_not_in_base_store.fetch_add(1, Ordering::SeqCst);
            }
            base_store_state
        };

        let old_entry = std::mem::replace(&mut **entry, Some(BlockCacheEntry::new(Arc::clone(base_store), new_value, dirty, base_store_state)));
        // Now the old cache entry is in the old_entry variable and we need to discard it
        // so we don't trigger a panic when it gets destructed and is dirty.
        if let Some(old_entry) = old_entry {
            old_entry.discard();
        }
        Ok(())
    }

    pub fn num_blocks_in_cache_but_not_in_base_store(&self) -> u64 {
        self.num_blocks_in_cache_but_not_in_base_store.load(Ordering::SeqCst)
    }

    pub fn lock_entries_unlocked_for_longer_than(
        &self,
        duration: Duration,
    ) -> Vec<Guard<'_, BlockId, BlockCacheEntry<B>>> {
        self._cache().lock_entries_unlocked_for_longer_than(duration)
    }

    pub async fn into_entries_unordered(mut self) -> impl Stream<Item = (BlockId, BlockCacheEntry<B>)> {
        // Since self is passed in by value, we know that we're the only task with
        // access to this instance. No other task or thread can increase the refcount of the
        // Arc around self.cache. Other threads can still hold BlockCacheEntryGuard
        // instances and hold instances of this Arc, but the encapsulation in BlockCacheEntryGuard
        // doesn't allow them to increase the refcount since it is un-cloneable.
        // We just need to wait until the last thread/task destructs their BlockCacheEntryGuard
        // and then we can go ahead and destruct the cache.
        // This will also wait until self.prune_task is far enough in being stopped to have
        // released its clone of the Arc.
        let cache = self._cache();
        while Arc::strong_count(&*cache) > 1 {
            // TODO Is there a better alternative that doesn't involve busy waiting?
            tokio::task::yield_now().await;
        }
        // Now we're the only task having access to this arc
        let cache = Arc::try_unwrap(self.cache.take().expect("Value is already being dropped, this can't happen"))
            .expect("This can't fail since we are the only task having access");
        cache.into_entries_unordered()
    }
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static> Debug for BlockCacheImpl<B> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("BlockCacheImpl").finish()
    }
}
