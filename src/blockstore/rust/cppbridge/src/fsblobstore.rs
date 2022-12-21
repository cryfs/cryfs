use anyhow::{bail, Result};
use cryfs_blockstore::blobstore::{on_blocks::BlobStoreOnBlocks, BLOBID_LEN};
use cryfs_blockstore::cryfs::fsblobstore::{
    AtimeUpdateBehavior, DirBlob, DirEntry, EntryType, FileBlob, FsBlob, FsBlobStore, FsError,
    SymlinkBlob,
};
use cryfs_blockstore::cryfs::utils::fs_types::{Gid, Uid};
use cryfs_blockstore::utils::async_drop::AsyncDropGuard;
use cxx::UniquePtr;
use futures::{StreamExt, TryStreamExt};
use std::time::{Duration, SystemTime};

use super::blockstore::DynBlockStore;
use super::runtime::{LOGGER_INIT, TOKIO_RUNTIME};
use super::utils::log_errors;

#[cxx::bridge]
mod ffi {
    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    #[derive(Clone, Copy)]
    struct RustTimespec {
        tv_sec: u64,
        tv_nsec: u32,
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    #[derive(Clone, Copy)]
    enum RustEntryType {
        Dir = 0x00,
        File = 0x01,
        Symlink = 0x02,
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    #[derive(Clone, Copy)]
    enum AtimeUpdateBehavior {
        Noatime,
        Strictatime,
        Relatime,
        NodiratimeRelatime,
        NodiratimeStrictatime,
    }

    #[namespace = "blockstore::rust"]
    unsafe extern "C++" {
        include!("blockstore/implementations/rustbridge/CxxCallback.h");
        type CxxCallback = super::super::blockstore::ffi::CxxCallback;
    }

