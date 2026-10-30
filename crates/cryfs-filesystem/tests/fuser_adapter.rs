//! Tests that drive CryFS through rustfs's fuser adapter, [ObjectBasedFsAdapterLL], the way the kernel does,
//! but without mounting anything. Like for a mount, the adapter keeps every inode it handed out loaded until
//! it is forgotten, so nodes stay in use across operations.
//!
//! Some tests also run requests concurrently that the kernel wouldn't, like callers in this process can.
//! They poll the requests by hand and pause a flush in the blob store, see [FlushPause], so that the
//! requests interleave the same way every time.

// The async state machines built here nest deeply enough that computing their layout
// overflows rustc's default recursion limit of 128.
#![recursion_limit = "512"]

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use byte_unit::Byte;
use cryfs_blobstore::{Blob, BlobId, BlobStore, BlobStoreOnBlocks, RemoveResult};
use cryfs_blockstore::{BlockId, InMemoryBlockStore, LockingBlockStore};
use cryfs_filesystem::filesystem::CryDevice;
use cryfs_rustfs::{
    AtimeUpdateBehavior, FsError, Gid, InodeNumber, Mode, NodeAttrs, NumBytes, OpenInFlags,
    RequestInfo, Uid,
    low_level_api::AsyncFilesystemLL,
    object_based_api::{FUSE_ROOT_ID, ObjectBasedFsAdapterLL},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    data::Data,
    path::PathComponent,
};
use futures::{FutureExt as _, stream::BoxStream};
use tokio::sync::Notify;

type InMemoryBlobStore = BlobStoreOnBlocks<LockingBlockStore<InMemoryBlockStore>>;
type Adapter = ObjectBasedFsAdapterLL<CryDevice<PausingBlobStore>>;

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

fn read_only_file_mode() -> Mode {
    Mode::default().add_file_flag().add_user_read_flag()
}

async fn make_adapter() -> (AsyncDropGuard<Adapter>, Arc<FlushPause>) {
    let blockstore = LockingBlockStore::new(InMemoryBlockStore::new());
    let blobstore = BlobStoreOnBlocks::new(blockstore, 1024u32.into())
        .await
        .unwrap();
    let pause = Arc::new(FlushPause::default());
    let blobstore = AsyncDropGuard::new(PausingBlobStore {
        underlying_store: blobstore,
        pause: Arc::clone(&pause),
    });
    let device = CryDevice::create_new_filesystem(
        blobstore,
        BlobId::new_random(),
        AtimeUpdateBehavior::Relatime,
    )
    .await
    .unwrap();
    let adapter = ObjectBasedFsAdapterLL::new(|_uid, _gid| device);
    adapter.init(&req()).await.unwrap();
    (adapter, pause)
}

/// Runs `test` on a fresh file system. Fails instead of hanging if `test` blocks forever.
async fn run(test: impl AsyncFnOnce(&Adapter)) {
    run_with_pause(async |fs, _pause| test(fs).await).await
}

