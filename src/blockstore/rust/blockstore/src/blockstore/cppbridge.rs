use anyhow::{bail, Result};
use async_trait::async_trait;
use futures::stream::Stream;
use futures::TryStreamExt;
use std::path::{PathBuf, Path};
use std::pin::Pin;

use super::high_level::{self, Block, LockingBlockStore};
use super::low_level::{
    self,
    caching::CachingBlockStore,
    encrypted::EncryptedBlockStore,
    inmemory::InMemoryBlockStore,
    integrity::{ClientId, IntegrityBlockStore, IntegrityConfig},
    ondisk::OnDiskBlockStore,
    BlockStore, BlockStoreDeleter, BlockStoreReader, BlockStoreWriter,
    OptimizedBlockStoreWriter,
};
use crate::blockstore::{BlockId, BLOCKID_LEN};
use crate::crypto::symmetric::{Aes256Gcm, Cipher, EncryptionKey, self, CipherCallback};
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
        fn new_locking_integrity_encrypted_ondisk_blockstore(integrity_file_path: &str, my_client_id: u32, allow_integrity_violations: bool, missing_block_is_integrity_violation: bool, cipher_name: &str, encryption_key_hex: &str, basedir: &str) -> Result<Box<RustBlockStoreBridge>>;
        fn new_locking_integrity_encrypted_inmemory_blockstore(integrity_file_path: &str, my_client_id: u32, allow_integrity_violations: bool, missing_block_is_integrity_violation: bool, cipher_name: &str, encryption_key_hex: &str) -> Result<Box<RustBlockStoreBridge>>;
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type RustBlockStoreBridge;
        fn create_block_id(&self) -> Box<BlockId>;
        fn try_create(&self, block_id: &BlockId, data: &[u8])
            -> Result<Box<OptionRustBlockBridge>>;
        fn load(&self, block_id: &BlockId) -> Result<Box<OptionRustBlockBridge>>;
        fn overwrite(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<RustBlockBridge>>;
        fn remove(&self, block_id: &BlockId) -> Result<bool>;
        fn num_blocks(&self) -> Result<u64>;
        fn estimate_num_free_bytes(&self) -> Result<u64>;
        fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64>;
        fn all_blocks(&self) -> Result<Vec<BlockId>>;
        fn async_drop(&mut self) -> Result<()>;
    }

    #[namespace = "blockstore::rust::bridge"]
    extern "Rust" {
        type RustBlockBridge;
        fn block_id(&self) -> Box<BlockId>;
        fn size(&self) -> usize;
        fn flush(&mut self) -> Result<()>;
        fn resize(&mut self, new_size: usize);
        fn data(&self) -> &[u8];
        fn write(&mut self, source: &[u8], offset: usize) -> Result<()>;
    }
}

fn log_errors<R>(f: impl FnOnce() -> Result<R>) -> Result<R> {
    match f() {
        Ok(ok) => Ok(ok),
        Err(err) => {
            log::error!("Error: {:?}", err);
            Err(err)
        }
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
        log_errors(|| -> Result<&[u8]> {
            match &self.0 {
                None => bail!("OptionData doesn't have a value"),
                Some(data) => Ok(data),
            }
        })
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
    // TODO Will this runtime only execute things while an active call to rust is ongoing? Should we move it to a new thread
    //      and drive futures from there so that the runtime can execute even if we're currently mostly doing C++ stuff?
    static ref TOKIO_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    static ref LOGGER_INIT: LoggerInit = LoggerInit::new();
}

// Invariant: Option is always Some() unless the value was dropped
struct RustBlockBridge(Option<Block<DynBlockStore>>);

impl RustBlockBridge {
    fn new(block: Block<DynBlockStore>) -> Self {
        Self(Some(block))
    }

    fn block_id(&self) -> Box<BlockId> {
        Box::new(
            *self
                .0
                .as_ref()
                .expect("Block was already dropped")
                .block_id(),
        )
    }

    fn size(&self) -> usize {
        self.0
            .as_ref()
            .expect("Block was already dropped")
            .data()
            .len()
    }

    fn flush(&mut self) -> Result<()> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(self.0.as_mut().expect("Block was already dropped").flush())
        })
    }

    fn resize(&mut self, new_size: usize) {
        TOKIO_RUNTIME.block_on(
            self.0
                .as_mut()
                .expect("Block was already dropped")
                .resize(new_size),
        )
    }

    fn write(&mut self, source: &[u8], offset: usize) -> Result<()> {
        log_errors(|| {
            let s = self.0.as_mut().expect("Block was already dropped");
            let dest = &mut s.data_mut()[offset..(offset + source.len())];
            if dest.len() != source.len() {
                bail!("Tried to write out of block boundaries. Write offset {}, size {} but block size is {}", offset, source.len(), s.data().len());
            }
            dest.copy_from_slice(source);
            Ok(())
        })
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
        log_errors(|| {
            match self.0.take() {
                None => bail!("OptionRustBlockBridge doesn't have a value"),
                Some(data) => Ok(Box::new(data)),
            }
        })
    }
}