    #[namespace = "cryfs::fsblobstore::rust"]
    unsafe extern "C++" {
        include!("cryfs/impl/filesystem/rustfsblobstore/CxxCallbackWithBlobId.h");
        type CxxCallbackWithBlobId;
        fn call(&self, blob_id: &FsBlobId) -> Result<()>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type FsResult;
        fn is_err(&self) -> bool;
        fn is_errno_error(&self) -> bool;
        fn err_errno(&self) -> i32;
        fn err_message(&self) -> String;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type FsBlobId;
        fn data(&self) -> &[u8; 16]; // TODO Instead of '16' we should use BLOCKID_LEN here
        fn new_blobid(id: &[u8; 16]) -> Box<FsBlobId>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type RustFsBlobBridge<'a>;
        fn blob_id(&self) -> Box<FsBlobId>;
        fn parent(&self) -> Box<FsBlobId>;
        fn set_parent(&mut self, parent: &FsBlobId) -> Result<()>;
        fn is_file(&self) -> bool;
        fn is_dir(&self) -> bool;
        fn is_symlink(&self) -> bool;
        unsafe fn to_file<'a>(
            self: &'a mut RustFsBlobBridge<'a>,
        ) -> Result<Box<RustFileBlobBridge<'a>>>;
        unsafe fn to_dir<'a>(
            self: &'a mut RustFsBlobBridge<'a>,
        ) -> Result<Box<RustDirBlobBridge<'a>>>;
        unsafe fn to_symlink<'a>(
            self: &'a mut RustFsBlobBridge<'a>,
        ) -> Result<Box<RustSymlinkBlobBridge<'a>>>;
        fn remove(&mut self) -> Result<()>;
        fn lstat_size(&mut self) -> Result<u64>;
        fn all_blocks(&self) -> Result<Vec<FsBlobId>>;
        fn async_drop(&mut self) -> Result<()>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type RustFileBlobBridge<'a>;
        fn blob_id(&self) -> Box<FsBlobId>;
        fn parent(&self) -> Box<FsBlobId>;
        fn num_bytes(&mut self) -> Result<u64>;
        fn resize(&mut self, new_num_bytes: u64) -> Result<()>;
        fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize>;
        fn write(&mut self, source: &[u8], offset: u64) -> Result<()>;
        fn flush(&mut self) -> Result<()>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type RustDirBlobBridge<'a>;
        fn blob_id(&self) -> Box<FsBlobId>;
        fn parent(&self) -> Box<FsBlobId>;
        fn num_entries(&self) -> usize;
        fn entries(&self) -> Vec<RustDirEntryBridge>;
        fn flush(&mut self) -> Result<()>;
        fn entry_by_id(&self, id: &FsBlobId) -> Box<OptionRustDirEntryBridge>;
        fn entry_by_name(&self, name: &str) -> Result<Box<OptionRustDirEntryBridge>>;
        fn rename_entry(
            &mut self,
            blob_id: &FsBlobId,
            new_name: &str,
            on_overwritten: UniquePtr<CxxCallbackWithBlobId>,
        ) -> Box<FsResult>;
        fn update_modification_timestamp_of_entry(&mut self, blob_id: &FsBlobId) -> Box<FsResult>;
        fn maybe_update_access_timestamp_of_entry(
            &mut self,
            blob_id: &FsBlobId,
            atime_update_behavior: AtimeUpdateBehavior,
        ) -> Box<FsResult>;
        fn set_mode_of_entry(&mut self, blob_id: &FsBlobId, mode: u32) -> Box<FsResult>;
        fn set_uid_gid_of_entry(
            &mut self,
            blob_id: &FsBlobId,
            uid: &OptionU32,
            gid: &OptionU32,
        ) -> Box<FsResult>;
        fn set_access_times_of_entry(
            &mut self,
            blob_id: &FsBlobId,
            last_access_time: RustTimespec,
            last_modification_time: RustTimespec,
        ) -> Box<FsResult>;
        fn add_entry_dir(
            &mut self,
            name: &str,
            id: &FsBlobId,
            mode: u32,
            uid: u32,
            gid: u32,
            last_access_time: RustTimespec,
            last_modification_time: RustTimespec,
        ) -> Box<FsResult>;
        fn add_entry_file(
            &mut self,
            name: &str,
            id: &FsBlobId,
            mode: u32,
            uid: u32,
            gid: u32,
            last_access_time: RustTimespec,
            last_modification_time: RustTimespec,
        ) -> Box<FsResult>;
        fn add_entry_symlink(
            &mut self,
            name: &str,
            id: &FsBlobId,
            uid: u32,
            gid: u32,
            last_access_time: RustTimespec,
            last_modification_time: RustTimespec,
        ) -> Box<FsResult>;
        fn add_or_overwrite_entry(
            &mut self,
            name: &str,
            id: &FsBlobId,
            entry_type: RustEntryType,
            mode: u32,
            uid: u32,
            gid: u32,
            last_access_time: RustTimespec,
            last_modification_time: RustTimespec,
            on_overwritten: UniquePtr<CxxCallbackWithBlobId>,
        ) -> Box<FsResult>;
        fn remove_entry_by_name(&mut self, name: &str) -> Box<FsResult>;
        fn remove_entry_by_id_if_exists(&mut self, blob_id: &FsBlobId);
        fn async_drop(&mut self) -> Result<()>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type RustSymlinkBlobBridge<'a>;
        fn blob_id(&self) -> Box<FsBlobId>;
        fn parent(&self) -> Box<FsBlobId>;
        fn target(&mut self) -> Result<String>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type RustFsBlobStoreBridge;
        unsafe fn create_file_blob<'a>(
            &'a self,
            parent: &FsBlobId,
        ) -> Result<Box<RustFileBlobBridge<'a>>>;
        unsafe fn create_dir_blob<'a>(
            &'a self,
            parent: &FsBlobId,
        ) -> Result<Box<RustDirBlobBridge<'a>>>;
        unsafe fn create_symlink_blob<'a>(
            &'a self,
            parent: &FsBlobId,
            target: &str,
        ) -> Result<Box<RustSymlinkBlobBridge<'a>>>;
        unsafe fn load<'a>(&'a self, blob_id: &FsBlobId)
            -> Result<Box<OptionRustFsBlobBridge<'a>>>;
        fn num_blocks(&self) -> Result<u64>;
        fn estimate_space_for_num_blocks_left(&self) -> Result<u64>;
        fn virtual_block_size_bytes(&self) -> u32;
        fn load_block_depth(&self, block_id: &FsBlobId) -> Result<u8>;
        fn async_drop(&mut self) -> Result<()>;

        fn new_locking_integrity_encrypted_inmemory_fsblobstore(
            integrity_file_path: &str,
            my_client_id: u32,
            allow_integrity_violations: bool,
            missing_block_is_integrity_violation: bool,
            on_integrity_violation: UniquePtr<CxxCallback>,
            cipher_name: &str,
            encryption_key_hex: &str,
            block_size_bytes: u32,
        ) -> Result<Box<RustFsBlobStoreBridge>>;
        fn new_locking_integrity_encrypted_readonly_ondisk_fsblobstore(
            integrity_file_path: &str,
            my_client_id: u32,
            allow_integrity_violations: bool,
            missing_block_is_integrity_violation: bool,
            on_integrity_violation: UniquePtr<CxxCallback>,
            cipher_name: &str,
            encryption_key_hex: &str,
            basedir: &str,
            block_size_bytes: u32,
        ) -> Result<Box<RustFsBlobStoreBridge>>;
        fn new_locking_integrity_encrypted_ondisk_fsblobstore(
            integrity_file_path: &str,
            my_client_id: u32,
            allow_integrity_violations: bool,
            missing_block_is_integrity_violation: bool,
            on_integrity_violation: UniquePtr<CxxCallback>,
            cipher_name: &str,
            encryption_key_hex: &str,
            basedir: &str,
            block_size_bytes: u32,
        ) -> Result<Box<RustFsBlobStoreBridge>>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type OptionRustFsBlobBridge<'a>;
        fn has_value(&self) -> bool;
        unsafe fn extract_value<'a>(
            self: &mut OptionRustFsBlobBridge<'a>,
        ) -> Result<Box<RustFsBlobBridge<'a>>>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type RustDirEntryBridge;
        fn entry_type(&self) -> RustEntryType;
        fn mode(&self) -> u32;
        fn uid(&self) -> u32;
        fn gid(&self) -> u32;
        fn last_access_time(&self) -> RustTimespec;
        fn last_modification_time(&self) -> RustTimespec;
        fn last_metadata_change_time(&self) -> RustTimespec;
        fn name(&self) -> Result<&str>;
        fn blob_id(&self) -> Box<FsBlobId>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type OptionRustDirEntryBridge;
        fn has_value(&self) -> bool;
        fn extract_value(&mut self) -> Result<Box<RustDirEntryBridge>>;
    }

    #[namespace = "cryfs::fsblobstore::rust::bridge"]
    extern "Rust" {
        type OptionU32;
        fn new_some_u32(value: u32) -> Box<OptionU32>;
        fn new_none_u32() -> Box<OptionU32>;
    }
}

