use anyhow::{anyhow, bail, Context, Result};
use log::warn;
use std::path::PathBuf;

use super::{BlockId, BlockStore, BlockStoreReader, BlockStoreWriter, BLOCKID_LEN};

mod known_block_versions;

use known_block_versions::{ClientId, KnownBlockVersions};

const FORMAT_VERSION_HEADER: &[u8; 2] = &1u16.to_ne_bytes();

pub struct IntegrityConfig {
    allow_integrity_violations: bool,
    missing_block_is_integrity_violation: bool,
    on_integrity_violation: Box<dyn Fn()>,
}

pub struct IntegrityBlockStore<B> {
    underlying_block_store: B,
    config: IntegrityConfig,
    known_block_versions: KnownBlockVersions,
}

impl<B> IntegrityBlockStore<B> {
    pub fn new(
        underlying_block_store: B,
        integrity_file_path: PathBuf,
        my_client_id: ClientId,
        config: IntegrityConfig,
    ) -> Result<Self> {
        let known_block_versions = KnownBlockVersions::new(integrity_file_path, my_client_id)
            .context("Tried to create KnownBlockVersions")?;
        Ok(Self {
            underlying_block_store,
            config,
            known_block_versions,
        })
    }
}

impl<B: BlockStoreReader> BlockStoreReader for IntegrityBlockStore<B> {
    fn load(&self, block_id: &BlockId) -> Result<Option<Vec<u8>>> {
        let loaded = self.underlying_block_store.load(block_id).context(
            "IntegrityBlockStore tried to load the block from the underlying block store",
        )?;
        match loaded {
            None => {
                if self.config.missing_block_is_integrity_violation
                    && self.known_block_versions.should_block_exist(&block_id)
                {
                    let msg = format!(
                        "Block {} should exist but we didn't find it. Did an attacker delete it?",
                        block_id.to_hex()
                    );
                    self._integrity_violation_detected(&msg)?;
                }
                Ok(None)
            }
            Some(loaded) => todo!(),
        }
    }

    fn num_blocks(&self) -> Result<u64> {
        self.underlying_block_store.num_blocks()
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.underlying_block_store.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        block_size.checked_sub(FORMAT_VERSION_HEADER.len() as u64)
            .with_context(|| anyhow!("Physical block size of {} is too small to hold even the FORMAT_VERSION_HEADER. Must be at least {}.", block_size, FORMAT_VERSION_HEADER.len()))
    }

    fn all_blocks(&self) -> Result<Box<dyn Iterator<Item = BlockId>>> {
        todo!()
    }
}

impl<B: BlockStoreWriter> BlockStoreWriter for IntegrityBlockStore<B> {
    fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool> {
        todo!()
    }

    fn remove(&self, id: &BlockId) -> Result<bool> {
        todo!()
    }

    fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        todo!()
    }
}

impl<B> IntegrityBlockStore<B> {
    fn _integrity_violation_detected(&self, reason: &str) -> Result<()> {
        if self.config.allow_integrity_violations {
            warn!(
                "Integrity violation (but integrity checks are disabled): {}",
                reason
            );
            Ok(())
        } else {
            todo!()
            // self.known_block_versions
            //     .set_integrity_violation_in_previous_run();
            // (*self.config.on_integrity_violation)();
            // bail!("Integrity violation detected: {}", reason);
        }
    }

    // fn _check_and_remove_header(data: &[u8]) -> Result<&[u8]> {
    //     // TODO What about the data layout class we wrote somewhere?
    //     ensure!(data.len() >= FORMAT_VERSION_HEADER + )
    //     if !data.starts_with(FORMAT_VERSION_HEADER) {
    //         bail!("Wrong FORMAT_VERSION_HEADER of {:?}. Maybe it was created with a different major version of CryFS?", &data[..FORMAT_VERSION_HEADER.len()]);
    //     }
    //     let block_id_header =
    //         &data[FORMAT_VERSION_HEADER.len()..(FORMAT_VERSION_HEADER.len() + BLOCKID_LEN)];
    //     let

    //     Ok(&data[FORMAT_VERSION_HEADER.len()..])
    // }
}

impl<B: BlockStore> BlockStore for IntegrityBlockStore<B> {}