struct RustBlockStoreBridge(Option<AsyncDropGuard<LockingBlockStore<DynBlockStore>>>);

impl RustBlockStoreBridge {
    fn create_block_id(&self) -> Box<BlockId> {
        Box::new(BlockId::new_random())
    }

    fn _underlying(&self) -> &LockingBlockStore<DynBlockStore> {
        &self
            .0
            .as_ref()
            .expect("Tried to access RustBlockStoreBridge after it was destructed")
    }

    async fn _try_create(
        &self,
        block_id: &BlockId,
        data: &[u8],
    ) -> Result<Box<OptionRustBlockBridge>> {
        let underlying = self._underlying();
        match underlying
            .try_create(block_id, &data.to_vec().into())
            .await?
        {
            high_level::TryCreateResult::SuccessfullyCreated => {
                let loaded = underlying
                    .load(*block_id)
                    .await?
                    .expect("We just created this but it doesn't exist?");
                Ok(Box::new(OptionRustBlockBridge(Some(RustBlockBridge::new(
                    loaded,
                )))))
            }
            high_level::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => {
                Ok(Box::new(OptionRustBlockBridge(None)))
            }
        }
    }

    fn try_create(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<OptionRustBlockBridge>> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(self._try_create(block_id, data))
        })
    }

    fn load(&self, block_id: &BlockId) -> Result<Box<OptionRustBlockBridge>> {
        log_errors(|| {
            match TOKIO_RUNTIME.block_on(self._underlying().load(*block_id))? {
                Some(block) => Ok(Box::new(OptionRustBlockBridge(Some(RustBlockBridge::new(
                    block,
                ))))),
                None => Ok(Box::new(OptionRustBlockBridge(None))),
            }
        })
    }

    async fn _overwrite(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<RustBlockBridge>> {
        // TODO Overwriting and then loading could be slow. Should we instead change the rust API so that it also returns the block from the overwrite() call?
        let underlying = self._underlying();
        underlying
            .overwrite(block_id, &data.to_vec().into())
            .await?;
        let loaded = underlying
            .load(*block_id)
            .await?
            .expect("We just created this but it doesn't exist?");
        Ok(Box::new(RustBlockBridge::new(loaded)))
    }

    fn overwrite(&self, block_id: &BlockId, data: &[u8]) -> Result<Box<RustBlockBridge>> {
        log_errors(||
            TOKIO_RUNTIME.block_on(self._overwrite(block_id, data))
        )
    }

    fn remove(&self, block_id: &BlockId) -> Result<bool> {
        log_errors(|| {
            match TOKIO_RUNTIME.block_on(self._underlying().remove(block_id))? {
                high_level::RemoveResult::SuccessfullyRemoved => Ok(true),
                high_level::RemoveResult::NotRemovedBecauseItDoesntExist => Ok(false),
            }
        })
    }

    fn num_blocks(&self) -> Result<u64> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(self._underlying().num_blocks())
        })
    }

    fn estimate_num_free_bytes(&self) -> Result<u64> {
        log_errors(|| {
            self._underlying().estimate_num_free_bytes()
        })
    }

    fn block_size_from_physical_block_size(&self, block_size: u64) -> Result<u64> {
        log_errors(|| {
            self._underlying()
                .block_size_from_physical_block_size(block_size)
        })
    }

    fn all_blocks(&self) -> Result<Vec<BlockId>> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(async {
                TryStreamExt::try_collect(self._underlying().all_blocks().await?).await
            })
        })
    }

    fn async_drop(&mut self) -> Result<()> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(
                self.0
                    .take()
                    .expect("Tried to destruct RustBlockStoreBridge after it is already destructed")
                    .async_drop(),
            )
        })
    }
}

struct DynBlockStore(Box<dyn BlockStore + Send + Sync>);

#[async_trait]
impl BlockStoreReader for DynBlockStore {
    async fn exists(&self, id: &BlockId) -> Result<bool> {
        self.0.exists(id).await
    }

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
        log_errors(|| {
            // TODO Can we avoid a copy at the ffi boundary? i.e. use OptimizedBlockStoreWriter?
            match TOKIO_RUNTIME.block_on(self.0.try_create(id, data))? {
                low_level::TryCreateResult::SuccessfullyCreated => Ok(true),
                low_level::TryCreateResult::NotCreatedBecauseBlockIdAlreadyExists => Ok(false),
            }
        })
    }
    fn remove(&self, id: &BlockId) -> Result<bool> {
        log_errors(|| {
            match TOKIO_RUNTIME.block_on(self.0.remove(id))? {
                low_level::RemoveResult::SuccessfullyRemoved => Ok(true),
                low_level::RemoveResult::NotRemovedBecauseItDoesntExist => Ok(false),
            }
        })
    }
    fn load(&self, id: &BlockId) -> Result<Box<OptionData>> {
        log_errors(|| {
            let loaded = TOKIO_RUNTIME.block_on(self.0.load(id))?;
            Ok(Box::new(OptionData(loaded)))
        })
    }
    fn store(&self, id: &BlockId, data: &[u8]) -> Result<()> {
        log_errors(|| {
            // TODO Can we avoid a copy at the ffi boundary? i.e. use OptimizedBlockStoreWriter?
            TOKIO_RUNTIME.block_on(self.0.store(id, data))
        })
    }
    fn num_blocks(&self) -> Result<u64> {
        log_errors(|| {
            Ok(TOKIO_RUNTIME.block_on(self.0.num_blocks()).unwrap())
        })
    }
    fn estimate_num_free_bytes(&self) -> Result<u64> {
        log_errors(|| {
            self.0.estimate_num_free_bytes()
        })
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
        log_errors(|| {
            TOKIO_RUNTIME
                .block_on(async { TryStreamExt::try_collect(self.0.all_blocks().await?).await })
        })
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
    log_errors(|| {
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
    })
}