unsafe impl Send for ffi::CxxCallbackWithBlobId {}

impl From<AtimeUpdateBehavior> for ffi::AtimeUpdateBehavior {
    fn from(behavior: AtimeUpdateBehavior) -> Self {
        match behavior {
            AtimeUpdateBehavior::Noatime => Self::Noatime,
            AtimeUpdateBehavior::Strictatime => Self::Strictatime,
            AtimeUpdateBehavior::Relatime => Self::Relatime,
            AtimeUpdateBehavior::NodiratimeRelatime => Self::NodiratimeRelatime,
            AtimeUpdateBehavior::NodiratimeStrictatime => Self::NodiratimeStrictatime,
        }
    }
}

impl From<ffi::AtimeUpdateBehavior> for AtimeUpdateBehavior {
    fn from(behavior: ffi::AtimeUpdateBehavior) -> Self {
        match behavior {
            ffi::AtimeUpdateBehavior::Noatime => Self::Noatime,
            ffi::AtimeUpdateBehavior::Strictatime => Self::Strictatime,
            ffi::AtimeUpdateBehavior::Relatime => Self::Relatime,
            ffi::AtimeUpdateBehavior::NodiratimeRelatime => Self::NodiratimeRelatime,
            ffi::AtimeUpdateBehavior::NodiratimeStrictatime => Self::NodiratimeStrictatime,
            _ => panic!("Invalid AtimeUpdateBehavior"),
        }
    }
}

fn new_blobid(data: &[u8; BLOBID_LEN]) -> Box<FsBlobId> {
    Box::new(FsBlobId(cryfs_blockstore::blobstore::BlobId::from_array(
        data,
    )))
}

struct FsResult(Option<anyhow::Error>);
impl FsResult {
    fn is_err(&self) -> bool {
        self.0.is_some()
    }

    fn is_errno_error(&self) -> bool {
        if let Some(err) = &self.0 {
            err.downcast_ref::<FsError>().is_some()
        } else {
            false
        }
    }