/// Like [run], for tests that pause a flush
async fn run_with_pause(test: impl AsyncFnOnce(&Adapter, &FlushPause)) {
    let (adapter, pause) = make_adapter().await;
    {
        let mut test = std::pin::pin!(test(&adapter, &pause));
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

async fn getattr(fs: &Adapter, ino: InodeNumber) -> NodeAttrs {
    fs.getattr(&req(), ino, None).await.unwrap().attr
}

async fn chmod(fs: &Adapter, ino: InodeNumber, mode: Mode) {
    fs.setattr(
        &req(),
        ino,
        Some(mode),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap();
}

// A rename within one directory changes the name of a node the kernel still has loaded.

/// Through a mount, `touch file; mv file renamed; touch file; chmod 400 renamed` used to change the mode of the
/// new `file` instead of `renamed`, because the node looked up its entry by the name it had when it was loaded.
#[tokio::test]
async fn setattr_after_rename_within_dir_changes_the_renamed_node() {
    run(async |fs| {
        let renamed = touch(fs, FUSE_ROOT_ID, "file").await;
        fs.rename(
            &req(),
            FUSE_ROOT_ID,
            name("file"),
            FUSE_ROOT_ID,
            name("renamed"),
            0,
        )
        .await
        .unwrap();
        let new_file = touch(fs, FUSE_ROOT_ID, "file").await;

        chmod(fs, renamed, read_only_file_mode()).await;

        assert_eq!(read_only_file_mode(), getattr(fs, renamed).await.mode);
        assert_eq!(file_mode(), getattr(fs, new_file).await.mode);
    })
    .await
}

// A rename across directories moves a node the kernel still has loaded to a different directory.

/// Through a mount, `echo hello > a/f; mv a/f b/f; cat b/f` used to fail with ENOENT, because the node looked
/// for its entry in the directory it was loaded from. https://github.com/cryfs/cryfs/issues/565 is the same
/// bug in the C++ implementation.
#[tokio::test]
async fn setattr_after_rename_across_dirs() {
    run(async |fs| {
        let a = mkdir(fs, FUSE_ROOT_ID, "a").await;
        let b = mkdir(fs, FUSE_ROOT_ID, "b").await;
        let file = touch(fs, a, "file").await;

        fs.rename(&req(), a, name("file"), b, name("file"), 0)
            .await
            .unwrap();

        chmod(fs, file, read_only_file_mode()).await;
        assert_eq!(read_only_file_mode(), getattr(fs, file).await.mode);
        assert_eq!(Some(file), lookup(fs, b, "file").await);
    })
    .await
}

#[tokio::test]
async fn write_to_open_file_after_rename_across_dirs() {
    run(async |fs| {
        let a = mkdir(fs, FUSE_ROOT_ID, "a").await;
        let b = mkdir(fs, FUSE_ROOT_ID, "b").await;
        let file = fs
            .create(
                &req(),
                a,
                name("file"),
                file_mode(),
                0,
                OpenInFlags::ReadWrite,
            )
            .await
            .unwrap();

        fs.rename(&req(), a, name("file"), b, name("file"), 0)
            .await
            .unwrap();

        // Writing updates the modification time in the file's entry
        fs.write(
            &req(),
            file.ino.handle,
            file.fh,
            NumBytes::from(0),
            b"hello".to_vec(),
            0,
            0,
            None,
        )
        .await
        .unwrap();
        assert_eq!(
            NumBytes::from(5),
            getattr(fs, file.ino.handle).await.num_bytes
        );
        fs.release(
            &req(),
            file.ino.handle,
            file.fh,
            OpenInFlags::ReadWrite,
            None,
            true,
        )
        .await
        .unwrap();
    })
    .await
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

        assert_eq!(None, lookup(fs, FUSE_ROOT_ID, "source").await);
        assert_eq!(Some(source), lookup(fs, FUSE_ROOT_ID, "target").await);
        // The kernel forgets the orphaned node once it doesn't need it anymore. That must not take the
        // node that replaced it out of the inode list.
        fs.forget(&req(), target, 1).await.unwrap();
        assert_eq!(Some(source), lookup(fs, FUSE_ROOT_ID, "target").await);
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

        assert_eq!(None, lookup(fs, FUSE_ROOT_ID, "source").await);
        assert_eq!(Some(source), lookup(fs, FUSE_ROOT_ID, "target").await);
        // The kernel forgets the orphaned node once it doesn't need it anymore. That must not take the
        // node that replaced it out of the inode list.
        fs.forget(&req(), target, 1).await.unwrap();
        assert_eq!(Some(source), lookup(fs, FUSE_ROOT_ID, "target").await);
    })
    .await
}

// Removing a directory waits until nobody has its blob loaded anymore. A node the kernel still has loaded must not
// keep the directory it was moved out of or removed from loaded, otherwise removing that directory blocks until
// the kernel forgets the node.

/// Fails the test if the directory can't be removed, or if removing it blocks
async fn rmdir(fs: &Adapter, parent: InodeNumber, dir_name: &str) {
    tokio::time::timeout(
        Duration::from_secs(10),
        fs.rmdir(&req(), parent, name(dir_name)),
    )
    .await
    .expect("rmdir blocked")
    .unwrap();
}

/// Through a mount, `mkdir -p a/x b; mv a/x b/x; rmdir a` blocked until the kernel forgot `x`
#[tokio::test]
async fn rmdir_after_moving_a_loaded_dir_out() {
    run(async |fs| {
        let a = mkdir(fs, FUSE_ROOT_ID, "a").await;
        let x = mkdir(fs, a, "x").await;
        let b = mkdir(fs, FUSE_ROOT_ID, "b").await;
        fs.rename(&req(), a, name("x"), b, name("x"), 0)
            .await
            .unwrap();

        rmdir(fs, FUSE_ROOT_ID, "a").await;

        // The moved node finds its new parent when it is used again
        assert_eq!(dir_mode(), getattr(fs, x).await.mode);
    })
    .await
}

#[tokio::test]
async fn rmdir_after_moving_an_open_file_out() {
    run(async |fs| {
        let a = mkdir(fs, FUSE_ROOT_ID, "a").await;
        let b = mkdir(fs, FUSE_ROOT_ID, "b").await;
        let file = fs
            .create(
                &req(),
                a,
                name("file"),
                file_mode(),
                0,
                OpenInFlags::ReadWrite,
            )
            .await
            .unwrap();
        fs.rename(&req(), a, name("file"), b, name("file"), 0)
            .await
            .unwrap();

        rmdir(fs, FUSE_ROOT_ID, "a").await;

        fs.write(
            &req(),
            file.ino.handle,
            file.fh,
            NumBytes::from(0),
            b"hello".to_vec(),
            0,
            0,
            None,
        )
        .await
        .unwrap();
        fs.release(
            &req(),
            file.ino.handle,
            file.fh,
            OpenInFlags::ReadWrite,
            None,
            true,
        )
        .await
        .unwrap();
    })
    .await
}

#[tokio::test]
async fn rmdir_after_removing_an_open_file() {
    run(async |fs| {
        let dir = mkdir(fs, FUSE_ROOT_ID, "dir").await;
        let file = fs
            .create(
                &req(),
                dir,
                name("file"),
                file_mode(),
                0,
                OpenInFlags::ReadWrite,
            )
            .await
            .unwrap();
        fs.unlink(&req(), dir, name("file")).await.unwrap();

        rmdir(fs, FUSE_ROOT_ID, "dir").await;

        // Don't flush, the file's blob is already gone
        fs.release(
            &req(),
            file.ino.handle,
            file.fh,
            OpenInFlags::ReadWrite,
            None,
            false,
        )
        .await
        .unwrap();
    })
    .await
}

/// The node the rename overwrote stays loaded as an orphan
#[tokio::test]
async fn rmdir_after_a_rename_overwrote_a_loaded_node() {
    run(async |fs| {
        let dir = mkdir(fs, FUSE_ROOT_ID, "dir").await;
        touch(fs, dir, "a").await;
        touch(fs, dir, "b").await;
        fs.rename(&req(), dir, name("a"), dir, name("b"), 0)
            .await
            .unwrap();
        fs.unlink(&req(), dir, name("b")).await.unwrap();

        rmdir(fs, FUSE_ROOT_ID, "dir").await;
    })
    .await
}

// While the kernel removes a name from a directory, it only looks up names there to revalidate inodes it already
// has, which are loaded, but callers in this process can look up any name. Then a lookup can still be loading a
// node when the removal tells the node that it was removed.

/// Starts `request` without polling it, so that the test can interleave it with other requests by hand.
/// Without [tokio::task::unconstrained], tokio's cooperative scheduling could make it pending even though
/// it doesn't wait for anything.
fn start<F: Future>(request: F) -> Pin<Box<impl Future<Output = F::Output>>> {
    Box::pin(tokio::task::unconstrained(request))
}

/// Polls `request` once and returns its result if it finished. Nothing wakes it up, the test decides when
/// to poll it again.
fn poll_once<T>(request: Pin<&mut impl Future<Output = T>>) -> Option<T> {
    request.now_or_never()
}

#[derive(Clone, Copy)]
enum Removal {
    UnlinkFile,
    RmdirDir,
}

/// The removal of P/x tells x that it was removed while a lookup of P/x is still loading x. This used to panic
/// with "Forgot to call async_drop on AsyncDropArc { v: AsyncDropGuard(Some(ConcurrentStoreInner)) }".
async fn remove_while_a_lookup_is_still_loading(removal: Removal) {
    run_with_pause(async |fs, pause| {
        let req = req();
        let p = mkdir(fs, FUSE_ROOT_ID, "P").await;
        let p_blob = pause.last_created_blob();
        let x = match removal {
            Removal::UnlinkFile => touch(fs, p, "x").await,
            Removal::RmdirDir => mkdir(fs, p, "x").await,
        };
        // So that the lookup below has to load x
        fs.forget(&req, x, 1).await.unwrap();

        // The removal pauses while it holds P's lock, after it removed the entry of x
        pause.pause_next_flush_of(p_blob);
        let mut remove = start(async {
            match removal {
                Removal::UnlinkFile => fs.unlink(&req, p, name("x")).await,
                Removal::RmdirDir => fs.rmdir(&req, p, name("x")).await,
            }
        });
        assert!(poll_once(remove.as_mut()).is_none());
        assert!(pause.flush_is_paused());

        // A getattr of P waits for P's lock, and so does the lookup of x, which already added x to the inode list
        let mut getattr = start(fs.getattr(&req, p, None));
        assert!(poll_once(getattr.as_mut()).is_none());
        let mut lookup = start(fs.lookup(&req, p, name("x")));
        assert!(poll_once(lookup.as_mut()).is_none());

        // The removal releases P's lock, which goes to the getattr, and we don't poll that yet. So the lookup is still
        // loading x when the removal tells x that it was removed, and the removal waits until the lookup loaded x.
        pause.release_flush();
        assert!(poll_once(remove.as_mut()).is_none());

        let (remove, getattr, lookup) = futures::join!(remove, getattr, lookup);
        remove.unwrap();
        getattr.unwrap();
        // The lookup doesn't find x anymore once it gets P's lock
        assert!(
            matches!(lookup, Err(FsError::NodeDoesNotExist)),
            "lookup returned {lookup:?}"
        );
        // Nothing keeps P loaded, so removing it doesn't block
        rmdir(fs, FUSE_ROOT_ID, "P").await;
    })
    .await
}

#[tokio::test]
async fn unlink_while_a_lookup_is_still_loading_the_file() {
    remove_while_a_lookup_is_still_loading(Removal::UnlinkFile).await
}

#[tokio::test]
async fn rmdir_while_a_lookup_is_still_loading_the_dir() {
    remove_while_a_lookup_is_still_loading(Removal::RmdirDir).await
}

/// Lets a test pause the next flush of a chosen blob until it releases it, see [PausingBlobStore]
#[derive(Debug, Default)]
struct FlushPause {
    last_created_blob: Mutex<Option<BlobId>>,
    blob_to_pause: Mutex<Option<BlobId>>,
    paused: AtomicBool,
    released: Notify,
}

impl FlushPause {
    /// The blob that the file system created last, e.g. for the directory that `mkdir` just created
    fn last_created_blob(&self) -> BlobId {
        self.last_created_blob
            .lock()
            .unwrap()
            .expect("No blob was created yet")
    }

    /// The next flush of `blob_id` waits until [Self::release_flush] is called
    fn pause_next_flush_of(&self, blob_id: BlobId) {
        *self.blob_to_pause.lock().unwrap() = Some(blob_id);
    }

    fn flush_is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    fn release_flush(&self) {
        self.released.notify_one();
    }

    async fn maybe_pause_flush_of(&self, blob_id: BlobId) {
        let pause = self
            .blob_to_pause
            .lock()
            .unwrap()
            .take_if(|blob_to_pause| *blob_to_pause == blob_id)
            .is_some();
        if pause {
            self.paused.store(true, Ordering::SeqCst);
            self.released.notified().await;
            self.paused.store(false, Ordering::SeqCst);
        }
    }
}

/// The in-memory blob store, with blobs whose flush can be paused, see [FlushPause]
#[derive(Debug)]
struct PausingBlobStore {
    underlying_store: AsyncDropGuard<InMemoryBlobStore>,
    pause: Arc<FlushPause>,
}

impl PausingBlobStore {
    fn wrap(
        &self,
        blob: AsyncDropGuard<<InMemoryBlobStore as BlobStore>::ConcreteBlob>,
    ) -> AsyncDropGuard<PausingBlob> {
        AsyncDropGuard::new(PausingBlob {
            blob,
            pause: Arc::clone(&self.pause),
        })
    }
}

impl BlobStore for PausingBlobStore {
    type ConcreteBlob = PausingBlob;

    async fn create(&self) -> anyhow::Result<AsyncDropGuard<PausingBlob>> {
        let blob = self.underlying_store.create().await?;
        *self.pause.last_created_blob.lock().unwrap() = Some(blob.id());
        Ok(self.wrap(blob))
    }

    async fn try_create(&self, id: &BlobId) -> anyhow::Result<Option<AsyncDropGuard<PausingBlob>>> {
        Ok(self
            .underlying_store
            .try_create(id)
            .await?
            .map(|blob| self.wrap(blob)))
    }

    async fn load(&self, id: &BlobId) -> anyhow::Result<Option<AsyncDropGuard<PausingBlob>>> {
        Ok(self
            .underlying_store
            .load(id)
            .await?
            .map(|blob| self.wrap(blob)))
    }

    async fn remove_by_id(&self, id: &BlobId) -> anyhow::Result<RemoveResult> {
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

impl AsyncDrop for PausingBlobStore {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> anyhow::Result<()> {
        let Self {
            underlying_store,
            pause: _,
        } = self;
        underlying_store.async_drop().await
    }
}

/// A blob of [PausingBlobStore]
#[derive(Debug)]
struct PausingBlob {
    blob: AsyncDropGuard<<InMemoryBlobStore as BlobStore>::ConcreteBlob>,
    pause: Arc<FlushPause>,
}

impl Blob for PausingBlob {
    fn id(&self) -> BlobId {
        self.blob.id()
    }

    async fn num_bytes(&mut self) -> anyhow::Result<u64> {
        self.blob.num_bytes().await
    }

    async fn resize(&mut self, new_num_bytes: u64) -> anyhow::Result<()> {
        self.blob.resize(new_num_bytes).await
    }

    async fn read_all(&mut self) -> anyhow::Result<Data> {
        self.blob.read_all().await
    }

    async fn read(&mut self, target: &mut [u8], offset: u64) -> anyhow::Result<()> {
        self.blob.read(target, offset).await
    }

    async fn try_read(&mut self, target: &mut [u8], offset: u64) -> anyhow::Result<usize> {
        self.blob.try_read(target, offset).await
    }

    async fn write(&mut self, source: &[u8], offset: u64) -> anyhow::Result<()> {
        self.blob.write(source, offset).await
    }

    async fn flush(&mut self) -> anyhow::Result<()> {
        self.pause.maybe_pause_flush_of(self.id()).await;
        self.blob.flush().await
    }

    async fn num_nodes(&mut self) -> anyhow::Result<u64> {
        self.blob.num_nodes().await
    }

    async fn remove(this: AsyncDropGuard<Self>) -> anyhow::Result<()> {
        let Self { blob, pause: _ } = this.unsafe_into_inner_dont_drop();
        Blob::remove(blob).await
    }

    fn all_blocks(&self) -> anyhow::Result<BoxStream<'_, anyhow::Result<BlockId>>> {
        self.blob.all_blocks()
    }
}

impl AsyncDrop for PausingBlob {
    type Error = anyhow::Error;

    async fn async_drop_impl(self) -> anyhow::Result<()> {
        let Self { blob, pause: _ } = self;
        blob.async_drop().await
    }
}
