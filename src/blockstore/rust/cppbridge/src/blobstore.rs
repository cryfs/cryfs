use anyhow::{bail, Result};
use std::sync::Mutex;

use super::blockstore::DynBlockStore;
use super::runtime::{LOGGER_INIT, TOKIO_RUNTIME};
use crate::utils::log_errors;
use cryfs_blockstore::{
    blobstore::{
        on_blocks::{BlobOnBlocks, BlobStoreOnBlocks},
        Blob, BlobStore, RemoveResult,
    },
    blockstore::BLOCKID_LEN,
    blockstore::{high_level::LockingBlockStore, low_level::inmemory::InMemoryBlockStore},
    utils::async_drop::AsyncDropGuard,
};

#[cxx::bridge]
mod ffi {
    #[namespace = "blockstore::rust"]
    unsafe extern "C++" {
        include!("blockstore/implementations/rustbridge/CxxCallback.h");
        type CxxCallback = super::super::blockstore::ffi::CxxCallback;
    }

    #[namespace = "blobstore::rust::bridge"]
    extern "Rust" {
        type BlobId;
        fn data(&self) -> &[u8; 16]; // TODO Instead of '16' we should use BLOCKID_LEN here
        fn new_blobid(id: &[u8; 16]) -> Box<BlobId>;
    }

    #[namespace = "blobstore::rust::bridge"]
    extern "Rust" {
        type Data;
        fn data(&self) -> &[u8];
    }

    #[namespace = "blobstore::rust::bridge"]
    extern "Rust" {
        type RustBlobStoreBridge;
        fn create(&self) -> Result<Box<RustBlobBridge>>;
        fn load(&self, blob_id: &BlobId) -> Result<Box<OptionRustBlobBridge>>;
        fn num_nodes(&self) -> Result<u64>;
        fn remove_by_id(&self, id: &BlobId) -> Result<()>;
        fn estimate_space_for_num_blocks_left(&self) -> Result<u64>;
        fn virtual_block_size_bytes(&self) -> u32;
        fn async_drop(&mut self) -> Result<()>;

        fn new_inmemory_blobstore(block_size_bytes: u32) -> Result<Box<RustBlobStoreBridge>>;
        fn new_locking_integrity_encrypted_ondisk_blobstore(
            integrity_file_path: &str,
            my_client_id: u32,
            allow_integrity_violations: bool,
            missing_block_is_integrity_violation: bool,
            on_integrity_violation: UniquePtr<CxxCallback>,
            cipher_name: &str,
            encryption_key_hex: &str,
            basedir: &str,
            block_size_bytes: u32,
        ) -> Result<Box<RustBlobStoreBridge>>;
        fn new_locking_integrity_encrypted_inmemory_blobstore(
            integrity_file_path: &str,
            my_client_id: u32,
            allow_integrity_violations: bool,
            missing_block_is_integrity_violation: bool,
            on_integrity_violation: UniquePtr<CxxCallback>,
            cipher_name: &str,
            encryption_key_hex: &str,
            block_size_bytes: u32,
        ) -> Result<Box<RustBlobStoreBridge>>;
    }

    #[namespace = "blobstore::rust::bridge"]
    extern "Rust" {
        type RustBlobBridge;
        fn blob_id(&self) -> Box<BlobId>;
        fn num_bytes(&self) -> Result<u64>;
        fn resize(&self, new_num_bytes: u64) -> Result<()>;
        fn read_all(&self) -> Result<Box<Data>>;
        fn read(&self, target: &mut [u8], offset: u64) -> Result<()>;
        fn try_read(&self, target: &mut [u8], offset: u64) -> Result<usize>;
        fn write(&self, source: &[u8], offset: u64) -> Result<()>;
        fn flush(&self) -> Result<()>;
        fn num_nodes(&self) -> Result<u64>;
        fn remove(&self) -> Result<()>;
        fn async_drop(&mut self) -> Result<()>;
    }

