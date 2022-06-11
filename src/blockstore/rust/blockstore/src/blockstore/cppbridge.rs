use anyhow::{bail, Result};
use futures::TryStreamExt;
use std::path::Path;

use super::low_level::{
    caching::CachingBlockStore,
    encrypted::EncryptedBlockStore,
    inmemory::InMemoryBlockStore,
    integrity::{ClientId, IntegrityBlockStore, IntegrityConfig},
    ondisk::OnDiskBlockStore,
    BlockStore, RemoveResult, TryCreateResult,
};
use crate::blockstore::{BlockId, BLOCKID_LEN};
use crate::crypto::symmetric::{Aes256Gcm, Cipher, EncryptionKey};
use crate::data::Data;

#[cxx::bridge]
mod ffi {
    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type BlockId;
        fn data(&self) -> &[u8; 16]; // TODO Instead of '16' we should use BLOCKID_LEN here
        fn new_blockid(id: &[u8; 16]) -> Box<BlockId>;
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type OptionData;
        fn has_value(&self) -> bool;
        fn value(&self) -> Result<&[u8]>;
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type RustBlockStore2Bridge;
        fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool>;
        fn remove(&self, id: &BlockId) -> Result<bool>;
        fn load(&self, id: &BlockId) -> Result<Box<OptionData>>;
        fn store(&self, id: &BlockId, data: &[u8]) -> Result<()>;
        fn num_blocks(&self) -> Result<u64>;
        fn estimate_num_free_bytes(&self) -> Result<u64>;
        fn block_size_from_physical_block_size(&self, block_size: u64) -> u64;
        fn all_blocks(&self) -> Result<Vec<BlockId>>;

        fn new_inmemory_blockstore() -> Box<RustBlockStore2Bridge>;
        fn new_encrypted_inmemory_blockstore() -> Box<RustBlockStore2Bridge>;
        fn new_caching_inmemory_blockstore() -> Box<RustBlockStore2Bridge>;
        fn new_integrity_inmemory_blockstore(
            integrity_file_path: &str,
        ) -> Result<Box<RustBlockStore2Bridge>>;
        fn new_ondisk_blockstore(basedir: &str) -> Box<RustBlockStore2Bridge>;
    }
}

fn new_blockid(data: &[u8; BLOCKID_LEN]) -> Box<BlockId> {
    Box::new(BlockId::from_array(data))
}

pub struct OptionData(Option<Data>);

impl OptionData {
    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn value(&self) -> Result<&[u8]> {
        match &self.0 {
            None => bail!("OptionData doesn't have a value"),
            Some(data) => Ok(data),
        }
    }
}

struct LoggerInit {}
impl LoggerInit {
    pub fn new() -> Self {
        env_logger::init();
        Self {}
    }

    pub fn ensure_initialized(&self) {
        // noop. But calling this means the lazy static has to be created.
    }
}

lazy_static::lazy_static! {
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    static ref LOGGER_INIT: LoggerInit = LoggerInit::new();
}

struct RustBlockStore2Bridge(Box<dyn BlockStore>);

impl RustBlockStore2Bridge {
    fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool> {
        // TODO Can we avoid a copy at the ffi boundary? i.e. use OptimizedBlockStoreWriter?
        match TOKIO_RUNTIME.block_on(self.0.try_create(id, data))? {
            TryCreateResult::SuccessfullyCreated => Ok(true),
            TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => Ok(false),
        }
    }
    fn remove(&self, id: &BlockId) -> Result<bool> {
        match TOKIO_RUNTIME.block_on(self.0.remove(id))? {
            RemoveResult::SuccessfullyRemoved => Ok(true),
            RemoveResult::NotRemovedBecauseItDoesntExist => Ok(false),
        }
    }
    fn load(&self, id: &BlockId) -> Result<Box<OptionData>> {
        let loaded = TOKIO_RUNTIME.block_on(self.0.load(id))?;
        Ok(Box::new(OptionData(loaded)))
    }
    fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        // TODO Can we avoid a copy at the ffi boundary? i.e. use OptimizedBlockStoreWriter?
        TOKIO_RUNTIME.block_on(self.0.store(id, data))
    }
    fn num_blocks(&self) -> Result<u64> {
        Ok(TOKIO_RUNTIME.block_on(self.0.num_blocks()).unwrap())
    }
    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.0.estimate_num_free_bytes()
    }
    fn block_size_from_physical_block_size(&self, block_size: u64) -> u64 {
        // In C++, the convention was to return 0 instead of an error,
        // so let's catch errors and return 0 instead.
        // TODO Is there a better way?
        self.0
            .block_size_from_physical_block_size(block_size)
            .unwrap_or(0)
    }
    fn all_blocks(&self) -> Result<Vec<BlockId>> {
        TOKIO_RUNTIME
            .block_on(async { TryStreamExt::try_collect(self.0.all_blocks().await?).await })
    }
}

fn new_inmemory_blockstore() -> Box<RustBlockStore2Bridge> {
    LOGGER_INIT.ensure_initialized();
    Box::new(RustBlockStore2Bridge(Box::new(InMemoryBlockStore::new())))
}

fn new_encrypted_inmemory_blockstore() -> Box<RustBlockStore2Bridge> {
    LOGGER_INIT.ensure_initialized();
    let key =
        EncryptionKey::from_hex("9726ca3703940a918802953d8db5996c5fb25008a20c92cb95aa4b8fe92702d9")
            .unwrap();
    Box::new(RustBlockStore2Bridge(Box::new(EncryptedBlockStore::new(
        InMemoryBlockStore::new(),
        Aes256Gcm::new(key),
    ))))
}

fn new_caching_inmemory_blockstore() -> Box<RustBlockStore2Bridge> {
    LOGGER_INIT.ensure_initialized();
    Box::new(RustBlockStore2Bridge(Box::new(CachingBlockStore::new(
        InMemoryBlockStore::new(),
    ))))
}

fn new_integrity_inmemory_blockstore(
    integrity_file_path: &str,
) -> Result<Box<RustBlockStore2Bridge>> {
    LOGGER_INIT.ensure_initialized();
    Ok(Box::new(RustBlockStore2Bridge(Box::new(
        IntegrityBlockStore::new(
            InMemoryBlockStore::new(),
            Path::new(integrity_file_path).to_path_buf(),
            ClientId { id: 1 },
            IntegrityConfig {
                allow_integrity_violations: false,
                missing_block_is_integrity_violation: true,
                on_integrity_violation: Box::new(|| {}),
            },
        )?,
    ))))
}

fn new_ondisk_blockstore(basedir: &str) -> Box<RustBlockStore2Bridge> {
    LOGGER_INIT.ensure_initialized();
    Box::new(RustBlockStore2Bridge(Box::new(OnDiskBlockStore::new(
        Path::new(basedir).to_path_buf(),
    ))))
}
