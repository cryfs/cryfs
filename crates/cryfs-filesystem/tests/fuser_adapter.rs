//! Tests that drive CryFS through rustfs's fuser adapter, [ObjectBasedFsAdapterLL], the way the kernel does,
//! but without mounting anything. Like for a mount, the adapter keeps every inode it handed out loaded until
//! it is forgotten, so nodes stay in use across operations.

// The async state machines built here nest deeply enough that computing their layout
// overflows rustc's default recursion limit of 128.
#![recursion_limit = "512"]

use std::time::Duration;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AtimeUpdateBehavior, FsError, Gid, InodeNumber, Mode, OpenInFlags, RequestInfo, Uid,
    low_level_api::AsyncFilesystemLL,
    object_based_api::{FUSE_ROOT_ID, ObjectBasedFsAdapterLL},
};
use cryfs_utils::{async_drop::AsyncDropGuard, path::PathComponent};

type Adapter =
    ObjectBasedFsAdapterLL<CryDevice<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>>;

fn req() -> RequestInfo {
    RequestInfo {
        unique: 0,
        uid: Uid::from(1000),
        gid: Gid::from(1000),
        pid: 0,
    }
}

fn name(name: &str) -> &PathComponent {
    PathComponent::try_from_str(name).unwrap()
}

fn dir_mode() -> Mode {
    Mode::default()
        .add_dir_flag()
        .add_user_read_flag()
        .add_user_write_flag()
        .add_user_exec_flag()
}

fn file_mode() -> Mode {
    Mode::default()
        .add_file_flag()
        .add_user_read_flag()
        .add_user_write_flag()
}

async fn make_adapter() -> AsyncDropGuard<Adapter> {
    let blockstore = LockingBlockStore::new(InMemoryBlockStore::new());
    let blobstore = BlobStoreOnBlocks::new(blockstore, 1024u32.into())
        .await
        .unwrap();
    let device = CryDevice::create_new_filesystem(
        blobstore,
        BlobId::new_random(),
        AtimeUpdateBehavior::Relatime,
    )
    .await
    .unwrap();
    let adapter = ObjectBasedFsAdapterLL::new(|_uid, _gid| device);
    adapter.init(&req()).await.unwrap();
    adapter
}

/// Runs `test` on a fresh file system. Fails instead of hanging if `test` blocks forever.
async fn run(test: impl AsyncFnOnce(&Adapter)) {
    let adapter = make_adapter().await;
    {
        let mut test = std::pin::pin!(test(&adapter));
        if tokio::time::timeout(Duration::from_secs(60), &mut test)
            .await
            .is_err()
        {
            panic!("Test timed out");
        }
    }
    adapter.async_drop().await.unwrap();
}

/// Creates a directory and returns its inode, which stays loaded like after a `mkdir` through a mount
async fn mkdir(fs: &Adapter, parent: InodeNumber, dir_name: &str) -> InodeNumber {
    fs.mkdir(&req(), parent, name(dir_name), dir_mode(), 0)
        .await
        .unwrap()
        .ino
        .handle
}

/// Creates a file and closes it again, and returns its inode, which stays loaded like after `touch` through a mount
async fn touch(fs: &Adapter, parent: InodeNumber, file_name: &str) -> InodeNumber {
    let reply = fs
        .create(
            &req(),
            parent,
            name(file_name),
            file_mode(),
            0,
            OpenInFlags::ReadWrite,
        )
        .await
        .unwrap();
    fs.release(
        &req(),
        reply.ino.handle,
        reply.fh,
        OpenInFlags::ReadWrite,
        None,
        false,
    )
    .await
    .unwrap();
    reply.ino.handle
}

async fn lookup(fs: &Adapter, parent: InodeNumber, child_name: &str) -> Option<InodeNumber> {
    match fs.lookup(&req(), parent, name(child_name)).await {
        Ok(reply) => Some(reply.ino.handle),
        Err(FsError::NodeDoesNotExist) => None,
        Err(err) => panic!("Unexpected error: {err:?}"),
    }
}

// Renames within one directory that overwrite a node the kernel still has loaded, e.g. an editor saving a
// file by writing a temporary file and renaming it over the original. The overwritten node stays loaded
// as an orphan until the kernel forgets it.

#[tokio::test]
async fn rename_within_dir_onto_loaded_file() {
    run(async |fs| {
        let source = touch(fs, FUSE_ROOT_ID, "source").await;
        let target = touch(fs, FUSE_ROOT_ID, "target").await;

        fs.rename(
            &req(),
            FUSE_ROOT_ID,
            name("source"),
            FUSE_ROOT_ID,
            name("target"),
            0,
        )
        .await
        .unwrap();

        // The kernel forgets the orphaned node once it doesn't need it anymore, and the moved node too
        fs.forget(&req(), target, 1).await.unwrap();
        fs.forget(&req(), source, 1).await.unwrap();
        assert_eq!(None, lookup(fs, FUSE_ROOT_ID, "source").await);
        assert!(lookup(fs, FUSE_ROOT_ID, "target").await.is_some());
    })
    .await
}

#[tokio::test]
async fn rename_within_dir_onto_loaded_empty_dir() {
    run(async |fs| {
        let source = mkdir(fs, FUSE_ROOT_ID, "source").await;
        let target = mkdir(fs, FUSE_ROOT_ID, "target").await;

        fs.rename(
            &req(),
            FUSE_ROOT_ID,
            name("source"),
            FUSE_ROOT_ID,
            name("target"),
            0,
        )
        .await
        .unwrap();

        // The kernel forgets the orphaned node once it doesn't need it anymore, and the moved node too
        fs.forget(&req(), target, 1).await.unwrap();
        fs.forget(&req(), source, 1).await.unwrap();
        assert_eq!(None, lookup(fs, FUSE_ROOT_ID, "source").await);
        assert!(lookup(fs, FUSE_ROOT_ID, "target").await.is_some());
    })
    .await
}
