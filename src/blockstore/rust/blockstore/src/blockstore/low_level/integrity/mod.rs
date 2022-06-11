use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use async_trait::async_trait;
use binary_layout::prelude::*;
use futures::stream::{Stream, StreamExt, TryStreamExt};
use log::warn;
use std::collections::hash_set::HashSet;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Mutex;

use super::block_data::IBlockData;
use super::{
    BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter,
    RemoveResult, TryCreateResult, BLOCKID_LEN,
};

mod known_block_versions;

use crate::data::Data;
pub use known_block_versions::ClientId;
use known_block_versions::{BlockVersion, IntegrityViolationError, KnownBlockVersions};

const FORMAT_VERSION_HEADER: u16 = 1;

binary_layout::define_layout!(block_layout, LittleEndian, {
    // TODO Use types BlockId, FormatVersionHeader as types instead of slices
    format_version_header: u16,
    block_id: [u8; BLOCKID_LEN],
    last_update_client_id: ClientId as u32,
    block_version: BlockVersion as u64,
    data: [u8],
});

const HEADER_SIZE: usize = block_layout::data::OFFSET;

pub struct IntegrityConfig {
    pub allow_integrity_violations: bool,
    pub missing_block_is_integrity_violation: bool,
    pub on_integrity_violation: Box<dyn Sync + Send + Fn()>,
}

pub struct IntegrityBlockStore<B> {
    underlying_block_store: B,
    config: IntegrityConfig,
    known_block_versions: Mutex<KnownBlockVersions>,
}

impl<B> IntegrityBlockStore<B> {
    pub fn new(
        underlying_block_store: B,
        integrity_file_path: PathBuf,
        my_client_id: ClientId,
        config: IntegrityConfig,
    ) -> Result<Self> {
        let known_block_versions =
            KnownBlockVersions::new(integrity_file_path.clone(), my_client_id)
                .context("Tried to create KnownBlockVersions")?;
        if known_block_versions.integrity_violation_in_previous_run() {
            if config.allow_integrity_violations {
                warn!("Integrity violation in previous run (but integrity checks are disabled)");
            } else {
                return Err(IntegrityViolationError::IntegrityViolationInPreviousRun {
                    integrity_file_path: integrity_file_path.clone(),
                }
                .into());
            }
        }
        Ok(Self {
            underlying_block_store,
            config,
            known_block_versions: Mutex::new(known_block_versions),
        })
    }
}

#[async_trait]
impl<B: BlockStoreReader + Sync + Send> BlockStoreReader for IntegrityBlockStore<B> {
    async fn load(&self, block_id: &BlockId) -> Result<Option<Data>> {
        let loaded = self.underlying_block_store.load(block_id).await.context(
            "IntegrityBlockStore tried to load the block from the underlying block store",
        )?;
        match loaded {
            None => {
                if self.config.missing_block_is_integrity_violation
                    && self
                        .known_block_versions
                        .lock()
                        .unwrap()
                        .should_block_exist(&block_id)
                {
                    self._integrity_violation_detected(
                        IntegrityViolationError::MissingBlock { block: *block_id }.into(),
                    )?;
                }
                Ok(None)
            }
            Some(loaded) => {
                let data = self._check_and_remove_header(loaded, block_id)?;
                Ok(Some(data))
            }
        }
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_block_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        block_size.checked_sub(HEADER_SIZE as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, HEADER_SIZE))
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        let all_underlying_blocks = self.underlying_block_store.all_blocks().await?;
        if self.config.missing_block_is_integrity_violation {
            // TODO No collect, but pass through stream asynchronously?
            let all_underlying_blocks: Vec<BlockId> = all_underlying_blocks.try_collect().await?;
            let mut expected_blocks: HashSet<BlockId> = self
                .known_block_versions
                .lock()
                .unwrap()
                .existing_blocks()
                .copied()
                .collect();
            for existing_block in &all_underlying_blocks {
                expected_blocks.remove(existing_block);
            }
            if !expected_blocks.is_empty() {
                self._integrity_violation_detected(
                    IntegrityViolationError::MissingBlocks {
                        blocks: expected_blocks,
                    }
                    .into(),
                )?;
            }
            Ok(futures::stream::iter(all_underlying_blocks.into_iter().map(Ok)).boxed())
        } else {
            Ok(all_underlying_blocks)
        }
    }
}

