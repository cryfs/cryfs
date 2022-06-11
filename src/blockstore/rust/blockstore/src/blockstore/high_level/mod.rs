use anyhow::Result;
use futures::stream::Stream;
use log;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use crate::blockstore::BlockId;
use crate::data::Data;

// TODO Add locking so that we don't open the same block multiple times or remove a block while it is open

pub struct Block<B: super::low_level::BlockStore> {
    block_id: BlockId,
    data: Data,
    dirty: bool,
    base_store: Rc<B>,
}

impl<B: super::low_level::BlockStore> Block<B> {
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

impl<B: super::low_level::BlockStore> Drop for Block<B> {
    fn drop(&mut self) {
        assert!(!self.dirty, "Dropped dirty block {:?}. This means we didn't write the changes back. This is a bug. Not good!", self.block_id)
    }
}

// TODO Should we require B: OptimizedBlockStoreWriter and use its methods?
pub struct LockingBlockStore<B: super::low_level::BlockStore> {
    base_store: Rc<B>,
}

impl<B: super::low_level::BlockStore> LockingBlockStore<B> {
    async fn with_load<T, F>(
        &self,
        block_id: BlockId,
        func: impl FnOnce(Option<&mut Block<B>>) -> F,
    ) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let loaded = self.base_store.load(&block_id).await?;
        if let Some(loaded) = loaded {
            let mut block = Block {
                block_id,
                data: loaded,
                dirty: false,
                base_store: Rc::clone(&self.base_store),
            };
            let result = func(Some(&mut block)).await;
            match block.flush().await {
                Ok(()) => result,
                Err(flush_err) => {
                    // Report the flush error but log any potentially swallowed error from func
                    if let Err(func_err) = result {
                        log::error!(
                            "Swallowed error because another error happened: {}",
                            func_err
                        );
                    }
                    Err(flush_err)
                }
            }
        } else {
            func(None).await
        }
    }

    async fn try_create(&self, block_id: &BlockId, data: &Data) -> Result<TryCreateResult> {
        // TODO Is self.base_store.try_create_optimized() better?
        match self.base_store.try_create(block_id, data.as_ref()).await? {
            crate::blockstore::low_level::TryCreateResult::SuccessfullyCreated => Ok(TryCreateResult::SuccessfullyCreated),
            crate::blockstore::low_level::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => Ok(TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists),
        }
    }

    async fn overwrite(&self, block_id: &BlockId, data: &Data) -> Result<()> {
        // TODO Does an API make more sense where we call a callback similar to with_load, allow call sites to modify that block, and only write back once the callback returns?
        // TODO Is self.base_store.store_optimized() better?
        self.base_store.store(block_id, data).await
    }

    async fn remove(&self, block_id: &BlockId) -> Result<RemoveResult> {
        match self.base_store.remove(block_id).await? {
            crate::blockstore::low_level::RemoveResult::SuccessfullyRemoved => Ok(RemoveResult::SuccessfullyRemoved),
            crate::blockstore::low_level::RemoveResult::NotRemovedBecauseItDoesntExist => Ok(RemoveResult::NotRemovedBecauseItDoesntExist),
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.base_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.base_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.base_store.block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        self.base_store.all_blocks().await
    }

    async fn create(&self, data: &Data) -> Result<()> {
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
