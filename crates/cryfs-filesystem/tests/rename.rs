//! Tests for renaming nodes, through both entry points the FUSE backends use:
//! - `Dir::rename_child` and `Dir::move_child_to` (fuser backend)
//! - `Device::rename` (fuse-mt backend)
//!
//! A rename that is rejected must not change the file system. In particular, a rejected
//! rename across two directories must not remove the source entry from its parent.

// The async state machines built here nest deeply enough that computing their layout
// overflows rustc's default recursion limit of 128.
#![recursion_limit = "512"]

use std::fmt::Debug;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use byte_unit::Byte;
use cryfs_blobstore::{BlobId, BlobStore, BlobStoreOnBlocks, RemoveResult};
use cryfs_blockstore::{InMemoryBlockStore, LockingBlockStore};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AtimeUpdateBehavior, FsError, FsResult, Gid, Mode, OpenInFlags, Uid,
    object_based_api::{Device, Dir, Node},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    path::AbsolutePath,
};

type TestDevice =
    CryDevice<FaultInjectingBlobStore<BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>>>;

/// Wraps a blob store and makes [BlobStore::remove_by_id] fail while `fail_removals` is set.
/// A rename that overwrites a node removes its blob, so this lets a test make the overwrite
/// fail after the rename already removed the entry from the source directory.
#[derive(Debug)]
struct FaultInjectingBlobStore<B>
where
    B: BlobStore + AsyncDrop + Debug + Send + Sync + 'static,
{
    underlying_store: AsyncDropGuard<B>,
    fail_removals: Arc<AtomicBool>,
}

impl<B> BlobStore for FaultInjectingBlobStore<B>
where
    B: BlobStore + AsyncDrop + Debug + Send + Sync + 'static,
{
    type ConcreteBlob = B::ConcreteBlob;

    async fn create(&self) -> anyhow::Result<AsyncDropGuard<Self::ConcreteBlob>> {
        self.underlying_store.create().await
    }

    async fn try_create(
        &self,
        id: &BlobId,
    ) -> anyhow::Result<Option<AsyncDropGuard<Self::ConcreteBlob>>> {
        self.underlying_store.try_create(id).await
    }

    async fn load(
        &self,
        id: &BlobId,
    ) -> anyhow::Result<Option<AsyncDropGuard<Self::ConcreteBlob>>> {
        self.underlying_store.load(id).await
    }

    async fn remove_by_id(&self, id: &BlobId) -> anyhow::Result<RemoveResult> {
        if self.fail_removals.load(Ordering::SeqCst) {
            anyhow::bail!("Injected error when removing blob {id:?}");
        }
        self.underlying_store.remove_by_id(id).await
    }

    async fn num_nodes(&self) -> anyhow::Result<u64> {
        self.underlying_store.num_nodes().await
    }

    fn estimate_space_for_num_blocks_left(&self) -> anyhow::Result<u64> {
        self.underlying_store.estimate_space_for_num_blocks_left()
    }

    fn logical_block_size_bytes(&self) -> Byte {
        self.underlying_store.logical_block_size_bytes()
    }

    async fn flush_if_cached(&self, blob_id: BlobId) -> anyhow::Result<()> {
        self.underlying_store.flush_if_cached(blob_id).await
    }

    async fn clear_cache_slow(&self) -> anyhow::Result<()> {
        self.underlying_store.clear_cache_slow().await
    }

    async fn clear_unloaded_blocks_from_cache(&self) -> anyhow::Result<()> {
        self.underlying_store
            .clear_unloaded_blocks_from_cache()
            .await
    }
}

impl<B> AsyncDrop for FaultInjectingBlobStore<B>
where
    B: BlobStore + AsyncDrop + Debug + Send + Sync + 'static,
{
    type Error = B::Error;

    async fn async_drop_impl(self) -> Result<(), B::Error> {
        let Self {
            underlying_store,
            fail_removals: _,
        } = self;
        underlying_store.async_drop().await
    }
}

/// Which entry point a rename goes through
#[derive(Debug, Clone, Copy)]
enum Api {
    /// `Dir::rename_child` within one directory, `Dir::move_child_to` across directories (fuser backend)
    Dir,
    /// `Device::rename` (fuse-mt backend)
    Device,
}