    fn err_errno(&self) -> i32 {
        let Some(err) = &self.0 else {
            panic!("Not an error");
        };
        let err = err.downcast_ref::<FsError>().expect("Not an errno error");
        match *err {
            FsError::ENOENT { .. } => libc::ENOENT,
            FsError::EISDIR { .. } => libc::EISDIR,
            FsError::ENOTDIR { .. } => libc::ENOTDIR,
            FsError::EEXIST { .. } => libc::EEXIST,
        }
    }
    fn err_message(&self) -> String {
        let Some(err) = &self.0 else {
            panic!("Not an error");
        };
        err.to_string()
    }
}

impl From<Result<()>> for Box<FsResult> {
    fn from(result: Result<()>) -> Self {
        match result {
            Ok(()) => Box::new(FsResult(None)),
            Err(err) => Box::new(FsResult(Some(err))),
        }
    }
}

struct OptionU32(Option<u32>);

fn new_some_u32(value: u32) -> Box<OptionU32> {
    Box::new(OptionU32(Some(value)))
}

fn new_none_u32() -> Box<OptionU32> {
    Box::new(OptionU32(None))
}

pub struct FsBlobId(pub cryfs_blockstore::blobstore::BlobId);
impl FsBlobId {
    fn data(&self) -> &[u8; BLOBID_LEN] {
        self.0.data()
    }
}

impl From<ffi::RustEntryType> for EntryType {
    fn from(entry_type: ffi::RustEntryType) -> Self {
        const FILE: u8 = EntryType::File as u8;
        const DIR: u8 = EntryType::Dir as u8;
        const SYMLINK: u8 = EntryType::Symlink as u8;
        match entry_type.repr {
            FILE => EntryType::File,
            DIR => EntryType::Dir,
            SYMLINK => EntryType::Symlink,
            v => panic!("Unknown entry type {}", v),
        }
    }
}

impl From<EntryType> for ffi::RustEntryType {
    fn from(entry_type: EntryType) -> Self {
        match entry_type {
            EntryType::File => ffi::RustEntryType::File,
            EntryType::Dir => ffi::RustEntryType::Dir,
            EntryType::Symlink => ffi::RustEntryType::Symlink,
        }
    }
}

impl From<ffi::RustTimespec> for SystemTime {
    fn from(timespec: ffi::RustTimespec) -> Self {
        SystemTime::UNIX_EPOCH + Duration::new(timespec.tv_sec, timespec.tv_nsec)
    }
}

impl From<SystemTime> for ffi::RustTimespec {
    fn from(system_time: SystemTime) -> Self {
        let duration = system_time.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        ffi::RustTimespec {
            tv_sec: duration.as_secs(),
            tv_nsec: duration.subsec_nanos(),
        }
    }
}

pub struct RustDirEntryBridge(DirEntry);

impl RustDirEntryBridge {
    fn entry_type(&self) -> ffi::RustEntryType {
        self.0.entry_type().into()
    }

    fn mode(&self) -> u32 {
        self.0.mode().into()
    }

    fn uid(&self) -> u32 {
        self.0.uid().into()
    }

    fn gid(&self) -> u32 {
        self.0.gid().into()
    }

    fn last_access_time(&self) -> ffi::RustTimespec {
        self.0.last_access_time().into()
    }

    fn last_modification_time(&self) -> ffi::RustTimespec {
        self.0.last_modification_time().into()
    }

    fn last_metadata_change_time(&self) -> ffi::RustTimespec {
        self.0.last_metadata_change_time().into()
    }

    fn name(&self) -> Result<&str> {
        Ok(self.0.name()?)
    }

    fn blob_id(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(*self.0.blob_id()))
    }
}

pub struct OptionRustFsBlobBridge<'a>(Option<RustFsBlobBridge<'a>>);

impl<'a> OptionRustFsBlobBridge<'a> {
    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn extract_value(&mut self) -> Result<Box<RustFsBlobBridge<'a>>> {
        log_errors(|| match self.0.take() {
            None => bail!("OptionRustBlobBridge doesn't have a value"),
            Some(data) => Ok(Box::new(data)),
        })
    }
}

pub struct OptionRustDirEntryBridge(Option<RustDirEntryBridge>);

impl OptionRustDirEntryBridge {
    fn has_value(&self) -> bool {
        self.0.is_some()
    }