    #[namespace = "blobstore::rust::bridge"]
    extern "Rust" {
        type OptionRustBlobBridge;
        fn has_value(&self) -> bool;
        fn extract_value(&mut self) -> Result<Box<RustBlobBridge>>;
    }
}

fn new_blobid(data: &[u8; BLOCKID_LEN]) -> Box<BlobId> {
    Box::new(BlobId(cryfs_blockstore::blobstore::BlobId::from_array(
        data,
    )))
}

pub struct BlobId(cryfs_blockstore::blobstore::BlobId);
impl BlobId {
    fn data(&self) -> &[u8; BLOCKID_LEN] {
        self.0.data()
    }
}

pub struct Data(cryfs_blockstore::data::Data);

impl Data {
    fn data(&self) -> &[u8] {
        self.0.as_ref()
    }
}

pub struct OptionRustBlobBridge(Option<RustBlobBridge>);

impl OptionRustBlobBridge {
    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn extract_value(&mut self) -> Result<Box<RustBlobBridge>> {
        log_errors(|| match self.0.take() {
            None => bail!("OptionRustBlobBridge doesn't have a value"),
            Some(data) => Ok(Box::new(data)),
        })
    }
}

struct RustBlobBridge(Mutex<Option<AsyncDropGuard<BlobOnBlocks<DynBlockStore>>>>);

impl RustBlobBridge {
    // TODO If we manage to change the read-only methods to take &self instead of &mut self, we might not need the Mutex anymore, or maybe RwLock would work.
    fn new(blob: AsyncDropGuard<BlobOnBlocks<DynBlockStore>>) -> Self {
        Self(Mutex::new(Some(blob)))
    }

    fn blob_id(&self) -> Box<BlobId> {
        let blob = self.0.lock().unwrap();
        let blob = blob.as_ref().expect("Blob is already destructed");
        Box::new(BlobId(blob.id()))
    }

    fn num_bytes(&self) -> Result<u64> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(blob.num_bytes()))
    }

    fn resize(&self, new_num_bytes: u64) -> Result<()> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(blob.resize(new_num_bytes)))
    }

    fn read_all(&self) -> Result<Box<Data>> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || Ok(Box::new(Data(TOKIO_RUNTIME.block_on(blob.read_all())?))))
    }

    fn read(&self, target: &mut [u8], offset: u64) -> Result<()> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(blob.read(target, offset)))
    }

    fn try_read(&self, target: &mut [u8], offset: u64) -> Result<usize> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(blob.try_read(target, offset)))
    }

    fn write(&self, source: &[u8], offset: u64) -> Result<()> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(blob.write(source, offset)))
    }

    fn flush(&self) -> Result<()> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(blob.flush()))
    }

    fn num_nodes(&self) -> Result<u64> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.as_mut().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(blob.num_nodes()))
    }

    fn remove(&self) -> Result<()> {
        let mut blob = self.0.lock().unwrap();
        let blob = blob.take().expect("Blob is already destructed");
        log_errors(move || TOKIO_RUNTIME.block_on(BlobOnBlocks::remove(blob)))
    }

    fn async_drop(&mut self) -> Result<()> {
        let mut blob = self.0.lock().unwrap();
        let mut blob = blob.take().expect("Blob is already destructed");
        log_errors(|| TOKIO_RUNTIME.block_on(blob.async_drop()))
    }
}

// TODO Do DynBlobStore instead of BlobStoreOnBlocks<DynBlockStore>? Having the dyn layer further outside means dynamic calls only when calling into the BlobStore, not when BlobStore calls into BlockStore, which is much more frequent.
struct RustBlobStoreBridge(AsyncDropGuard<BlobStoreOnBlocks<DynBlockStore>>);

impl RustBlobStoreBridge {
    fn create(&self) -> Result<Box<RustBlobBridge>> {
        log_errors(|| {
            Ok(Box::new(RustBlobBridge::new(
                TOKIO_RUNTIME.block_on(self.0.create())?,
            )))
        })
    }