fn new_ondisk_blockstore(basedir: &str) -> Box<RustBlockStore2Bridge> {
    LOGGER_INIT.ensure_initialized();
    Box::new(RustBlockStore2Bridge(Box::new(OnDiskBlockStore::new(
        Path::new(basedir).to_path_buf(),
    ))))
}

fn new_locking_inmemory_blockstore() -> Box<RustBlockStoreBridge> {
    LOGGER_INIT.ensure_initialized();
    let _init_tokio = TOKIO_RUNTIME.enter();
    Box::new(RustBlockStoreBridge(Some(LockingBlockStore::new(
        DynBlockStore(Box::new(InMemoryBlockStore::new())),
    ))))
}

struct _BlockStoreCreator<'a, B> {
    integrity_file_path: PathBuf,
    my_client_id: ClientId,
    integrity_config: IntegrityConfig,
    encryption_key_hex: &'a str,
    base_store: B,
}

impl <'a, B: BlockStore + OptimizedBlockStoreWriter + Send + Sync + 'static> CipherCallback for _BlockStoreCreator<'a, B> {
    type Result = Result<Box<RustBlockStoreBridge>>;

    fn callback<C: Cipher + Send + Sync + 'static>(self) -> Result<Box<RustBlockStoreBridge>> {
        Ok(Box::new(RustBlockStoreBridge(Some(LockingBlockStore::new(
            DynBlockStore(
                Box::new(
                    IntegrityBlockStore::new(
                        EncryptedBlockStore::new(
                            self.base_store,
                            C::new(EncryptionKey::<C::KeySize>::from_hex(self.encryption_key_hex)?),
                        ),
                        self.integrity_file_path,
                        self.my_client_id,
                        self.integrity_config,
                    )?,
                )
            )
        )))))
    }
}

fn _new_locking_integrity_encrypted_blockstore(integrity_file_path: &str, my_client_id: u32, allow_integrity_violations: bool, missing_block_is_integrity_violation: bool, cipher_name: &str, encryption_key_hex: &str, base_store: impl BlockStore + OptimizedBlockStoreWriter + Send + Sync + 'static) -> Result<Box<RustBlockStoreBridge>> {
    symmetric::lookup_cipher(cipher_name, _BlockStoreCreator {
        integrity_file_path: Path::new(integrity_file_path).to_path_buf(),
        my_client_id: ClientId{id: my_client_id},
        integrity_config: IntegrityConfig {
            allow_integrity_violations,
            missing_block_is_integrity_violation,
            on_integrity_violation: Box::new(|| {
                // TODO
                todo!()
            }),
        },
        encryption_key_hex,
        base_store,
    })
}

fn new_locking_integrity_encrypted_ondisk_blockstore(integrity_file_path: &str, my_client_id: u32, allow_integrity_violations: bool, missing_block_is_integrity_violation: bool, cipher_name: &str, encryption_key_hex: &str, basedir: &str) -> Result<Box<RustBlockStoreBridge>> {
    LOGGER_INIT.ensure_initialized();
    let _init_tokio = TOKIO_RUNTIME.enter();

    log_errors(|| {
        _new_locking_integrity_encrypted_blockstore(integrity_file_path, my_client_id, allow_integrity_violations, missing_block_is_integrity_violation, cipher_name, encryption_key_hex, OnDiskBlockStore::new(Path::new(basedir).to_path_buf()))
    })
}

fn new_locking_integrity_encrypted_inmemory_blockstore(integrity_file_path: &str, my_client_id: u32, allow_integrity_violations: bool, missing_block_is_integrity_violation: bool, cipher_name: &str, encryption_key_hex: &str) -> Result<Box<RustBlockStoreBridge>> {
    LOGGER_INIT.ensure_initialized();
    let _init_tokio = TOKIO_RUNTIME.enter();

    log_errors(|| {
        _new_locking_integrity_encrypted_blockstore(integrity_file_path, my_client_id, allow_integrity_violations, missing_block_is_integrity_violation, cipher_name, encryption_key_hex, InMemoryBlockStore::new())
    })
}