    fn extract_value(&mut self) -> Result<Box<RustDirEntryBridge>> {
        log_errors(|| match self.0.take() {
            None => bail!("OptionRustDirEntryBridge doesn't have a value"),
            Some(data) => Ok(Box::new(data)),
        })
    }
}

// Parameter is always Some unless already destructed
struct RustFsBlobBridge<'a>(Option<AsyncDropGuard<FsBlob<'a, BlobStoreOnBlocks<DynBlockStore>>>>);

impl<'a> RustFsBlobBridge<'a> {
    fn new(blob: AsyncDropGuard<FsBlob<'a, BlobStoreOnBlocks<DynBlockStore>>>) -> Self {
        RustFsBlobBridge(Some(blob))
    }

    fn blob_id(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(
            self.0
                .as_ref()
                .expect("FsBlob already destructed")
                .blob_id(),
        ))
    }

    fn parent(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(
            self.0.as_ref().expect("FsBlob already destructed").parent(),
        ))
    }

    fn set_parent(&mut self, parent: &FsBlobId) -> Result<()> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(
                self.0
                    .as_mut()
                    .expect("FsBlob already destructed")
                    .set_parent(&parent.0),
            )
        })
    }

    fn is_file(&self) -> bool {
        matches!(
            **self.0.as_ref().expect("FsBlob already destructed"),
            FsBlob::File(_),
        )
    }

    fn is_dir(&self) -> bool {
        matches!(
            **self.0.as_ref().expect("FsBlob already destructed"),
            FsBlob::Directory(_),
        )
    }

    fn is_symlink(&self) -> bool {
        matches!(
            **self.0.as_ref().expect("FsBlob already destructed"),
            FsBlob::Symlink(_),
        )
    }

    fn to_file(&mut self) -> Result<Box<RustFileBlobBridge<'a>>> {
        Ok(Box::new(RustFileBlobBridge(FsBlob::into_file(
            self.0.take().expect("FsBlob already destructed"),
        )?)))
    }

    fn to_dir(&mut self) -> Result<Box<RustDirBlobBridge<'a>>> {
        Ok(Box::new(RustDirBlobBridge(FsBlob::into_dir(
            self.0.take().expect("FsBlob already destructed"),
        )?)))
    }

    fn to_symlink(&mut self) -> Result<Box<RustSymlinkBlobBridge<'a>>> {
        Ok(Box::new(RustSymlinkBlobBridge(FsBlob::into_symlink(
            self.0.take().expect("FsBlob already destructed"),
        )?)))
    }

    fn remove(&mut self) -> Result<()> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(FsBlob::remove(
                self.0.take().expect("FsBlob already destructed"),
            ))
        })
    }

    fn lstat_size(&mut self) -> Result<u64> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(
                self.0
                    .as_mut()
                    .expect("FsBlob already destructed")
                    .lstat_size(),
            )
        })
    }

    fn all_blocks(&self) -> Result<Vec<FsBlobId>> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(async {
                let blocks = self
                    .0
                    .as_ref()
                    .expect("FsBlob already destructed")
                    .all_blocks()
                    .await?
                    .map(|id| {
                        id.map(|id| {
                            FsBlobId(cryfs_blockstore::blobstore::BlobId::from_array(id.data()))
                        })
                    })
                    .try_collect()
                    .await?;
                Ok(blocks)
            })
        })
    }

    fn async_drop(&mut self) -> Result<()> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(
                self.0
                    .take()
                    .expect("FsBlob already destructed")
                    .async_drop(),
            )
        })
    }
}

struct RustFileBlobBridge<'a>(FileBlob<'a, BlobStoreOnBlocks<DynBlockStore>>);

impl<'a> RustFileBlobBridge<'a> {
    fn blob_id(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(self.0.blob_id()))
    }

    fn num_bytes(&mut self) -> Result<u64> {
        // TODO Does self need to be mut?
        log_errors(|| Ok(TOKIO_RUNTIME.block_on(self.0.num_bytes())?))
    }

    fn resize(&mut self, new_num_bytes: u64) -> Result<()> {
        log_errors(|| Ok(TOKIO_RUNTIME.block_on(self.0.resize(new_num_bytes))?))
    }

    fn try_read(&mut self, target: &mut [u8], offset: u64) -> Result<usize> {
        // TODO Does self need to be mut?
        log_errors(|| Ok(TOKIO_RUNTIME.block_on(self.0.try_read(target, offset))?))
    }

    fn write(&mut self, source: &[u8], offset: u64) -> Result<()> {
        log_errors(|| Ok(TOKIO_RUNTIME.block_on(self.0.write(source, offset))?))
    }

    fn flush(&mut self) -> Result<()> {
        log_errors(|| Ok(TOKIO_RUNTIME.block_on(self.0.flush())?))
    }

    fn parent(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(self.0.parent()))
    }
}

