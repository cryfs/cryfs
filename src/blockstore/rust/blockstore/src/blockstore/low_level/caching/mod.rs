use anyhow::{ensure, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use futures::StreamExt;
use log::debug;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

// TODO Can I remove the underlying : OptimizedBlockStoreWriter requirement from the read methods? It's currently needed because the cache needs it for writing back dirty blocks,
// but in theory, dirty blocks shouldn't exist if we are read only.

// TODO Think through concurrency here. Is everything we want concurrent actually concurrent or do we lock too many mutexes? Are there race conditions left?

use super::{
    block_data::IBlockData, BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader,
    OptimizedBlockStoreWriter, RemoveResult, TryCreateResult,
};

use crate::data::Data;

mod cache;
use cache::{Cache, EvictionCallback, LRUCache};

const MAX_NUM_CACHE_ENTRIES: NonZeroUsize = unsafe {
    // TODO Use ::new().unwrap() instead of unsafe once that is const
    NonZeroUsize::new_unchecked(1000)
};

struct CachingBlockStoreImplImpl<B: OptimizedBlockStoreWriter> {
    underlying_block_store: B,
    cached_blocks_not_in_base_store: Mutex<HashSet<BlockId>>,
}

struct CachingBlockStoreImpl<B: OptimizedBlockStoreWriter + Send + Sync> {
    store_impl_impl: Arc<CachingBlockStoreImplImpl<B>>,
    cache: LRUCache<BlockId, CachedBlock<B::BlockData>, EvictionHandler<B>>,
}

impl<B: BlockStoreReader + OptimizedBlockStoreWriter + Send + Sync> CachingBlockStoreImpl<B> {
    async fn exists_in_cache_or_base_store(&mut self, block_id: &BlockId) -> Result<bool> {
        if self.cache.contains(block_id) {
            Ok(true)
        } else {
            self.store_impl_impl
                .underlying_block_store
                .exists(block_id)
                .await
        }
    }

    async fn load_from_cache_or_base_store(
        &mut self,
        block_id: &BlockId,
    ) -> Result<Option<CachedBlock<B::BlockData>>> {
        let loaded = if let Some(cached_block) = self.cache.pop(block_id) {
            debug!("Loaded {:?} from cache", block_id);
            Some(cached_block)
        } else {
            // TODO This is a bottleneck for concurrency since we're going to the basestore while we have a lock
            let loaded = self
                .store_impl_impl
                .underlying_block_store
                .load(block_id)
                .await?
                .map(|loaded| {
                    CachedBlock {
                        // TODO This assumes that what was returned from underlying_block_store.load() has enough prefix bytes so that we can stuff it back into OptimizedBlockStoreWriter. Not sure if all block stores fulfill this, we may violate the BlockData invariant here.
                        data: B::BlockData::new(loaded),
                        dirty: false,
                    }
                });
            if loaded.is_some() {
                debug!("Loaded {:?} from base store", block_id);
            } else {
                debug!("Didn't find {:?} in base store", block_id);
            }
            loaded
        };
        Ok(loaded)
    }
}

pub struct CachingBlockStore<B: OptimizedBlockStoreWriter + Send + Sync> {
    store_impl: AsyncMutex<CachingBlockStoreImpl<B>>,
}

struct EvictionHandler<B: OptimizedBlockStoreWriter> {
    store_impl_impl: Arc<CachingBlockStoreImplImpl<B>>,
}

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Send + Sync>
    EvictionCallback<BlockId, CachedBlock<B::BlockData>> for EvictionHandler<B>
{
    async fn on_evict(
        &self,
        block_id: BlockId,
        cached_block: CachedBlock<B::BlockData>,
    ) -> Result<()> {
        if cached_block.dirty {
            self.store_impl_impl
                .underlying_block_store
                .store_optimized(&block_id, cached_block.data)
                .await?;

            // remove it from the list of blocks not in the base store, if it's on it
            self.store_impl_impl
                .cached_blocks_not_in_base_store
                .lock()
                .unwrap()
                .remove(&block_id);
        }
        Ok(())
    }
}

struct CachedBlock<BlockData> {
    data: BlockData,
    dirty: bool,
}

impl<B: OptimizedBlockStoreWriter + Send + Sync> CachingBlockStore<B> {
    pub fn new(underlying_block_store: B) -> Self {
        let store_impl_impl = Arc::new(CachingBlockStoreImplImpl {
            underlying_block_store,
            cached_blocks_not_in_base_store: Mutex::new(HashSet::new()),
        });
        let store_impl_impl_rcclone = Arc::clone(&store_impl_impl);
        let cache = LRUCache::new(
            MAX_NUM_CACHE_ENTRIES,
            EvictionHandler {
                store_impl_impl: store_impl_impl_rcclone,
            },
        );
        let store_impl = AsyncMutex::new(CachingBlockStoreImpl {
            store_impl_impl,
            cache,
        });
        Self { store_impl }
    }
}

#[async_trait]
impl<B: BlockStoreReader + OptimizedBlockStoreWriter + Send + Sync> BlockStoreReader
    for CachingBlockStore<B>
{
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        let mut store_impl = self.store_impl.lock().await;
        store_impl.exists_in_cache_or_base_store(id).await
    }

    async fn load(&self, block_id: &BlockId) -> Result<Option<Data>> {
        debug!("Loading {:?}", block_id);
        // TODO Move this (and other) code to CachingBlockStoreImpl?
        let mut store_impl = self.store_impl.lock().await;
        if let Some(loaded) = store_impl.load_from_cache_or_base_store(block_id).await? {
            let data = loaded.data.clone();
            store_impl.cache.push(*block_id, loaded)
                .expect("Adding an element to the cache failed even though we either just extracted it from the cache or checked that it doesn't exist in the cache.");
            Ok(Some(data.extract()))
        } else {
            // TODO Cache non-existence?
            Ok(None)
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        let store_impl = self.store_impl.lock().await;
        let underlying_num_blocks = store_impl
            .store_impl_impl
            .underlying_block_store
            .num_blocks()
            .await?;
        let num_blocks_not_in_base_store = store_impl
            .store_impl_impl
            .cached_blocks_not_in_base_store
            .lock()
            .unwrap()
            .len() as u64;
        Ok(underlying_num_blocks + num_blocks_not_in_base_store)
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        // TODO Should we make estimate_num_free_bytes async instead of using block_on here? Or is there a way to avoid the lock or use a sync mutex?
        let store_impl = tokio::runtime::Handle::current().block_on(self.store_impl.lock());
        store_impl
            .store_impl_impl
            .underlying_block_store
            .estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        // TODO Should we make estimate_num_free_bytes async instead of using block_on here? Or is there a way to avoid the lock or use a sync mutex?
        let store_impl = tokio::runtime::Handle::current().block_on(self.store_impl.lock());
        store_impl
            .store_impl_impl
            .underlying_block_store
            .block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        let store_impl = self.store_impl.lock().await;
        let cached_blocks_not_in_base_store: Vec<Result<BlockId>> = store_impl
            .store_impl_impl
            .cached_blocks_not_in_base_store
            .lock()
            .unwrap()
            .iter()
            .cloned()
            .map(Ok)
            .collect();
        let cached_blocks_in_base_store = store_impl
            .store_impl_impl
            .underlying_block_store
            .all_blocks()
            .await?;
        Ok(Box::pin(
            futures::stream::iter(cached_blocks_not_in_base_store)
                .chain(cached_blocks_in_base_store),
        ))
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + OptimizedBlockStoreWriter + Send + Sync> BlockStoreDeleter
    for CachingBlockStore<B>
{
    async fn remove(&self, block_id: &BlockId) -> Result<RemoveResult> {
        // TODO Don't write-through but cache remove operations?
        let mut store_impl = self.store_impl.lock().await;
        match store_impl.cache.pop(block_id) {
            Some(_cached_block) => {
                let block_should_not_exist_in_base_store = store_impl
                    .store_impl_impl
                    .cached_blocks_not_in_base_store
                    .lock()
                    .unwrap()
                    .remove(&block_id);
                let block_should_exist_in_base_store = !block_should_not_exist_in_base_store;
                if block_should_exist_in_base_store {
                    let remove_result = store_impl
                        .store_impl_impl
                        .underlying_block_store
                        .remove(block_id)
                        .await?;
                    ensure!(remove_result == RemoveResult::SuccessfullyRemoved, "Tried to remove block {:?}. Block existed in cache and stated it exists in base store, but wasn't found there.", block_id);
                }
                Ok(RemoveResult::SuccessfullyRemoved)
            }
            None => {
                store_impl
                    .store_impl_impl
                    .underlying_block_store
                    .remove(block_id)
                    .await
            }
        }
    }
}

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Send + Sync> OptimizedBlockStoreWriter
    for CachingBlockStore<B>
{
    type BlockData = B::BlockData;

    fn allocate(size: usize) -> Self::BlockData {
        B::allocate(size)
    }

    async fn try_create_optimized(
        &self,
        block_id: &BlockId,
        data: Self::BlockData,
    ) -> Result<TryCreateResult> {
        let mut store_impl = self.store_impl.lock().await;
        // TODO Check if block exists in base store? Performance hit? It's very unlikely it exists.
        if let Some(cached_block) = store_impl.cache.pop(block_id) {
            // push the just popped element back to the cache
            store_impl.cache.push(*block_id, cached_block).expect("Failed to re-add an element that we just popped from the cache. This should always succeed.");
            Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists)
        } else {
            store_impl.cache.push(
                *block_id,
                CachedBlock {
                    data: data.clone(),
                    dirty: true,
                },
            )?;
            store_impl
                .store_impl_impl
                .cached_blocks_not_in_base_store
                .lock()
                .unwrap()
                .insert(*block_id);
            Ok(TryCreateResult::SuccessfullyCreated)
        }
    }

    async fn store_optimized(&self, block_id: &BlockId, data: Self::BlockData) -> Result<()> {
        debug!("Store {:?}", block_id);
        let mut store_impl = self.store_impl.lock().await;
        let new_cached_block = if let Some(_old_cached_block) = store_impl.cache.pop(block_id) {
            CachedBlock { data, dirty: true }
        } else {
            // Make sure that the block exists in the underlying store
            // TODO Instead of storing it to the base store, we could just keep it dirty in the cache
            //      and (if it doesn't exist in base store yet) add it to _cachedBlocksNotInBaseStore
            // TODO This is a bottleneck for concurrency since we're going to the basestore while we have a lock
            let data_clone = data.clone();
            store_impl
                .store_impl_impl
                .underlying_block_store
                .store_optimized(block_id, data_clone)
                .await?;

            CachedBlock { data, dirty: false }
        };
        store_impl.cache.push(*block_id, new_cached_block)?;
        Ok(())
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter + Send + Sync> BlockStore for CachingBlockStore<B> {}
