use anyhow::Result;
use futures::stream::Stream;
use lockable::{AsyncLimit, LockableLruCache, LruOwnedGuard};
use std::fmt::Debug;
use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::Duration;
use std::num::NonZeroUsize;

use super::entry::{BlockBaseStoreState, BlockCacheEntry, CacheEntryState};
use super::guard::BlockCacheEntryGuard;
use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::AsyncDropGuard;

// TODO Replace unsafe{NonZeroUSize::new_unchecked(_)} with NonZeroUsize::new(_).unwrap() once unwrap is const
const MAX_CACHE_ENTRIES: NonZeroUsize = unsafe {NonZeroUsize::new_unchecked(1024)};

pub struct BlockCacheImpl<
    B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static,
> {
    // Only None while it is being dropped
    cache: Option<Arc<LockableLruCache<BlockId, BlockCacheEntry<B>>>>,

    // This variable counts how many blocks in the cache are not in the base store.
    // Since this isn't protected by the same mutex as cache, it is only eventually consistent.
    // While operations are adding or removing entries from the cache or the base store,
    // this may temporarily have the wrong value.
    num_blocks_in_cache_but_not_in_base_store: AtomicU64,
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static>
    BlockCacheImpl<B>
{
    pub fn new() -> Arc<Self> {
        Arc::new(BlockCacheImpl {
            cache: Some(Arc::new(LockableLruCache::new())),
            num_blocks_in_cache_but_not_in_base_store: 0.into(),
        })
    }

    fn _cache(&self) -> &Arc<LockableLruCache<BlockId, BlockCacheEntry<B>>> {
        &self
            .cache
            .as_ref()
            .expect("Instance is currently being dropped")
    }

    pub fn keys(&self) -> Vec<BlockId> {
        self._cache().keys()
    }

    pub async fn async_lock<F, OnEvictFn>(
        &self,
        block_id: BlockId,
        on_evict: OnEvictFn,
    ) -> Result<BlockCacheEntryGuard<B>>
    where
        F: Future<Output = Result<()>>,
        OnEvictFn: Fn(Vec<LruOwnedGuard<BlockId, BlockCacheEntry<B>>>) -> F,
    {
        let guard = self
            ._cache()
            .async_lock_owned(
                block_id,
                AsyncLimit::Bounded {
                    max_entries: MAX_CACHE_ENTRIES,
                    on_evict: move |evicted| {
                        // TODO Should we wrap this into a BlockCacheEntryGuard for better abstraction separation?
                        on_evict(evicted)
                    },
                },
            )
            .await?;
        Ok(BlockCacheEntryGuard { guard })
    }

    pub fn delete_entry_from_cache(&self, entry: &mut LruOwnedGuard<BlockId, BlockCacheEntry<B>>) {
        let entry = entry
            .remove()
            .expect("Tried to delete an entry that wasn't set");
        if entry.block_exists_in_base_store() == BlockBaseStoreState::DoesntExistInBaseStore {
            let prev = self
                .num_blocks_in_cache_but_not_in_base_store
                .fetch_sub(1, Ordering::SeqCst);
            assert!(
                prev > 0,
                "Underflow in num_blocks_in_cache_but_not_in_base_store"
            );
        }

        // This will cause BlockCacheEntry to get destructed and that'll trigger a panic if it was dirty.
        // Since entry is now None, when the Guard is dropped and calls LockableCache::_unlock, it will remove the entry from the cache.
    }

    pub fn delete_entry_from_cache_even_if_dirty(
        &self,
        entry: &mut LruOwnedGuard<BlockId, BlockCacheEntry<B>>,
    ) {
        let old_entry = entry
            .remove()
            .expect("Tried to delete an entry that wasn't set");

        if old_entry.block_exists_in_base_store() == BlockBaseStoreState::DoesntExistInBaseStore {
            let prev = self
                .num_blocks_in_cache_but_not_in_base_store
                .fetch_sub(1, Ordering::SeqCst);
            assert!(
                prev > 0,
                "Underflow in num_blocks_in_cache_but_not_in_base_store"
            );
        }

        // Now the old cache entry is in the old_entry variable and we need to discard it
        // so we don't trigger a panic when it gets destructed and is dirty.
        old_entry.discard();

        // Since entry is now None, when the Guard is dropped and calls LockableCache::_unlock, it will remove the entry from the cache.
    }

    pub fn set_entry(
        &self,
        base_store: &Arc<AsyncDropGuard<B>>,
        entry: &mut BlockCacheEntryGuard<B>,
        new_value: Data,
        dirty: CacheEntryState,
        base_store_state: BlockBaseStoreState,
    ) {
        assert!(
            entry.value().is_none(),
            "Can only set an entry if it wasn't set beforehand. Otherwise, use overwrite_entry"
        );
        if base_store_state == BlockBaseStoreState::DoesntExistInBaseStore {
            assert!(
                dirty == CacheEntryState::Dirty,
                "If it doesn't exist in the base store, it must be dirty"
            );
            self.num_blocks_in_cache_but_not_in_base_store
                .fetch_add(1, Ordering::SeqCst);
        }
        let old_entry = entry.insert(BlockCacheEntry::new(
            Arc::clone(base_store),
            new_value,
            dirty,
            base_store_state,
        ));
        assert!(
            old_entry.is_none(),
            "We checked above already that the entry isn't set"
        );
    }

    pub async fn set_or_overwrite_entry_even_if_dirty<F>(
        &self,
        base_store: &Arc<AsyncDropGuard<B>>,
        entry: &mut BlockCacheEntryGuard<B>,
        new_value: Data,
        dirty: CacheEntryState,
        base_store_state: impl FnOnce() -> F,
    ) -> Result<()>
    where
        F: Future<Output = Result<BlockBaseStoreState>>,
    {
        let base_store_state = if let Some(entry) = entry.value() {
            entry.block_exists_in_base_store()
        } else {
            let base_store_state = base_store_state().await?;
            if base_store_state == BlockBaseStoreState::DoesntExistInBaseStore {
                assert!(
                    dirty == CacheEntryState::Dirty,
                    "If it doesn't exist in the base store, it must be dirty"
                );
                self.num_blocks_in_cache_but_not_in_base_store
                    .fetch_add(1, Ordering::SeqCst);
            }
            base_store_state
        };

        let old_entry = entry.insert(BlockCacheEntry::new(
            Arc::clone(base_store),
            new_value,
            dirty,
            base_store_state,
        ));
        // Now the old cache entry is in the old_entry variable and we need to discard it
        // so we don't trigger a panic when it gets destructed and is dirty.
        if let Some(old_entry) = old_entry {
            old_entry.discard();
        }
        Ok(())
    }

    pub fn num_blocks_in_cache_but_not_in_base_store(&self) -> u64 {
        self.num_blocks_in_cache_but_not_in_base_store
            .load(Ordering::SeqCst)
    }

    pub fn lock_entries_unlocked_for_at_least(
        &self,
        duration: Duration,
    ) -> impl Iterator<Item = LruOwnedGuard<BlockId, BlockCacheEntry<B>>> {
        self._cache()
            .lock_entries_unlocked_for_at_least_owned(duration)
    }

    pub async fn lock_all_entries(
        &self,
    ) -> impl Stream<Item = LruOwnedGuard<BlockId, BlockCacheEntry<B>>> {
        self._cache().lock_all_entries_owned().await
    }

    pub async fn into_entries_unordered(
        mut self,
    ) -> impl Iterator<Item = (BlockId, BlockCacheEntry<B>)> {
        // At this point, we have exclusive access to self, but there may still be other threads/tasks having access to
        // a clone of the Arc containing self.cache. The only way for them to hold such a copy is through a
        // LruOwnedGuard that locks one of the block ids. LruOwnedGuard cannot be cloned. It is reasonable to assume that
        // these threads/tasks will at some point release their lock. Since we have exclusive access to the self object,
        // We know that no new threads/tasks can acquire such a lock. We just have to wait until the last thread/task releases their lock.
        // This will also wait until self.prune_task is far enough in being stopped to have
        // released its clone of the Arc.
        // However, note that there is still a chance of a deadlock here. If one of those threads is the current thread, or if one of
        // those threads waits for the current thread on something, then we have a deadlock.
        // TODO Is there a better way to handle this?
        let cache = self._cache();
        while Arc::strong_count(&*cache) > 1 {
            // TODO Is there a better alternative that doesn't involve busy waiting?
            tokio::task::yield_now().await;
        }
        // Now we're the only task having access to this arc
        let cache = Arc::try_unwrap(
            self.cache
                .take()
                .expect("Value is already being dropped, this can't happen"),
        )
        .expect("This can't fail since we are the only task having access");
        cache.into_entries_unordered()
    }
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> Debug
    for BlockCacheImpl<B>
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("BlockCacheImpl").finish()
    }
}