struct RustDirBlobBridge<'a>(AsyncDropGuard<DirBlob<'a, BlobStoreOnBlocks<DynBlockStore>>>);

impl<'a> RustDirBlobBridge<'a> {
    fn flush(&mut self) -> Result<()> {
        log_errors(|| Ok(TOKIO_RUNTIME.block_on(self.0.flush())?))
    }

    fn blob_id(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(self.0.blob_id()))
    }

    fn num_entries(&self) -> usize {
        self.0.num_entries()
    }

    fn entries(&self) -> Vec<RustDirEntryBridge> {
        self.0
            .entries()
            .map(|v| RustDirEntryBridge(v.clone()))
            .collect()
    }

    fn parent(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(self.0.parent()))
    }

    fn entry_by_id(&self, id: &FsBlobId) -> Box<OptionRustDirEntryBridge> {
        Box::new(OptionRustDirEntryBridge(
            self.0
                .entry_by_id(&id.0)
                .map(|v| RustDirEntryBridge(v.clone())),
        ))
    }

    fn entry_by_name(&self, name: &str) -> Result<Box<OptionRustDirEntryBridge>> {
        log_errors(|| {
            Ok(Box::new(OptionRustDirEntryBridge(
                self.0
                    .entry_by_name(name)?
                    .map(|v| RustDirEntryBridge(v.clone())),
            )))
        })
    }

    fn rename_entry(
        &mut self,
        blob_id: &FsBlobId,
        new_name: &str,
        on_overwritten: UniquePtr<ffi::CxxCallbackWithBlobId>,
    ) -> Box<FsResult> {
        log_errors(|| {
            Ok(self.0.rename_entry(&blob_id.0, new_name, |id| {
                on_overwritten.call(&FsBlobId(*id))?;
                Ok(())
            })?)
        })
        .into()
    }

    fn update_modification_timestamp_of_entry(&mut self, blob_id: &FsBlobId) -> Box<FsResult> {
        log_errors(|| Ok(self.0.update_modification_timestamp_of_entry(&blob_id.0)?)).into()
    }

    fn maybe_update_access_timestamp_of_entry(
        &mut self,
        blob_id: &FsBlobId,
        atime_update_behavior: ffi::AtimeUpdateBehavior,
    ) -> Box<FsResult> {
        log_errors(|| {
            self.0
                .maybe_update_access_timestamp_of_entry(&blob_id.0, atime_update_behavior.into())
        })
        .into()
    }

    pub fn set_mode_of_entry(&mut self, blob_id: &FsBlobId, mode: u32) -> Box<FsResult> {
        log_errors(|| self.0.set_mode_of_entry(&blob_id.0, mode.into())).into()
    }

    pub fn set_uid_gid_of_entry(
        &mut self,
        blob_id: &FsBlobId,
        uid: &OptionU32,
        gid: &OptionU32,
    ) -> Box<FsResult> {
        log_errors(|| {
            self.0
                .set_uid_gid_of_entry(&blob_id.0, uid.0.map(Uid::from), gid.0.map(Gid::from))
        })
        .into()
    }

    pub fn set_access_times_of_entry(
        &mut self,
        blob_id: &FsBlobId,
        last_access_time: ffi::RustTimespec,
        last_modification_time: ffi::RustTimespec,
    ) -> Box<FsResult> {
        log_errors(|| {
            self.0.set_access_times_of_entry(
                &blob_id.0,
                last_access_time.into(),
                last_modification_time.into(),
            )
        })
        .into()
    }

    pub fn add_entry_dir(
        &mut self,
        name: &str,
        id: &FsBlobId,
        mode: u32,
        uid: u32,
        gid: u32,
        last_access_time: ffi::RustTimespec,
        last_modification_time: ffi::RustTimespec,
    ) -> Box<FsResult> {
        log_errors(|| {
            self.0.add_entry_dir(
                name,
                id.0,
                mode.into(),
                Uid::from(uid),
                Gid::from(gid),
                last_access_time.into(),
                last_modification_time.into(),
            )
        })
        .into()
    }

    pub fn add_entry_file(
        &mut self,
        name: &str,
        id: &FsBlobId,
        mode: u32,
        uid: u32,
        gid: u32,
        last_access_time: ffi::RustTimespec,
        last_modification_time: ffi::RustTimespec,
    ) -> Box<FsResult> {
        log_errors(|| {
            self.0.add_entry_file(
                name,
                id.0,
                mode.into(),
                Uid::from(uid),
                Gid::from(gid),
                last_access_time.into(),
                last_modification_time.into(),
            )
        })
        .into()
    }

    pub fn add_entry_symlink(
        &mut self,
        name: &str,
        id: &FsBlobId,
        uid: u32,
        gid: u32,
        last_access_time: ffi::RustTimespec,
        last_modification_time: ffi::RustTimespec,
    ) -> Box<FsResult> {
        log_errors(|| {
            self.0.add_entry_symlink(
                name,
                id.0,
                uid.into(),
                gid.into(),
                last_access_time.into(),
                last_modification_time.into(),
            )
        })
        .into()
    }

    pub fn add_or_overwrite_entry(
        &mut self,
        name: &str,
        id: &FsBlobId,
        entry_type: ffi::RustEntryType,
        mode: u32,
        uid: u32,
        gid: u32,
        last_access_time: ffi::RustTimespec,
        last_modification_time: ffi::RustTimespec,
        on_overwritten: UniquePtr<ffi::CxxCallbackWithBlobId>,
    ) -> Box<FsResult> {
        log_errors(|| {
            self.0.add_or_overwrite_entry(
                name,
                id.0,
                entry_type.into(),
                mode.into(),
                uid.into(),
                gid.into(),
                last_access_time.into(),
                last_modification_time.into(),
                |id| {
                    on_overwritten.call(&FsBlobId(*id))?;
                    Ok(())
                },
            )
        })
        .into()
    }

    pub fn remove_entry_by_name(&mut self, name: &str) -> Box<FsResult> {
        log_errors(|| self.0.remove_entry_by_name(name)).into()
    }

    pub fn remove_entry_by_id_if_exists(&mut self, blob_id: &FsBlobId) {
        self.0.remove_entry_by_id_if_exists(&blob_id.0);
    }

    fn async_drop(&mut self) -> Result<()> {
        log_errors(|| TOKIO_RUNTIME.block_on(self.0.async_drop()))
    }
}

