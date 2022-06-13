use anyhow::Result;
use async_trait::async_trait;
use futures::{
    future,
    stream::{self, Stream, StreamExt, TryStreamExt},
};
use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::pin::Pin;
use std::sync::Arc;

use crate::blockstore::{low_level::BlockStore, BlockId};
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

mod cache;
use cache::{BlockBaseStoreState, BlockCache, BlockCacheEntryGuard, CacheEntryState};

pub struct Block<B: BlockStore + Send + Sync + Debug + 'static> {
    cache_entry: BlockCacheEntryGuard<B>,
}

impl<B: super::low_level::BlockStore + Send + Sync + Debug> Block<B> {
    #[inline]
    pub fn block_id(&self) -> &BlockId {
        &self.cache_entry.key()
    }

    #[inline]
    pub fn data(&self) -> &Data {
        self.cache_entry
            .value()
            .expect("An existing block cannot have a None cache entry")
            .data()
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut Data {
        self.cache_entry
            .value_mut()
            .expect("An existing block cannot have a None cache entry")
            .data_mut()
    }

    pub async fn flush(&mut self) -> Result<()> {
        let block_id = *self.block_id();
        self.cache_entry
            .value_mut()
            .expect("An existing block cannot have a None cache entry")
            .flush(&block_id)
            .await
    }

    pub async fn resize(&mut self, new_size: usize) {
        self.cache_entry
            .value_mut()
            .expect("An existing block cannot have a None cache entry")
            .resize(new_size)
            .await;
    }
}

impl<B: super::low_level::BlockStore + Send + Sync + Debug + 'static> fmt::Debug for Block<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("block_id", self.block_id())
            .field("cache_entry", &self.cache_entry)
            .finish()
    }
}

// TODO Should we require B: OptimizedBlockStoreWriter and use its methods?
pub struct LockingBlockStore<B: super::low_level::BlockStore + Send + Sync + Debug + 'static> {
    // Always Some unless during destruction
    base_store: Option<Arc<AsyncDropGuard<B>>>,

    // cache doubles as a cache for blocks that are being returned and might be
    // re-requested, and as a set of mutexes making sure we don't concurrently
    // do multiple actions on the same block (e.g. remove it while it is loaded).
    cache: AsyncDropGuard<BlockCache<B>>,
}

impl<B: super::low_level::BlockStore + Send + Sync + Debug + 'static> LockingBlockStore<B> {
    pub fn new(base_store: AsyncDropGuard<B>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            base_store: Some(Arc::new(base_store)),
            cache: BlockCache::new(),
        })
    }

    pub async fn load(&self, block_id: BlockId) -> Result<Option<Block<B>>> {
        // TODO Cache non-existence?
        let mut cache_entry = self.cache.async_lock(block_id).await;
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
            Ok(Some(Block { cache_entry }))
        } else {
            Ok(None)
        }
    }

    pub async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult> {
        let mut cache_entry = self.cache.async_lock(*block_id).await;
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
        let mut cache_entry = self.cache.async_lock(*block_id).await;

        let base_store = self.base_store.as_ref().expect("Already destructed");

        let exists_in_base_store = || async {
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
        // TODO Don't write-through but cache remove operations?

        let mut cache_entry_guard = self.cache.async_lock(*block_id).await;

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
                crate::blockstore::low_level::RemoveResult::SuccessfullyRemoved => true,
                crate::blockstore::low_level::RemoveResult::NotRemovedBecauseItDoesntExist => false,
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

    pub fn estimate_num_free_bytes(&self) -> Result<u64> {
        let base_store = self.base_store.as_ref().expect("Already destructed");
        base_store.estimate_num_free_bytes()
    }

    pub fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        let base_store = self.base_store.as_ref().expect("Already destructed");
        base_store.block_size_from_physical_block_size(block_size)
    }

    // Note: for any blocks that are created or removed while the returned stream is running,
    // we don't give any guarantees for whether they'll be part of the stream or not.
    // TODO Make sure we have tests that have some blocks in the cache and some in the base store
    pub async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        let base_store = self.base_store.as_ref().expect("Already destructed");

        let blocks_in_cache = self.cache.keys();
        let blocks_in_base_store = base_store.all_blocks().await?;

        let blocks_in_cache_set: HashSet<_> = blocks_in_cache.iter().copied().collect();
        let blocks_in_base_store_and_not_in_cache = blocks_in_base_store
            .try_filter(move |block_id| future::ready(!blocks_in_cache_set.contains(block_id)));

        Ok(Box::pin(
            stream::iter(blocks_in_cache.into_iter().map(Ok))
                .chain(blocks_in_base_store_and_not_in_cache),
        ))
    }

    pub async fn create(&self, data: &Data) -> Result<BlockId> {
        loop {
            let block_id = BlockId::new_random();
            let created = self.try_create(&block_id, data).await?;
            match created {
                TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => {
                    /* just continue */
                    ()
                }
                TryCreateResult::SuccessfullyCreated => {
                    return Ok(block_id);
                }
            }
        }
    }

    /// clear_cache_slow is only used in test cases. Without test cases calling it, they would only
    /// ever test cached blocks and never have to store/reload them to the base store.
    /// This is implemented in a very slow way and shouldn't be used in non-test code.
    #[cfg(test)]
    pub async fn clear_cache_slow(&self) -> Result<()> {
        self.cache.prune_all_blocks().await
    }
}

