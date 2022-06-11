use anyhow::{bail, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use futures::TryStreamExt;
use std::path::Path;
use std::pin::Pin;

use super::high_level::{Block, LockingBlockStore, self};
use super::low_level::{
    caching::CachingBlockStore,
    encrypted::EncryptedBlockStore,
    inmemory::InMemoryBlockStore,
    integrity::{ClientId, IntegrityBlockStore, IntegrityConfig},
    ondisk::OnDiskBlockStore,
    BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter,
    self,
};
use crate::blockstore::{BlockId, BLOCKID_LEN};
use crate::crypto::symmetric::{Aes256Gcm, Cipher, EncryptionKey};
use crate::data::Data;
use crate::utils::async_drop::AsyncDropGuard;

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
        type OptionRustBlockBridge;
        fn has_value(&self) -> bool;
        fn extract_value(&mut self) -> Result<Box<RustBlockBridge>>;
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

        fn new_locking_inmemory_blockstore() -> Box<RustBlockStoreBridge>;
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type RustBlockStoreBridge;
        fn create_block_id(&self) -> Box<BlockId>;
        fn try_create(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<OptionRustBlockBridge>>;
        fn load(&self, block_id: &BlockId) -> Result<Box<OptionRustBlockBridge>>;
        fn overwrite(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<RustBlockBridge>>;
        fn remove(&self, block_id: &BlockId) -> Result<bool>;
        fn num_blocks(&self) -> Result<u64>;
        fn estimate_num_free_bytes(&self) -> Result<u64>;
        fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64>;
        fn all_blocks(&self) -> Result<Vec<BlockId>>;
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type RustBlockBridge;
        fn block_id(&self) -> Box<BlockId>;
        fn async_drop(&mut self) -> Result<()>;
        fn size(&self) -> usize;
        fn flush(&mut self) -> Result<()>;
        fn resize(&mut self, new_size: usize);
        fn data(&self) -> &[u8];
        fn write(&mut self, source: &[u8], offset: usize) -> Result<()>;
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

// Invariant: Option is always Some() unless the value was dropped
struct RustBlockBridge(Option<AsyncDropGuard<Block<DynBlockStore>>>);

impl RustBlockBridge {
    fn new(block: AsyncDropGuard<Block<DynBlockStore>>) -> Self {
        Self(Some(block))
    }

    fn block_id(&self) -> Box<BlockId> {
        Box::new(*self.0.as_ref().expect("Block was already dropped").block_id())
    }

    fn async_drop(&mut self) -> Result<()> {
        TOKIO_RUNTIME.block_on(
            self.0
                .take()
                .expect("Block was already dropped")
                .async_drop(),
        )?;
        Ok(())
    }

    fn size(&self) -> usize {
        self.0
            .as_ref()
            .expect("Block was already dropped")
            .data()
            .len()
    }

    fn flush(&mut self) -> Result<()> {
        TOKIO_RUNTIME.block_on(
            self.0.as_mut().expect("Block was already dropped").flush()
        )
    }

    fn resize(&mut self, new_size: usize) {
        TOKIO_RUNTIME.block_on(
            self.0.as_mut().expect("Block was already dropped").resize(new_size)
        )
    }

    fn write(&mut self, source: &[u8], offset: usize) -> Result<()> {
        let s = self.0.as_mut().expect("Block was already dropped");
        let dest = &mut s.data_mut()[offset..(offset+source.len())];
        if dest.len() != source.len() {
            bail!("Tried to write out of block boundaries. Write offset {}, size {} but block size is {}", offset, source.len(), s.data().len());
        }
        dest.copy_from_slice(source);
        Ok(())
    }

    fn data(&self) -> &[u8] {
        self.0.as_ref().expect("Block was already dropped").data()
    }
}

pub struct OptionRustBlockBridge(Option<RustBlockBridge>);

impl OptionRustBlockBridge {
    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn extract_value(&mut self) -> Result<Box<RustBlockBridge>> {
        match self.0.take() {
            None => bail!("OptionRustBlockBridge doesn't have a value"),
            Some(data) => Ok(Box::new(data)),
        }
    }
}

struct RustBlockStoreBridge(LockingBlockStore<DynBlockStore>);

impl RustBlockStoreBridge {
    fn create_block_id(&self) -> Box<BlockId> {
        Box::new(BlockId::new_random())
    }

    async fn _try_create(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<OptionRustBlockBridge>> {
        match self.0.try_create(block_id, &data.to_vec().into()).await? {
            high_level::TryCreateResult::SuccessfullyCreated => {
                let loaded = self.0.load(*block_id).await?.expect("We just created this but it doesn't exist?");
                Ok(Box::new(OptionRustBlockBridge(Some(RustBlockBridge::new(loaded)))))
            }
            high_level::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => Ok(Box::new(OptionRustBlockBridge(None))),
        }
    }

    fn try_create(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<OptionRustBlockBridge>> {
        TOKIO_RUNTIME.block_on(self._try_create(block_id, data))
    }

    fn load(&self, block_id: &BlockId) -> Result<Box<OptionRustBlockBridge>> {
        match TOKIO_RUNTIME.block_on(self.0.load(*block_id))? {
            Some(block) => Ok(Box::new(OptionRustBlockBridge(Some(RustBlockBridge::new(block))))),
            None => Ok(Box::new(OptionRustBlockBridge(None))),
        }
    }

    async fn _overwrite(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<RustBlockBridge>> {
        // TODO Overwriting and then loading could be slow. Should we instead change the rust API so that it also returns the block from the overwrite() call?
        self.0.overwrite(block_id, &data.to_vec().into()).await?;
        let loaded = self.0.load(*block_id).await?.expect("We just created this but it doesn't exist?");
        Ok(Box::new(RustBlockBridge::new(loaded)))
    }

    fn overwrite(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<RustBlockBridge>> {
        TOKIO_RUNTIME.block_on(self._overwrite(block_id, data))
    }

    fn remove(&self, block_id: &BlockId) -> Result<bool> {
        match TOKIO_RUNTIME.block_on(self.0.remove(block_id))? {
            high_level::RemoveResult::SuccessfullyRemoved => Ok(true),
            high_level::RemoveResult::NotRemovedBecauseItDoesntExist => Ok(false),
        }
    }

    fn num_blocks(&self) -> Result<u64> {
        TOKIO_RUNTIME.block_on(self.0.num_blocks())
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.0.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.0.block_size_from_physical_block_size(block_size)
    }

    fn all_blocks(&self) -> Result<Vec<BlockId>> {
        TOKIO_RUNTIME
            .block_on(async { TryStreamExt::try_collect(self.0.all_blocks().await?).await })
    }
}

struct DynBlockStore(Box<dyn BlockStore + Send + Sync>);

#[async_trait]
impl BlockStoreReader for DynBlockStore {
    async fn load(&self, id: &BlockId) -> Result<Option<Data>> {
        self.0.load(id).await
    }

    async fn num_blocks(&self) -> Result<u64> {
        self.0.num_blocks().await
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        self.0.estimate_num_free_bytes()
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        self.0.block_size_from_physical_block_size(block_size)
    }

    async fn all_blocks(&self) -> Result<Pin<Box<dyn Stream<Item = Result<BlockId>> + Send>>> {
        self.0.all_blocks().await
    }
}

#[async_trait]
impl BlockStoreDeleter for DynBlockStore {
    async fn remove(&self, id: &BlockId) -> Result<low_level::RemoveResult> {
        self.0.remove(id).await
    }
}

#[async_trait]
impl BlockStoreWriter for DynBlockStore {
    async fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<low_level::TryCreateResult> {
        self.0.try_create(id, data).await
    }

    async fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        self.0.store(id, data).await
    }
}

impl BlockStore for DynBlockStore {}

struct RustBlockStore2Bridge(Box<dyn BlockStore>);

impl RustBlockStore2Bridge {
    fn try_create(&self, id: &BlockId, data: &[u8]) -> Result<bool> {
        // TODO Can we avoid a copy at the ffi boundary? i.e. use OptimizedBlockStoreWriter?
        match TOKIO_RUNTIME.block_on(self.0.try_create(id, data))? {
            low_level::TryCreateResult::SuccessfullyCreated => Ok(true),
            low_level::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => Ok(false),
        }
    }
    fn remove(&self, id: &BlockId) -> Result<bool> {
        match TOKIO_RUNTIME.block_on(self.0.remove(id))? {
            low_level::RemoveResult::SuccessfullyRemoved => Ok(true),
            low_level::RemoveResult::NotRemovedBecauseItDoesntExist => Ok(false),
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

fn new_locking_inmemory_blockstore() -> Box<RustBlockStoreBridge> {
    LOGGER_INIT.ensure_initialized();
    Box::new(RustBlockStoreBridge(LockingBlockStore::new(DynBlockStore(
        Box::new(InMemoryBlockStore::new()),
    ))))
}
