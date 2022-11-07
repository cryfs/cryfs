use anyhow::Result;
use async_trait::async_trait;
use futures::join;
use lockable::{Lockable, LockableLruCache};
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use tokio::time::Duration;

use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};
use crate::utils::periodic_task::PeriodicTask;
use crate::utils::stream::for_each_unordered;

mod cache_impl;
mod entry;
mod guard;

use cache_impl::BlockCacheImpl;
pub use entry::{BlockBaseStoreState, BlockCacheEntry, CacheEntryState};
pub use guard::BlockCacheEntryGuard;

// How often to run the task to prune old blocks
const PRUNE_BLOCKS_INTERVAL: Duration = Duration::from_millis(500);
// The cutoff age of blocks. Each time the task runs, blocks older than this will be pruned.
const PRUNE_BLOCKS_OLDER_THAN: Duration = Duration::from_millis(500);

pub struct BlockCache<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> {
    // Always Some except during destruction
    cache: Option<Arc<BlockCacheImpl<B>>>,
    // Always Some except during destruction
    prune_task: Option<AsyncDropGuard<PeriodicTask>>,
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> BlockCache<B> {
    pub fn new() -> AsyncDropGuard<Self> {
        let cache = BlockCacheImpl::new();
        let cache_clone = Arc::clone(&cache);
        AsyncDropGuard::new(Self {
            cache: Some(cache),
            prune_task: Some(PeriodicTask::spawn(
                "BlockCache::prune",
                PRUNE_BLOCKS_INTERVAL,
                move || {
                    let cache_clone = Arc::clone(&cache_clone);
                    async move { Self::_prune_old_blocks(cache_clone).await }
                },
            )),
        })
    }

    pub async fn async_lock(&self, block_id: BlockId) -> Result<BlockCacheEntryGuard<B>> {
        let cache = Arc::clone(self.cache.as_ref().expect("Object is already destructed"));
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .async_lock(block_id, move |evicted| {
                Self::_prune_blocks(Arc::clone(&cache), evicted.into_iter())
            })
            .await
    }

    pub fn keys_with_entries_or_locked(&self) -> Vec<BlockId> {
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .keys_with_entries_or_locked()
    }

    pub fn delete_entry_from_cache_even_if_dirty(&self, entry: &mut BlockCacheEntryGuard<B>) {
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .delete_entry_from_cache_even_if_dirty(&mut entry.guard);
    }

    pub fn set_entry(
        &self,
        base_store: &Arc<AsyncDropGuard<B>>,
        entry: &mut BlockCacheEntryGuard<B>,
        new_value: Data,
        dirty: CacheEntryState,
        base_store_state: BlockBaseStoreState,
    ) {
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .set_entry(base_store, entry, new_value, dirty, base_store_state);
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
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .set_or_overwrite_entry_even_if_dirty(
                base_store,
                entry,
                new_value,
                dirty,
                base_store_state,
            )
            .await
    }

    pub fn num_blocks_in_cache_but_not_in_base_store(&self) -> u64 {
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .num_blocks_in_cache_but_not_in_base_store()
    }

    async fn _prune_old_blocks(cache: Arc<BlockCacheImpl<B>>) -> Result<()> {
        Self::_prune_blocks_not_accessed_for_at_least(cache, PRUNE_BLOCKS_OLDER_THAN).await
    }

    /// TODO Docs
    /// TODO Test
    async fn _prune_blocks_not_accessed_for_at_least(
        cache: Arc<BlockCacheImpl<B>>,
        duration: Duration,
    ) -> Result<()> {
        let to_prune = cache.lock_entries_unlocked_for_at_least(duration);
        Self::_prune_blocks(cache, to_prune).await
    }

    /// TODO Docs
    /// TODO Test
    #[cfg(test)]
    pub async fn prune_all_blocks(&self) -> Result<()> {
        use futures::StreamExt;
        let cache = self.cache.as_ref().expect("Object is already destructed");
        let to_prune = cache
            .lock_all_entries()
            .await
            // TODO Is this possible by directly processing the stream and not collecting into a Vec?
            .collect::<Vec<_>>()
            .await
            .into_iter();
        Self::_prune_blocks(Arc::clone(cache), to_prune).await
    }

    async fn _prune_blocks(
        cache: Arc<BlockCacheImpl<B>>,
        to_prune: impl Iterator<
            Item = <LockableLruCache<BlockId, BlockCacheEntry<B>> as Lockable<
                BlockId,
                BlockCacheEntry<B>,
            >>::OwnedGuard,
        >,
    ) -> Result<()> {
        // Now we have a list of mutex guards, locking all keys that we want to prune.
        // The global mutex for the cache is unlocked, so other threads may now come in
        // and could get one of those mutexes, waiting for them to lock. To address this,
        // we change the cache entry to None after pruning, which will cause the guard
        // to return it to the cache using LockableCache::_unlock(), which will then check if
        // other tasks are waiting and only remove it if no other tasks are waiting.
        // TODO Test what the previous paragraph describes
        for_each_unordered(to_prune, |guard| Self::_prune_block(&cache, guard)).await
    }

    async fn _prune_block(
        cache: &BlockCacheImpl<B>,
        mut guard: <LockableLruCache<BlockId, BlockCacheEntry<B>> as Lockable<
            BlockId,
            BlockCacheEntry<B>,
        >>::OwnedGuard,
    ) -> Result<()> {
        // Write back the block data
        let block_id = *guard.key();
        if let Some(entry) = guard.value_mut() {
            cache.flush_entry(entry, &block_id).await?;
            cache.delete_entry_from_cache(&mut guard);
        } else {
            // Found a None entry in the cache.
            // This can for example happen when the cache entry was deleted while we were waiting for our lock on it.
        }

        Ok(())
    }

    pub async fn flush_block(
        &self,
        block: &mut BlockCacheEntry<B>,
        block_id: &BlockId,
    ) -> Result<()> {
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .flush_entry(block, block_id)
            .await
    }
}

#[async_trait]
impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> AsyncDrop
    for BlockCache<B>
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        let mut prune_task = self
            .prune_task
            .take()
            .expect("Object was already destructed");
        let stop_prune_task = async move { prune_task.async_drop().await };
        let drop_entries = async move {
            // The self.cache arc is shared between the prune task and self.
            // Since self is passed in by value, prune task is the only one
            // that also has an instance. We're dropping prune_task concurrently
            // with this task. So let's wait until it has dropped it
            let cache = self.cache.take().expect("Object is already destructed");
            while Arc::strong_count(&cache) > 1 {
                // TODO Is there a better alternative that doesn't involve busy waiting?
                tokio::task::yield_now().await;
            }
            // Now we're the only task having access to this arc
            let cache = Arc::try_unwrap(cache)
                .expect("This can't fail since we are the only task having access");
            for_each_unordered(
                cache.into_entries_unordered().await,
                |(key, mut value)| async move {
                    value._flush_to_base_store(&key).await?;
                    Ok(())
                },
            )
            .await

            // TODO We want this assertion but can't do it here since we already moved out of it and can't it in BlockCacheImpl destructor since that runs while there are still guards alive.
            // assert_eq!(0, self.num_blocks_in_cache_but_not_in_base_store(), "We somehow miscounted num_blocks_in_cache_but_not_in_base_store");
        };
        let (stop_prune_task, drop_entries) = join!(stop_prune_task, drop_entries);
        // TODO Report multiple errors if both stop_prune_task and drop_entries fail
        stop_prune_task?;
        drop_entries?;

        Ok(())
    }
}

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> Debug
    for BlockCache<B>
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("BlockCache").finish()
    }
}