struct RustSymlinkBlobBridge<'a>(SymlinkBlob<'a, BlobStoreOnBlocks<DynBlockStore>>);

impl<'a> RustSymlinkBlobBridge<'a> {
    fn blob_id(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(self.0.blob_id()))
    }

    fn target(&mut self) -> Result<String> {
        // TODO Does self need to be mut?
        log_errors(|| {
            Ok(TOKIO_RUNTIME
                .block_on(self.0.target())?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Symlink target is not valid UTF-8"))?
                .to_string())
        })
    }

    fn parent(&self) -> Box<FsBlobId> {
        Box::new(FsBlobId(self.0.parent()))
    }
}

// TODO Do DynBlobStore instead of BlobStoreOnBlocks<DynBlockStore>? Having the dyn layer further outside means dynamic calls only when calling into the BlobStore, not when BlobStore calls into BlockStore, which is much more frequent.
struct RustFsBlobStoreBridge(AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>>);

impl RustFsBlobStoreBridge {
    fn create_file_blob<'a>(&'a self, parent: &FsBlobId) -> Result<Box<RustFileBlobBridge<'a>>> {
        log_errors(|| {
            let file = TOKIO_RUNTIME.block_on(self.0.create_file_blob(&parent.0))?;
            Ok(Box::new(RustFileBlobBridge(file)))
        })
    }

    fn create_dir_blob<'a>(&'a self, parent: &FsBlobId) -> Result<Box<RustDirBlobBridge<'a>>> {
        log_errors(|| {
            let dir = TOKIO_RUNTIME.block_on(self.0.create_dir_blob(&parent.0))?;
            Ok(Box::new(RustDirBlobBridge(dir)))
        })
    }

    fn create_symlink_blob<'a>(
        &'a self,
        parent: &FsBlobId,
        target: &str,
    ) -> Result<Box<RustSymlinkBlobBridge<'a>>> {
        log_errors(|| {
            let symlink = TOKIO_RUNTIME.block_on(self.0.create_symlink_blob(&parent.0, target))?;
            Ok(Box::new(RustSymlinkBlobBridge(symlink)))
        })
    }

    fn load<'a>(&'a self, blob_id: &FsBlobId) -> Result<Box<OptionRustFsBlobBridge<'a>>> {
        log_errors(|| {
            let blob = TOKIO_RUNTIME
                .block_on(self.0.load(&blob_id.0))?
                .map(RustFsBlobBridge::new);
            Ok(Box::new(OptionRustFsBlobBridge(blob)))
        })
    }

    fn num_blocks(&self) -> Result<u64> {
        log_errors(|| Ok(TOKIO_RUNTIME.block_on(self.0.num_blocks())?))
    }

    fn estimate_space_for_num_blocks_left(&self) -> Result<u64> {
        log_errors(|| self.0.estimate_space_for_num_blocks_left())
    }

    fn virtual_block_size_bytes(&self) -> u32 {
        self.0.virtual_block_size_bytes()
    }

    fn load_block_depth(&self, block_id: &FsBlobId) -> Result<u8> {
        log_errors(|| {
            TOKIO_RUNTIME.block_on(async {
                Ok(self
                    .0
                    .load_block_depth(&cryfs_blockstore::blockstore::BlockId::from_array(
                        &block_id.0.data(),
                    ))
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("Block not found"))?)
            })
        })
    }

    fn async_drop(&mut self) -> Result<()> {
        log_errors(|| TOKIO_RUNTIME.block_on(self.0.async_drop()))
    }
}