fn abspath(s: &str) -> &AbsolutePath {
    AbsolutePath::try_from_str(s).unwrap()
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

async fn make_device(fail_removals: &Arc<AtomicBool>) -> AsyncDropGuard<TestDevice> {
    let blockstore = LockingBlockStore::new(InMemoryBlockStore::new());
    let blobstore = BlobStoreOnBlocks::new(blockstore, 1024u32.into())
        .await
        .unwrap();
    let blobstore = AsyncDropGuard::new(FaultInjectingBlobStore {
        underlying_store: blobstore,
        fail_removals: Arc::clone(fail_removals),
    });
    CryDevice::create_new_filesystem(
        blobstore,
        BlobId::new_random(),
        AtimeUpdateBehavior::Relatime,
    )
    .await
    .unwrap()
}

/// Runs `test` on a fresh file system. Fails instead of hanging if `test` blocks forever.
async fn run(test: impl AsyncFnOnce(&TestDevice)) {
    run_with_fault_injection(async move |device, _fail_removals| test(device).await).await
}

/// Like [run], but also gives `test` a flag that makes removing blobs fail while it is set.
async fn run_with_fault_injection(test: impl AsyncFnOnce(&TestDevice, &AtomicBool)) {
    let fail_removals = Arc::new(AtomicBool::new(false));
    let device = make_device(&fail_removals).await;
    {
        let mut test = std::pin::pin!(test(&device, &fail_removals));
        if tokio::time::timeout(Duration::from_secs(60), &mut test)
            .await
            .is_err()
        {
            panic!("Test timed out");
        }
    }
    device.async_drop().await.unwrap();
}

async fn mkdir(device: &TestDevice, path: &str) {
    let (parent, name) = abspath(path).split_last().unwrap();
    let parent = device.lookup(parent).await.unwrap();
    {
        let parent = parent.as_dir().await.unwrap();
        let (_, dir) = parent
            .create_child_dir(name, dir_mode(), Uid::from(1000), Gid::from(1000))
            .await
            .unwrap();
        dir.async_drop().await.unwrap();
        parent.async_drop().await.unwrap();
    }
    parent.async_drop().await.unwrap();
}

async fn touch(device: &TestDevice, path: &str) {
    let (parent, name) = abspath(path).split_last().unwrap();
    let parent = device.lookup(parent).await.unwrap();
    {
        let parent = parent.as_dir().await.unwrap();
        let (_, node, openfile) = parent
            .create_and_open_file(
                name,
                file_mode(),
                Uid::from(1000),
                Gid::from(1000),
                OpenInFlags::ReadWrite,
            )
            .await
            .unwrap();
        openfile.async_drop().await.unwrap();
        node.async_drop().await.unwrap();
        parent.async_drop().await.unwrap();
    }
    parent.async_drop().await.unwrap();
}

/// Sorted entry names of the directory at `path`, or `None` if `path` is not a directory.
async fn ls(device: &TestDevice, path: &str) -> Option<Vec<String>> {
    let node = device.lookup(abspath(path)).await.unwrap();
    let names = match node.as_dir().await {
        Ok(dir) => {
            let entries = dir.entries().await.unwrap();
            dir.async_drop().await.unwrap();
            let mut names: Vec<String> = entries.iter().map(|e| e.name.to_string()).collect();
            names.sort();
            Some(names)
        }
        Err(FsError::NodeIsNotADirectory) => None,
        Err(err) => panic!("Unexpected error: {err:?}"),
    };
    node.async_drop().await.unwrap();
    names
}

async fn ls_dir(device: &TestDevice, path: &str) -> Vec<String> {
    ls(device, path).await.expect("Not a directory")
}

/// Number of blocks in the block store. If a rejected rename keeps every directory entry
/// and also keeps the number of blocks, it neither removed nor orphaned a blob.
async fn num_blocks(device: &TestDevice) -> u64 {
    let statfs = device.statfs().await.unwrap();
    statfs.num_total_blocks - statfs.num_free_blocks
}

async fn rename(device: &TestDevice, api: Api, from: &str, to: &str) -> FsResult<()> {
    match api {
        Api::Device => device.rename(abspath(from), abspath(to)).await,
        Api::Dir => {
            let (from_parent, from_name) = abspath(from).split_last().unwrap();
            let (to_parent, to_name) = abspath(to).split_last().unwrap();
            let from_parent_node = device.lookup(from_parent).await.unwrap();
            let from_parent_dir = from_parent_node.as_dir().await.unwrap();
            let result = if from_parent == to_parent {
                from_parent_dir.rename_child(from_name, to_name).await
            } else {
                let to_parent_node = device.lookup(to_parent).await.unwrap();
                let result = {
                    let to_parent_dir = to_parent_node.as_dir().await.unwrap();
                    from_parent_dir
                        .move_child_to(from_name, to_parent_dir, to_name)
                        .await
                };
                to_parent_node.async_drop().await.unwrap();
                result
            };
            from_parent_dir.async_drop().await.unwrap();
            from_parent_node.async_drop().await.unwrap();
            result
        }
    }
}

/// Runs a rename that must fail with `expected_error` and checks that it didn't change
/// any of the directories in `dirs`, nor the number of blocks.
async fn assert_rejected_without_changes(
    device: &TestDevice,
    api: Api,
    from: &str,
    to: &str,
    expected_error: FsError,
    dirs: &[&str],
) {
    let mut entries_before = Vec::new();
    for dir in dirs {
        entries_before.push(ls_dir(device, dir).await);
    }
    let blocks_before = num_blocks(device).await;

    let result = rename(device, api, from, to).await;
    // FsError doesn't implement PartialEq, so compare the Debug output
    assert_eq!(
        format!("{:?}", Err::<(), _>(expected_error)),
        format!("{result:?}"),
        "{api:?}: rename {from} -> {to}"
    );

    for (dir, entries_before) in dirs.iter().zip(entries_before) {
        assert_eq!(
            entries_before,
            ls_dir(device, dir).await,
            "{api:?}: rename {from} -> {to} was rejected but changed the entries of {dir}"
        );
    }
    assert_eq!(
        blocks_before,
        num_blocks(device).await,
        "{api:?}: rename {from} -> {to} was rejected but changed the number of blocks"
    );
}

// Renames across two directories

async fn across_dirs_dir_onto_nonempty_dir_is_rejected(api: Api) {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p2").await;
        mkdir(device, "/p1/src").await;
        touch(device, "/p1/src/src_file").await;
        mkdir(device, "/p2/dst").await;
        touch(device, "/p2/dst/dst_file").await;

        assert_rejected_without_changes(
            device,
            api,
            "/p1/src",
            "/p2/dst",
            FsError::CannotOverwriteNonEmptyDirectory,
            &["/", "/p1", "/p2", "/p1/src", "/p2/dst"],
        )
        .await;
    })
    .await
}