#[async_trait]
impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> AsyncDrop
    for LockingBlockStore<B>
{
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<()> {
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

impl<B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static> Debug
    for LockingBlockStore<B>
{
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_struct("LockingBlockStore").finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
#[must_use]
pub enum TryCreateResult {
    SuccessfullyCreated,
    NotCreatedBecauseBlockIdAlreadyExists,
}

#[derive(Debug, PartialEq, Eq)]
#[must_use]
pub enum RemoveResult {
    SuccessfullyRemoved,
    NotRemovedBecauseItDoesntExist,
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::blockstore::low_level::{mock::MockBlockStore};
    use crate::blockstore::tests::data;
    use anyhow::anyhow;
    use mockall::predicate::{always, function};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Mutex,
    };

    fn make_mock_block_store() -> AsyncDropGuard<MockBlockStore> {
        let mut store = AsyncDropGuard::new(MockBlockStore::new());
        store
            .expect_async_drop_impl()
            .times(1)
            .returning(|| Box::pin(async { Ok(()) }));
        store
    }

    #[tokio::test]
    async fn test_whenCallingCreate_thenPassesThroughDataToBaseStore() {
        let mut underlying_store = make_mock_block_store();
        underlying_store
            .expect_exists()
            .returning(|_| Box::pin(async { Ok(false) }));
        underlying_store
            .expect_store()
            .with(always(), function(|v| v == data(1024, 0).as_ref()))
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let mut store = LockingBlockStore::new(underlying_store);

        store.create(&data(1024, 0)).await.unwrap();

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_whenCallingCreate_thenReturnsCorrectBlockId() {
        let mut underlying_store = make_mock_block_store();
        let id_watcher: Arc<Mutex<Option<BlockId>>> = Arc::new(Mutex::new(None));
        let _id_watcher = Arc::clone(&id_watcher);
        underlying_store
            .expect_exists()
            .once()
            .returning(move |id| {
                let mut id_watcher = _id_watcher.lock().unwrap();
                assert_eq!(None, *id_watcher);
                *id_watcher = Some(*id);
                Box::pin(async { Ok(false) })
            });
        let _id_watcher = Arc::clone(&id_watcher);
        underlying_store
            .expect_store()
            .with(always(), function(|v| v == data(1024, 0).as_ref()))
            .once()
            .returning(move |id, _| {
                let id_watcher = _id_watcher.lock().unwrap();
                assert_eq!(*id, id_watcher.expect("id_watcher not set yet"));
                Box::pin(async { Ok(()) })
            });
        let mut store = LockingBlockStore::new(underlying_store);

        let block_id = store.create(&data(1024, 0)).await.unwrap();
        assert_eq!(*id_watcher.lock().unwrap(), Some(block_id));

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_whenCallingCreate_butIdAlreadyExists_thenTriesAgain() {
        let countdown_attempts = AtomicUsize::new(10); // The first 10 attempted ids say the id already exists
        let attempted_ids = Arc::new(Mutex::new(Vec::new()));

        let mut underlying_store = make_mock_block_store();
        let _attempted_ids = Arc::clone(&attempted_ids);
        underlying_store
            .expect_exists()
            .times(10..)
            .returning(move |id| {
                let mut attempted_ids = _attempted_ids.lock().unwrap();
                if attempted_ids.contains(id) {
                    // This id was already previously attempted, just return "it exists" again
                    // This branch should only be executed if the calling code somehow tries an id multiple times which
                    // either means a bug or a very very unlucky random generator, unlikely enough to actually never happen.
                    // We're still handling it here because test flakiness is bad style.
                    Box::pin(async { Ok(true) })
                } else {
                    attempted_ids.push(*id);
                    let say_it_exists = countdown_attempts.fetch_sub(1, Ordering::SeqCst) > 0;
                    Box::pin(async move { Ok(say_it_exists) })
                }
            });
        let _attempted_ids = Arc::clone(&attempted_ids);
        underlying_store
            .expect_store()
            .with(always(), function(|v| v == data(1024, 0).as_ref()))
            .once()
            .returning(move |id, _| {
                let attempted_ids = _attempted_ids.lock().unwrap();
                assert_eq!(
                    *id,
                    *attempted_ids.last().expect("attempted_ids not set yet")
                );
                Box::pin(async { Ok(()) })
            });
        let mut store = LockingBlockStore::new(underlying_store);

        let block_id = store.create(&data(1024, 0)).await.unwrap();
        assert_eq!(attempted_ids.lock().unwrap().last(), Some(&block_id));

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_whenCallingCreate_butExistsReturnsError_thenReturnsError() {
        let mut underlying_store = make_mock_block_store();
        underlying_store
            .expect_exists()
            .once()
            .returning(move |_| Box::pin(async { Err(anyhow!("Some error")) }));
        underlying_store.expect_store().never();
        let mut store = LockingBlockStore::new(underlying_store);

        let err = store.create(&data(1024, 0)).await.unwrap_err();
        assert_eq!("Some error", err.to_string());

        store.async_drop().await.unwrap();
    }
}