fn new_locking_integrity_encrypted_inmemory_fsblobstore(
    integrity_file_path: &str,
    my_client_id: u32,
    allow_integrity_violations: bool,
    missing_block_is_integrity_violation: bool,
    on_integrity_violation: cxx::UniquePtr<ffi::CxxCallback>,
    cipher_name: &str,
    encryption_key_hex: &str,
    block_size_bytes: u32,
) -> Result<Box<RustFsBlobStoreBridge>> {
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
        Ok(Box::new(RustFsBlobStoreBridge(FsBlobStore::new(
            BlobStoreOnBlocks::new(blockstore.extract(), block_size_bytes)?,
        ))))
    })
}

fn new_locking_integrity_encrypted_readonly_ondisk_fsblobstore(
    integrity_file_path: &str,
    my_client_id: u32,
    allow_integrity_violations: bool,
    missing_block_is_integrity_violation: bool,
    on_integrity_violation: UniquePtr<ffi::CxxCallback>,
    cipher_name: &str,
    encryption_key_hex: &str,
    basedir: &str,
    block_size_bytes: u32,
) -> Result<Box<RustFsBlobStoreBridge>> {
    LOGGER_INIT.ensure_initialized();
    let _init_tokio = TOKIO_RUNTIME.enter();

    log_errors(|| {
        let blockstore =
            super::blockstore::new_locking_integrity_encrypted_readonly_ondisk_blockstore(
                integrity_file_path,
                my_client_id,
                allow_integrity_violations,
                missing_block_is_integrity_violation,
                on_integrity_violation,
                cipher_name,
                encryption_key_hex,
                basedir,
            )?;
        Ok(Box::new(RustFsBlobStoreBridge(FsBlobStore::new(
            BlobStoreOnBlocks::new(blockstore.extract(), block_size_bytes)?,
        ))))
    })
}

fn new_locking_integrity_encrypted_ondisk_fsblobstore(
    integrity_file_path: &str,
    my_client_id: u32,
    allow_integrity_violations: bool,
    missing_block_is_integrity_violation: bool,
    on_integrity_violation: UniquePtr<ffi::CxxCallback>,
    cipher_name: &str,
    encryption_key_hex: &str,
    basedir: &str,
    block_size_bytes: u32,
) -> Result<Box<RustFsBlobStoreBridge>> {
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
        Ok(Box::new(RustFsBlobStoreBridge(FsBlobStore::new(
            BlobStoreOnBlocks::new(blockstore.extract(), block_size_bytes)?,
        ))))
    })
}