async fn across_dirs_file_onto_dir_is_rejected(api: Api) {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p2").await;
        touch(device, "/p1/src").await;
        mkdir(device, "/p2/dst").await;

        assert_rejected_without_changes(
            device,
            api,
            "/p1/src",
            "/p2/dst",
            FsError::CannotOverwriteDirectoryWithNonDirectory,
            &["/", "/p1", "/p2", "/p2/dst"],
        )
        .await;
    })
    .await
}

async fn across_dirs_dir_onto_file_is_rejected(api: Api) {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p2").await;
        mkdir(device, "/p1/src").await;
        touch(device, "/p2/dst").await;

        assert_rejected_without_changes(
            device,
            api,
            "/p1/src",
            "/p2/dst",
            FsError::CannotOverwriteNonDirectoryWithDirectory,
            &["/", "/p1", "/p2", "/p1/src"],
        )
        .await;
    })
    .await
}

/// Moving `/p1/src` onto its own parent `/p1` must fail with ENOTEMPTY, because `/p1` contains `src`.
async fn across_dirs_dir_onto_its_own_parent_is_rejected(api: Api) {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p1/src").await;

        assert_rejected_without_changes(
            device,
            api,
            "/p1/src",
            "/p1",
            FsError::CannotOverwriteNonEmptyDirectory,
            &["/", "/p1", "/p1/src"],
        )
        .await;
    })
    .await
}

/// Moving `/p1/p2/src` onto its grandparent `/p1` must fail with ENOTEMPTY, because `/p1` contains `p2`.
async fn across_dirs_dir_onto_its_grandparent_is_rejected(api: Api) {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p1/p2").await;
        mkdir(device, "/p1/p2/src").await;

        assert_rejected_without_changes(
            device,
            api,
            "/p1/p2/src",
            "/p1",
            FsError::CannotOverwriteNonEmptyDirectory,
            &["/", "/p1", "/p1/p2", "/p1/p2/src"],
        )
        .await;
    })
    .await
}

async fn across_dirs_dir_onto_empty_dir_succeeds(api: Api) {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p2").await;
        mkdir(device, "/p1/src").await;
        touch(device, "/p1/src/src_file").await;
        mkdir(device, "/p2/dst").await;
        let blocks_before = num_blocks(device).await;

        rename(device, api, "/p1/src", "/p2/dst").await.unwrap();

        assert_eq!(Vec::<String>::new(), ls_dir(device, "/p1").await);
        assert_eq!(vec!["dst"], ls_dir(device, "/p2").await);
        assert_eq!(vec!["src_file"], ls_dir(device, "/p2/dst").await);
        // Exactly the one block of the overwritten empty directory was freed
        assert_eq!(blocks_before - 1, num_blocks(device).await);
    })
    .await
}

async fn across_dirs_to_new_name_succeeds(api: Api) {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p2").await;
        mkdir(device, "/p1/src").await;
        touch(device, "/p1/src/src_file").await;
        let blocks_before = num_blocks(device).await;

        rename(device, api, "/p1/src", "/p2/dst").await.unwrap();

        assert_eq!(Vec::<String>::new(), ls_dir(device, "/p1").await);
        assert_eq!(vec!["dst"], ls_dir(device, "/p2").await);
        assert_eq!(vec!["src_file"], ls_dir(device, "/p2/dst").await);
        assert_eq!(blocks_before, num_blocks(device).await);
    })
    .await
}