    fn load(&self, blob_id: &BlobId) -> Result<Box<OptionRustBlobBridge>> {
        log_errors(|| match TOKIO_RUNTIME.block_on(self.0.load(&blob_id.0))? {
            Some(blob) => Ok(Box::new(OptionRustBlobBridge(Some(RustBlobBridge::new(
                blob,
            ))))),
            None => Ok(Box::new(OptionRustBlobBridge(None))),
        })
    }

    fn remove_by_id(&self, id: &BlobId) -> Result<()> {
        log_errors(
            || match TOKIO_RUNTIME.block_on(self.0.remove_by_id(&id.0))? {
                RemoveResult::SuccessfullyRemoved =>
                /* do nothing */
                {
                    Ok(())
                }
                RemoveResult::NotRemovedBecauseItDoesntExist => {
                    bail!("Tried to remove blob {:?} but didn't find it", id.0)
                }
            },
        )
    }

    fn num_nodes(&self) -> Result<u64> {
        log_errors(|| TOKIO_RUNTIME.block_on(self.0.num_nodes()))
    }

    fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        log_errors(|| self.0.estimate_space_for_num_blocks_left())
    }

    fn virtual_block_size_bytes(&self) -> u32 {
        self.0.virtual_block_size_bytes()
    }

    fn async_drop(&mut self) -> Result<()> {
        log_errors(|| TOKIO_RUNTIME.block_on(self.0.async_drop()))
    }
}

fn new_inmemory_blobstore(block_size_bytes: u32) -> Result<Box<RustBlobStoreBridge>> {
    LOGGER_INIT.ensure_initialized();
    Ok(Box::new(RustBlobStoreBridge(BlobStoreOnBlocks::new(
        LockingBlockStore::new(DynBlockStore::from(InMemoryBlockStore::new().into_box())),
        block_size_bytes,
    )?)))
}

fn new_locking_integrity_encrypted_ondisk_blobstore(
    integrity_file_path: &str,
    my_client_id: u32,
    allow_integrity_violations: bool,
    missing_block_is_integrity_violation: bool,
    on_integrity_violation: cxx::UniquePtr<ffi::CxxCallback>,
    cipher_name: &str,
    encryption_key_hex: &str,
    basedir: &str,
    block_size_bytes: u32,
) -> Result<Box<RustBlobStoreBridge>> {
    LOGGER_INIT.ensure_initialized();
    let _init_tokio = TOKIO_RUNTIME.enter();

    log_errors(|| {
        let blockstore = super::blockstore::new_locking_integrity_encrypted_ondisk_blockstore(
            integrity_file_path,
            my_client_id,
            allow_integrity_violations,
            missing_block_is_integrity_violation,
            on_integrity_violation,
            cipher_name,
            encryption_key_hex,
            basedir,
        )?;
        Ok(Box::new(RustBlobStoreBridge(BlobStoreOnBlocks::new(
            blockstore.extract(),
            block_size_bytes,
        )?)))
    })
}

fn new_locking_integrity_encrypted_inmemory_blobstore(
    integrity_file_path: &str,
    my_client_id: u32,
    allow_integrity_violations: bool,
    missing_block_is_integrity_violation: bool,
    on_integrity_violation: cxx::UniquePtr<ffi::CxxCallback>,
    cipher_name: &str,
    encryption_key_hex: &str,
    block_size_bytes: u32,
) -> Result<Box<RustBlobStoreBridge>> {
    LOGGER_INIT.ensure_initialized();
    let _init_tokio = TOKIO_RUNTIME.enter();

    log_errors(|| {
        let blockstore = super::blockstore::new_locking_integrity_encrypted_inmemory_blockstore(
            integrity_file_path,
            my_client_id,
            allow_integrity_violations,
            missing_block_is_integrity_violation,
            on_integrity_violation,
            cipher_name,
            encryption_key_hex,
        )?;
        Ok(Box::new(RustBlobStoreBridge(BlobStoreOnBlocks::new(
            blockstore.extract(),
            block_size_bytes,
        )?)))
    })
}