#[async_trait]
impl<B: BlockStoreDeleter + Sync + Send> BlockStoreDeleter for IntegrityBlockStore<B> {
    async fn remove(&self, id: &BlockId) -> Result<RemoveResult> {
        self.known_block_versions
            .lock()
            .unwrap()
            .mark_block_as_deleted(*id);
        self.underlying_block_store.remove(id).await
    }
}

create_block_data_wrapper!(BlockData);

#[async_trait]
impl<B: OptimizedBlockStoreWriter + Sync + Send> OptimizedBlockStoreWriter
    for IntegrityBlockStore<B>
{
    type BlockData = BlockData;

    fn allocate(size: usize) -> BlockData {
        let mut data = B::allocate(HEADER_SIZE + size).extract();
        data.shrink_to_subregion(HEADER_SIZE..);
        BlockData::new(data)
    }

    async fn try_create_optimized(&self, id: &BlockId, data: BlockData) -> Result<TryCreateResult> {
        let data = self._prepend_header(id, data.extract());
        self.underlying_block_store
            .try_create_optimized(id, B::BlockData::new(data))
            .await
    }

    async fn store_optimized(&self, id: &BlockId, data: BlockData) -> Result<()> {
        let data = self._prepend_header(id, data.extract());
        self.underlying_block_store
            .store_optimized(id, B::BlockData::new(data))
            .await
    }
}

impl<B> IntegrityBlockStore<B> {
    fn _integrity_violation_detected(&self, reason: Error) -> Result<()> {
        assert!(
            reason.is::<IntegrityViolationError>(),
            "This should only be called with an IntegrityViolationError"
        );
        if self.config.allow_integrity_violations {
            warn!(
                "Integrity violation (but integrity checks are disabled): {:?}",
                reason,
            );
            Ok(())
        } else {
            self.known_block_versions
                .lock()
                .unwrap()
                .set_integrity_violation_in_previous_run();
            (*self.config.on_integrity_violation)();
            Err(reason)
        }
    }

    fn _prepend_header(&self, id: &BlockId, mut data: Data) -> Data {
        let (version, my_client_id) = {
            let ref mut known_block_versions = self.known_block_versions.lock().unwrap();
            let version = known_block_versions.increment_version(*id);
            let my_client_id = known_block_versions.my_client_id();
            (version, my_client_id)
        };

        data.grow_region_fail_if_reallocation_necessary(HEADER_SIZE, 0).expect("Tried to grow the data to contain the header in IntegrityBlockStore::_prepend_header");
        let mut view = block_layout::View::new(data);
        view.format_version_header_mut()
            .write(FORMAT_VERSION_HEADER);
        view.block_id_mut().copy_from_slice(id.data());
        view.last_update_client_id_mut().write(my_client_id);
        view.block_version_mut().write(version);
        view.into_storage()
    }

    fn _check_and_remove_header(&self, data: Data, expected_block_id: &BlockId) -> Result<Data> {
        ensure!(
            data.len() >= block_layout::data::OFFSET,
            "Block size is {} but we need at least {} to store the block header",
            data.len(),
            block_layout::data::OFFSET
        );
        let view = block_layout::View::new(data);
        let format_version_header = view.format_version_header().read();
        if format_version_header != FORMAT_VERSION_HEADER {
            bail!("Wrong FORMAT_VERSION_HEADER of {:?}. Expected {:?}. Maybe it was created with a different major version of CryFS?", format_version_header, FORMAT_VERSION_HEADER);
        }
        let block_id = BlockId::from_array(view.block_id());
        if block_id != *expected_block_id {
            self._integrity_violation_detected(
                IntegrityViolationError::WrongBlockId {
                    id_from_filename: *expected_block_id,
                    id_from_header: block_id,
                }
                .into(),
            )?;
        }
        let last_update_client_id = view.last_update_client_id().read();
        let block_version = view.block_version().read();
        match self
            .known_block_versions
            .lock()
            .unwrap()
            .check_and_update_version(last_update_client_id, block_id, block_version)
        {
            Ok(()) => (),
            Err(err) if err.is::<IntegrityViolationError>() => {
                // IntegrityViolationErrors are channeled through _integrity_violation_detected
                // so that we can silence them if integrity checking is disabled.
                self._integrity_violation_detected(err)?;
            }
            Err(err) => Err(err)?,
        }

        // TODO Use view.into_data().extract(), but that requires adding an IntoSubregion trait to binary-layout that we can implement for our Data class.
        let mut data = view.into_storage();
        data.shrink_to_subregion(block_layout::data::OFFSET..);
        Ok(data)
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter + Sync + Send> BlockStore
    for IntegrityBlockStore<B>
{
}