/// If overwriting the destination fails after the source entry was already removed, the source
/// entry must be put back instead of getting lost. Here, removing the overwritten directory fails.
/// The destination being changed concurrently after the rename checked it ends up in the same place.
async fn across_dirs_failed_overwrite_puts_source_back(api: Api) {
    run_with_fault_injection(async |device, fail_removals| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p2").await;
        mkdir(device, "/p1/src").await;
        touch(device, "/p1/src/src_file").await;
        mkdir(device, "/p2/dst").await;
        let blocks_before = num_blocks(device).await;

        fail_removals.store(true, Ordering::SeqCst);
        let result = rename(device, api, "/p1/src", "/p2/dst").await;
        fail_removals.store(false, Ordering::SeqCst);

        assert!(
            result.is_err(),
            "{api:?}: expected an error, got {result:?}"
        );
        assert_eq!(
            vec!["src"],
            ls_dir(device, "/p1").await,
            "{api:?}: the failed rename lost the source entry"
        );
        assert_eq!(vec!["src_file"], ls_dir(device, "/p1/src").await);
        assert_eq!(vec!["dst"], ls_dir(device, "/p2").await);
        assert_eq!(Vec::<String>::new(), ls_dir(device, "/p2/dst").await);
        assert_eq!(blocks_before, num_blocks(device).await);

        // Nothing was left half done, so the same rename succeeds now that removing works again
        rename(device, api, "/p1/src", "/p2/dst").await.unwrap();
        assert_eq!(Vec::<String>::new(), ls_dir(device, "/p1").await);
        assert_eq!(vec!["dst"], ls_dir(device, "/p2").await);
        assert_eq!(vec!["src_file"], ls_dir(device, "/p2/dst").await);
    })
    .await
}

// Renames within one directory (https://github.com/cryfs/cryfs/issues/566)

async fn same_dir_dir_onto_nonempty_dir_is_rejected(api: Api) {
    run(async |device| {
        mkdir(device, "/dir1").await;
        touch(device, "/dir1/file").await;
        mkdir(device, "/dir2").await;

        assert_rejected_without_changes(
            device,
            api,
            "/dir2",
            "/dir1",
            FsError::CannotOverwriteNonEmptyDirectory,
            &["/", "/dir1", "/dir2"],
        )
        .await;
    })
    .await
}

async fn same_dir_dir_onto_empty_dir_succeeds(api: Api) {
    run(async |device| {
        mkdir(device, "/dir1").await;
        mkdir(device, "/dir2").await;
        touch(device, "/dir2/file").await;

        rename(device, api, "/dir2", "/dir1").await.unwrap();

        assert_eq!(vec!["dir1"], ls_dir(device, "/").await);
        assert_eq!(vec!["file"], ls_dir(device, "/dir1").await);
    })
    .await
}

macro_rules! tests_for_both_apis {
    ($($name:ident),* $(,)?) => {
        mod dir_api {
            $(
                #[tokio::test]
                async fn $name() {
                    super::$name(super::Api::Dir).await
                }
            )*
        }
        mod device_api {
            $(
                #[tokio::test]
                async fn $name() {
                    super::$name(super::Api::Device).await
                }
            )*
        }
    };
}

tests_for_both_apis!(
    across_dirs_dir_onto_nonempty_dir_is_rejected,
    across_dirs_file_onto_dir_is_rejected,
    across_dirs_dir_onto_file_is_rejected,
    across_dirs_dir_onto_its_own_parent_is_rejected,
    across_dirs_dir_onto_its_grandparent_is_rejected,
    across_dirs_dir_onto_empty_dir_succeeds,
    across_dirs_to_new_name_succeeds,
    across_dirs_failed_overwrite_puts_source_back,
    same_dir_dir_onto_nonempty_dir_is_rejected,
    same_dir_dir_onto_empty_dir_succeeds,
);

/// Only `Device::rename` takes paths, so only it can be asked to move a node into a file.
/// `Dir::move_child_to` takes the new parent as a `Dir`, so it can't be given a file.
#[tokio::test]
async fn device_api_across_dirs_into_a_file_is_rejected() {
    run(async |device| {
        mkdir(device, "/p1").await;
        mkdir(device, "/p1/src").await;
        touch(device, "/file").await;

        assert_rejected_without_changes(
            device,
            Api::Device,
            "/p1/src",
            "/file/dst",
            FsError::NodeIsNotADirectory,
            &["/", "/p1", "/p1/src"],
        )
        .await;
    })
    .await
}
