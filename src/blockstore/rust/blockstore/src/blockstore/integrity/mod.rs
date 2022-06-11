use anyhow::{anyhow, bail, ensure, Context, Error, Result};
use binary_layout::FieldMetadata;
use log::warn;
use std::path::PathBuf;
use std::sync::Mutex;

use super::{
    BlockId, BlockStore, BlockStoreDeleter, BlockStoreReader, OptimizedBlockStoreWriter,
    BLOCKID_LEN,
};

mod known_block_versions;

use crate::data::Data;
use known_block_versions::{BlockVersion, ClientId, IntegrityViolationError, KnownBlockVersions};

const FORMAT_VERSION_HEADER: u16 = 1;

binary_layout::define_layout!(block_layout, LittleEndian, {
    // TODO Use types BlockId, ClientId, ... instead of slices, probably through some LayoutAs trait
    format_version_header: u16,
    block_id: [u8; BLOCKID_LEN],
    last_update_client_id: u32,
    block_version: u64,
    data: [u8],
});

const HEADER_SIZE: usize = block_layout::data::OFFSET;

pub struct IntegrityConfig {
    allow_integrity_violations: bool,
    missing_block_is_integrity_violation: bool,
    on_integrity_violation: Box<dyn Fn()>,
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
        let known_block_versions = Mutex::new(
            KnownBlockVersions::new(integrity_file_path, my_client_id)
                .context("Tried to create KnownBlockVersions")?,
        );
        Ok(Self {
            underlying_block_store,
            config,
            known_block_versions,
        })
    }
}

impl<B: BlockStoreReader> BlockStoreReader for IntegrityBlockStore<B> {
    fn load(&self, block_id: &BlockId) -> Result<Option<Data>> {
        let loaded = self.underlying_block_store.load(block_id).context(
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

    fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store.num_blocks()
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_block_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        block_size.checked_sub(HEADER_SIZE as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, HEADER_SIZE))
    }

    fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>> {
        todo!()
    }
}

impl<B: BlockStoreDeleter> BlockStoreDeleter for IntegrityBlockStore<B> {
    fn remove(&self, id: &BlockId) -> Result<bool> {
        todo!()
    }
}

create_block_data_wrapper!(BlockData);

impl<B: OptimizedBlockStoreWriter> OptimizedBlockStoreWriter for IntegrityBlockStore<B> {
    type BlockData = BlockData;

    fn allocate(size: usize) -> BlockData {
        todo!()
    }

    fn try_create_optimized(&self, id: &BlockId, data: BlockData) -> Result<bool> {
        todo!()
    }

    fn store_optimized(&self, id: &BlockId, data: BlockData) -> Result<()> {
        todo!()
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
        let block_id = BlockId::from_array(view.block_id().data());
        if block_id != *expected_block_id {
            self._integrity_violation_detected(
                IntegrityViolationError::WrongBlockId {
                    id_from_filename: *expected_block_id,
                    id_from_header: block_id,
                }
                .into(),
            )?;
        }
        let last_update_client_id = ClientId {
            id: view.last_update_client_id().read(),
        };
        let block_version = BlockVersion {
            version: view.block_version().read(),
        };
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
        Ok(view
            .into_storage()
            .into_subregion(block_layout::data::OFFSET..))
    }
}

impl<B: BlockStore + OptimizedBlockStoreWriter> BlockStore for IntegrityBlockStore<B> {}
