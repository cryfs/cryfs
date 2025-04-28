use std::sync::Arc;
use std::{collections::HashSet, fmt::Debug};

use anyhow::Result;
use async_trait::async_trait;
use byte_unit::Byte;
use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::{async_drop::AsyncDropGuard, data::Data};
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt as _, future, stream};

use crate::high_level::Block as _;
use crate::{BlockId, InvalidBlockSizeError, RemoveResult, TryCreateResult};

use super::LockingBlock;
use super::cache::{BlockBaseStoreState, BlockCache, BlockCacheEntryGuard, CacheEntryState};

// TODO Should we require B: OptimizedBlockStoreWriter and use its methods?
pub struct LockingBlockStore<B: crate::low_level::LLBlockStore + Send + Sync + Debug + 'static> {
    // Always Some unless during destruction
    base_store: Option<Arc<AsyncDropGuard<B>>>,

    // cache doubles as a cache for blocks that are being returned and might be
    // re-requested, and as a set of mutexes making sure we don't concurrently
    // do multiple actions on the same block (e.g. remove it while it is loaded).
    cache: AsyncDropGuard<BlockCache<B>>,
}

impl<B: crate::low_level::LLBlockStore + Send + Sync + Debug + 'static> LockingBlockStore<B> {
    pub fn new(base_store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            base_store: Some(Arc::new(base_store)),
            cache: BlockCache::new(),
        })
    }

    pub async fn load(&self, block_id: BlockId) -> Result<Option<LockingBlock<B>>> {
        // TODO Cache non-existence?
        let mut cache_entry = self.cache.async_lock(block_id).await?;
        if cache_entry.value().is_none() {
            let base_store = self.base_store.as_ref().expect("Already destructed");
            let loaded = base_store.load(&block_id).await?;
            if let Some(loaded) = loaded {
                self.cache.set_entry(
                    base_store,
                    &mut cache_entry,
                    loaded,
                    CacheEntryState::Clean,
                    BlockBaseStoreState::ExistsInBaseStore,
                );
            }
        }
        if cache_entry.value().is_some() {
            Ok(Some(LockingBlock { cache_entry }))
        } else {
            Ok(None)
        }
    }

    pub async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult> {
        let mut cache_entry = self.cache.async_lock(*block_id).await?;
        if cache_entry.value().is_some() {
            // Block already exists in the cache
            return Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists);
        }
        let base_store = self.base_store.as_ref().expect("Already destructed");
        if base_store.exists(block_id).await? {
            return Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists);
        }
        self.cache.set_entry(
            base_store,
            &mut cache_entry,
            data.clone(),
            CacheEntryState::Dirty,
            BlockBaseStoreState::DoesntExistInBaseStore,
        );
        Ok(TryCreateResult::SuccessfullyCreated)
    }

    pub async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()> {
        let mut cache_entry = self.cache.async_lock(*block_id).await?;

        let base_store = self.base_store.as_ref().expect("Already destructed");

        let exists_in_base_store = async || {
            if base_store.exists(block_id).await? {
                Ok(BlockBaseStoreState::ExistsInBaseStore)
            } else {
                Ok(BlockBaseStoreState::DoesntExistInBaseStore)
            }
        };

        // Add the new value to the cache.
        self.cache
            .set_or_overwrite_entry_even_if_dirty(
                base_store,
                &mut cache_entry,
                data.clone(),
                CacheEntryState::Dirty,
                exists_in_base_store,
            )
            .await?;

        Ok(())
    }

    pub async fn remove(&self, block_id: &BlockId) -> Result<RemoveResult> {
        let cache_entry_guard = self.cache.async_lock(*block_id).await?;
        self._remove(block_id, cache_entry_guard).await
    }

    pub(super) async fn _remove(
        &self,
        block_id: &BlockId,
        mut cache_entry_guard: BlockCacheEntryGuard<B>,
    ) -> Result<RemoveResult> {
        // TODO Don't write-through but cache remove operations?

        // Remove from cache
        // TODO This is dangerous, we could accidentally drop the cache entry lock if we put it into the let binding by value but it needs to be held while we remove from the base store. Instead make removed_from_base_store a lambda and invoke it from in here?
        let (removed_from_cache, should_remove_from_base_store) =
            if let Some(cache_entry) = cache_entry_guard.value() {
                let should_remove_from_base_store = cache_entry.block_exists_in_base_store()
                    == BlockBaseStoreState::ExistsInBaseStore;
                self.cache
                    .delete_entry_from_cache_even_if_dirty(&mut cache_entry_guard);
                (true, should_remove_from_base_store)
            } else {
                (false, true)
            };

        let removed_from_base_store = if should_remove_from_base_store {
            let base_store = self.base_store.as_ref().expect("Already destructed");
            match base_store.remove(block_id).await? {
                RemoveResult::SuccessfullyRemoved => true,
                RemoveResult::NotRemovedBecauseItDoesntExist => false,
            }
        } else {
            false
        };

        if removed_from_cache || removed_from_base_store {
            Ok(RemoveResult::SuccessfullyRemoved)
        } else {
            Ok(RemoveResult::NotRemovedBecauseItDoesntExist)
        }
    }

    // Note: for any blocks that are created or removed while the returned stream is running,
    // we don't give any guarantees for whether they're counted or not.
    pub async fn num_blocks(&self) -> Result<u64> {
        let base_store = self.base_store.as_ref().expect("Already destructed");
        Ok(base_store.num_blocks().await? + self.cache.num_blocks_in_cache_but_not_in_base_store())
    }

    pub fn estimate_num_free_bytes(&self) -> Result<Byte> {
        let base_store = self.base_store.as_ref().expect("Already destructed");
        base_store.estimate_num_free_bytes()
    }

    pub fn block_size_from_physical_block_size(
        &self,
        block_size: Byte,
    ) -> Result<Byte, InvalidBlockSizeError> {
        let base_store = self.base_store.as_ref().expect("Already destructed");
        base_store.block_size_from_physical_block_size(block_size)
    }

    // Note: for any blocks that are created or removed while the returned stream is running,
    // we don't give any guarantees for whether they'll be part of the stream or not.
    // TODO Make sure we have tests that have some blocks in the cache and some in the base store
    pub async fn all_blocks(&self) -> Result<BoxStream<'static, Result<BlockId>>> {
        let base_store = self.base_store.as_ref().expect("Already destructed");

        // TODO Is keys_with_entries_or_locked the right thing here? Do we want to count locked entries?
        let blocks_in_cache = self.cache.keys_with_entries_or_locked();
        let blocks_in_base_store = base_store.all_blocks().await?;

        let blocks_in_cache_set: HashSet<_> = blocks_in_cache.iter().copied().collect();
        let blocks_in_base_store_and_not_in_cache = blocks_in_base_store
            .try_filter(move |block_id| future::ready(!blocks_in_cache_set.contains(block_id)));

        Ok(stream::iter(blocks_in_cache.into_iter().map(Ok))
            .chain(blocks_in_base_store_and_not_in_cache)
            .boxed())
    }

    pub async fn create(&self, data: &Data) -> Result<BlockId> {
        loop {
            let block_id = BlockId::new_random();
            // TODO try_create first checks if a block id exists before creating it. That's expensive. Is the collision probability low enough that we can skip this?
            //      To make things worse, OnDiskBlockStore::try_create() is called when the cache is flushed and actually checks existence a second time before storing.
            let created = self.try_create(&block_id, data).await?;
            match created {
                TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => { /* just continue */ }
                TryCreateResult::SuccessfullyCreated => {
                    return Ok(block_id);
                }
            }
        }
    }

    pub async fn flush_block(&self, block: &mut LockingBlock<B>) -> Result<()> {
        let block_id = *block.block_id();
        let entry = block
            .cache_entry
            .value_mut()
            .expect("An existing block cannot have a None cache entry");
        self.cache.flush_block(entry, &block_id).await
    }

    pub async fn into_inner_block_store(this: AsyncDropGuard<Self>) -> Result<AsyncDropGuard<B>> {
        let mut this = this.unsafe_into_inner_dont_drop();
        // TODO Exception safety. Drop base_store if dropping the cache fails.
        this.cache.async_drop().await?;

        let base_store = this.base_store.take().expect("Already destructed");
        let base_store = Arc::try_unwrap(base_store).expect("We should be the only ones with access to self.base_store, but seems there is still something else accessing it");
        Ok(base_store)
    }

    /// clear_cache_slow is only used in test cases. Without test cases calling it, they would only
    /// ever test cached blocks and never have to store/reload them to the base store.
    /// This is implemented in a very slow way and shouldn't be used in non-test code.
    #[cfg(any(test, feature = "testutils"))]
    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.cache.prune_all_blocks().await
    }

    #[cfg(any(test, feature = "testutils"))]
    pub async fn clear_unloaded_blocks_from_cache(&self) -> Result<()> {
        self.cache.prune_unloaded_blocks().await
    }
}

#[async_trait]
impl<B: crate::low_level::LLBlockStore + Send + Sync + Debug + 'static> AsyncDrop
    for LockingBlockStore<B>
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
        // TODO Exception safety. Should we drop base_store even if dropping the cache fails?
        self.cache.async_drop().await?;

        // Since we just dropped the cache, we know there are no cache entries left with access to the self.base_store Arc.
        // This also means there can't be any other tasks/threads currently locking cache entries and doing things with it,
        // we're truly the only one with access to self.base_store.
        let base_store = self.base_store.take().expect("Already destructed");
        let mut base_store = Arc::try_unwrap(base_store).expect("We should be the only ones with access to self.base_store, but seems there is still something else accessing it");
        base_store.async_drop().await?;

        Ok(())
    }
}

impl<B: crate::low_level::LLBlockStore + Send + Sync + Debug + 'static> Debug
    for LockingBlockStore<B>
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("LockingBlockStore").finish()
    }
}
