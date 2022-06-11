use anyhow::{bail, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use log;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use lockpool::{AsyncLockPool, LockPool, TokioLockPool, TryLockError};

use crate::blockstore::BlockId;
use crate::data::Data;
use crate::utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub struct Block<B: super::low_level::BlockStore + Send + Sync> {
    block_id: BlockId,
    data: Data,
    dirty: bool,
    base_store: Arc<B>,
    lock_guard: <TokioLockPool<BlockId> as LockPool<BlockId>>::OwnedGuard,
}

impl<B: super::low_level::BlockStore + Send + Sync> fmt::Debug for Block<B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Block")
            .field("block_id", &self.block_id)
            .field("dirty", &self.dirty)
            .finish()
    }
}

impl<B: super::low_level::BlockStore + Send + Sync> Block<B> {
    #[inline]
    pub fn block_id(&self) -> &BlockId {
        &self.block_id
    }

    #[inline]
    pub fn data(&self) -> &Data {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut Data {
        self.dirty = true;
        &mut self.data
    }

    pub async fn flush(&mut self) -> Result<()> {
        if self.dirty {
            // TODO self.base_store.optimized_store() ?
            self.base_store.store(&self.block_id, &self.data).await?;
            self.dirty = false;
        }
        Ok(())
    }

    pub async fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size);
        self.dirty = true;
    }
}

#[async_trait]
impl<B: super::low_level::BlockStore + Send + Sync> AsyncDrop for Block<B> {
    type Error = anyhow::Error;

    async fn async_drop_impl(mut self) -> Result<()> {
        self.flush().await
    }
}

// TODO Should we require B: OptimizedBlockStoreWriter and use its methods?
pub struct LockingBlockStore<B: super::low_level::BlockStore + Send + Sync> {
    base_store: Arc<B>,
    lock_pool: Arc<TokioLockPool<BlockId>>,
}

impl<B: super::low_level::BlockStore + Send + Sync> LockingBlockStore<B> {
    pub fn new(base_store: B) -> Self {
        Self {
            base_store: Arc::new(base_store),
            lock_pool: Arc::new(LockPool::new()),
        }
    }

    pub async fn load(&self, block_id: BlockId) -> Result<Option<AsyncDropGuard<Block<B>>>> {
        let lock_guard = self.lock_pool.lock_owned_async(block_id).await;
        Ok(self.base_store.load(&block_id).await?.map(|data| {
            AsyncDropGuard::new(Block {
                block_id,
                data,
                dirty: false,
                base_store: Arc::clone(&self.base_store),
                lock_guard,
            })
        }))
    }

    pub async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult> {
        let lock_guard = self.lock_pool.try_lock(*block_id);
        match lock_guard {
            Err(TryLockError::WouldBlock) => {
                // If the lock is currently held, then the block already exists
                Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists)
            }
            Err(TryLockError::Poisoned(err)) => {
                bail!("Log poisoned: {:?}", err)
            }
            Ok(_guard) => {
                // TODO Is self.base_store.try_create_optimized() better?
                match self.base_store.try_create(block_id, data.as_ref()).await? {
                    crate::blockstore::low_level::TryCreateResult::SuccessfullyCreated => Ok(TryCreateResult::SuccessfullyCreated),
                    crate::blockstore::low_level::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists),
                }
            }
        }
    }

    pub async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()> {
        let _lock_guard = self.lock_pool.lock_async(*block_id).await;
        // TODO Does an API make more sense where we call a callback similar to with_load, allow call sites to modify that block, and only write back once the callback returns?
        // TODO Is self.base_store.store_optimized() better?
        self.base_store.store(block_id, data).await
    }

    pub async fn remove(&self, block_id: &BlockId) -> Result<RemoveResult> {
        let _lock_guard = self.lock_pool.lock_async(*block_id).await;

        match self.base_store.remove(block_id).await? {
            crate::blockstore::low_level::RemoveResult::SuccessfullyRemoved => {
                Ok(RemoveResult::SuccessfullyRemoved)
            }
            crate::blockstore::low_level::RemoveResult::NotRemovedBecauseItDoesntExist => {
                Ok(RemoveResult::NotRemovedBecauseItDoesntExist)
            }
        }
    }

    pub async fn num_blocks(&self) -> Result<u64> {
        self.base_store.num_blocks().await
    }

    pub fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.base_store.estimate_num_free_bytes()
    }

    pub fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.base_store
            .block_size_from_physical_block_size(block_size)
    }

    pub async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        self.base_store.all_blocks().await
    }

    pub async fn create(&self, data: &Data) -> Result<()> {
        loop {
            let created = self.try_create(&BlockId::new_random(), data).await?;
            match created {
                TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => {
                    /* just continue */
                    ()
                }
                TryCreateResult::SuccessfullyCreated => {
                    return Ok(());
                }
            }
        }
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
