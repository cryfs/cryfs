use anyhow::Result;
use async_trait::async_trait;
use futures::join;
use futures::{future, stream::FuturesUnordered, StreamExt};
use lockable::LruGuard;
use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;
use tokio::time::Duration;

use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};
use crate::utils::periodic_task::PeriodicTask;

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
                    async move { Self::_prune_old_blocks(&cache_clone).await }
                },
            )),
        })
    }

    pub async fn async_lock(&self, block_id: BlockId) -> BlockCacheEntryGuard<B> {
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .async_lock(block_id)
            .await
    }

    pub fn keys(&self) -> Vec<BlockId> {
        self.cache
            .as_ref()
            .expect("Object is already destructed")
            .keys()
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

    async fn _prune_old_blocks(cache: &BlockCacheImpl<B>) -> Result<()> {
        Self::_prune_blocks_not_accessed_for_longer_than(cache, PRUNE_BLOCKS_OLDER_THAN).await
    }

    /// TODO Docs
    /// TODO Test
    async fn _prune_blocks_not_accessed_for_longer_than(
        cache: &BlockCacheImpl<B>,
        duration: Duration,
    ) -> Result<()> {
        let to_prune = cache.lock_entries_unlocked_for_longer_than(duration);
        // Now we have a list of mutex guards, locking all keys that we want to prune.
        // The global mutex for the cache is unlocked, so other threads may now come in
        // and could get one of those mutexes, waiting for them to lock. To address this,
        // we change the cache entry to None after pruning, which will cause the guard
        // to return it to the cache using LockableCache::_unlock(), which will then check if
        // other tasks are waiting and only remove it if no other tasks are waiting.
        // TODO Test what the previous paragraph describes
        let pruning_tasks: FuturesUnordered<_> = to_prune
            .map(|guard| Self::_prune_block(cache, guard))
            .collect();
        let errors = pruning_tasks
            .filter(|result| future::ready(result.is_err()))
            .map(|result| result.unwrap_err());
        let errors: Vec<anyhow::Error> = errors.collect().await;
        for error in &errors {
            // Log all errors since we can only return one even if multiple ones happen
            // TODO Better to introduce a special MultiError class? I also think there's another place somewhere where we log errors instead of returning multiple, fix that too.
            log::error!("Error in prune_items_unlocked_for_longer_than: {:?}", error);
        }
        if let Some(first_error) = errors.into_iter().next() {
            Err(first_error)
        } else {
            Ok(())
        }
    }

    async fn _prune_block<'a>(
        cache: &BlockCacheImpl<B>,
        mut guard: LruGuard<'a, BlockId, BlockCacheEntry<B>>,
    ) -> Result<()> {
        // Write back the block data
        let block_id = *guard.key();
        let entry = guard
            .value_mut()
            .expect("Found a None entry in the cache. This violates our invariant.");
        entry.flush(&block_id).await?;

        cache.delete_entry_from_cache(&mut guard);

        Ok(())
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
            let entries: FuturesUnordered<_> = cache
                .into_entries_unordered()
                .await
                .map(future::ready)
                .collect();

            let errors = entries.filter_map(|(key, mut value)| async move {
                let result = value.flush(&key).await;
                match result {
                    Ok(()) => None,
                    Err(err) => Some(err),
                }
            });
            let mut errors = Box::pin(errors);
            let mut first_error = None;

            // This while loop drives the whole stream (successes and errors) but only enters the loop body for errors.
            while let Some(error) = errors.next().await {
                if first_error.is_none() {
                    first_error = Some(error);
                } else {
                    // TODO Return a list of all errors instead of logging swallowed ones
                    log::error!("Error in BlockCache::async_drop_impl: {:?}", error);
                }
            }

            // TODO We want this assertion but can't do it here since we already moved out of it and can't it in BlockCacheImpl destructor since that runs while there are still guards alive.
            // assert_eq!(0, self.num_blocks_in_cache_but_not_in_base_store(), "We somehow miscounted num_blocks_in_cache_but_not_in_base_store");

            if let Some(error) = first_error {
                Err(error)
            } else {
                Ok(())
            }
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
