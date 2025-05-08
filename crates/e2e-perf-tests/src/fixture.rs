use byte_unit::Byte;
use derive_more::{Add, AddAssign, Sum};
use std::fmt::Debug;
use std::num::NonZeroU32;
use tempdir::TempDir;

use cryfs_blobstore::{BlobStore, BlobStoreActionCounts, BlobStoreOnBlocks, TrackingBlobStore};
use cryfs_blockstore::{
    BLOCKID_LEN, ClientId, DynBlockStore, HLActionCounts, HLSharedBlockStore, HLTrackingBlockStore,
    InMemoryBlockStore, IntegrityConfig, LLActionCounts, LLSharedBlockStore, LLTrackingBlockStore,
    LockingBlockStore,
};
use cryfs_cli_utils::setup_blockstore_stack_dyn;
use cryfs_filesystem::{
    CRYFS_VERSION,
    config::{CryConfig, FILESYSTEM_FORMAT_VERSION, FilesystemId},
    filesystem::CryDevice,
    localstate::LocalStateDir,
};
use cryfs_runner::{CreateOrLoad, make_device};
use cryfs_rustfs::{AtimeUpdateBehavior, Gid, RequestInfo, Uid};
use cryfs_utils::async_drop::{AsyncDropArc, AsyncDropGuard, SyncDrop};

use crate::filesystem_driver::FilesystemDriver;

const NUM_CHILDREN_PER_INNER_NODE: u64 = 20;
const BLOCKSIZE_BYTES: u64 = NUM_CHILDREN_PER_INNER_NODE * BLOCKID_LEN as u64;
pub const NUM_BYTES_FOR_THREE_LEVEL_TREE: u64 =
    2 * NUM_CHILDREN_PER_INNER_NODE as u64 * BLOCKSIZE_BYTES;

const MY_CLIENT_ID: NonZeroU32 = NonZeroU32::new(10).unwrap();

#[derive(Debug, Add, AddAssign, Sum, PartialEq, Eq, Clone, Copy)]
pub struct ActionCounts {
    pub blobstore: BlobStoreActionCounts,
    pub high_level: HLActionCounts,
    pub low_level: LLActionCounts,
}

pub struct FilesystemFixture<FS>
where
    FS: FilesystemDriver,
{
    // filesystem needs to be dropped before _local_state_tempdir, so it's declared first in the struct
    filesystem: SyncDrop<FS>,

    ll_blockstore: SyncDrop<LLSharedBlockStore<LLTrackingBlockStore<InMemoryBlockStore>>>,
    hl_blockstore:
        SyncDrop<HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>>,
    blobstore: SyncDrop<
        AsyncDropArc<
            TrackingBlobStore<
                BlobStoreOnBlocks<
                    HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                >,
            >,
        >,
    >,

    _local_state_tempdir: TempDir,
}

impl<FS> FilesystemFixture<FS>
where
    FS: FilesystemDriver,
{
    pub async fn create_filesystem(atime_behavior: AtimeUpdateBehavior) -> Self {
        let fixture = Self::create_uninitialized_filesystem(atime_behavior).await;
        fixture.filesystem.init().await.unwrap();
        fixture
    }

    pub async fn create_uninitialized_filesystem(atime_behavior: AtimeUpdateBehavior) -> Self {
        let ll_blockstore = Self::make_ll_blockstore().await;
        let (local_state_tempdir, hl_blockstore) = Self::make_hl_blockstore(&ll_blockstore).await;
        let blobstore = Self::make_blobstore(&hl_blockstore).await;

        let blobstore = AsyncDropArc::new(blobstore);
        let device = Self::make_device(&blobstore, atime_behavior).await;

        let filesystem = FS::new(device).await;

        Self {
            filesystem: SyncDrop::new(filesystem),
            ll_blockstore: SyncDrop::new(ll_blockstore),
            hl_blockstore: SyncDrop::new(hl_blockstore),
            blobstore: SyncDrop::new(blobstore),
            _local_state_tempdir: local_state_tempdir,
        }
    }

    async fn make_ll_blockstore()
    -> AsyncDropGuard<LLSharedBlockStore<LLTrackingBlockStore<InMemoryBlockStore>>> {
        let blockstore = InMemoryBlockStore::new();
        let blockstore = LLTrackingBlockStore::new(blockstore);
        let blockstore = LLSharedBlockStore::new(blockstore);
        blockstore
    }

    async fn make_hl_blockstore(
        ll_blockstore: &AsyncDropGuard<
            LLSharedBlockStore<LLTrackingBlockStore<InMemoryBlockStore>>,
        >,
    ) -> (
        TempDir,
        AsyncDropGuard<HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>>,
    ) {
        let local_state_tempdir = TempDir::new("cryfs-e2e-perf-tests").unwrap();

        let locking_blockstore = setup_blockstore_stack_dyn(
            LLSharedBlockStore::clone(&ll_blockstore),
            &config(),
            ClientId { id: MY_CLIENT_ID },
            &LocalStateDir::new(local_state_tempdir.path().to_owned()),
            IntegrityConfig {
                allow_integrity_violations:
                    cryfs_blockstore::AllowIntegrityViolations::DontAllowViolations,
                missing_block_is_integrity_violation:
                    cryfs_blockstore::MissingBlockIsIntegrityViolation::IsAViolation,
                on_integrity_violation: Box::new(move |err| {
                    panic!("Didn't expect integrity violations in test but got {err:?}");
                }),
            },
        )
        .await
        .unwrap();

        let tracking_block_store = HLTrackingBlockStore::new(locking_blockstore);
        let shared_block_store = HLSharedBlockStore::new(tracking_block_store);
        (local_state_tempdir, shared_block_store)
    }

    async fn make_blobstore(
        hl_blockstore: &AsyncDropGuard<
            HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
        >,
    ) -> AsyncDropGuard<
        TrackingBlobStore<
            BlobStoreOnBlocks<
                HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
            >,
        >,
    > {
        TrackingBlobStore::new(
            BlobStoreOnBlocks::new(HLSharedBlockStore::clone(hl_blockstore), config().blocksize)
                .await
                .unwrap(),
        )
    }

    async fn make_device(
        blobstore: &AsyncDropGuard<
            AsyncDropArc<
                TrackingBlobStore<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
        atime_behavior: AtimeUpdateBehavior,
    ) -> AsyncDropGuard<
        CryDevice<
            AsyncDropArc<
                TrackingBlobStore<
                    BlobStoreOnBlocks<
                        HLSharedBlockStore<HLTrackingBlockStore<LockingBlockStore<DynBlockStore>>>,
                    >,
                >,
            >,
        >,
    > {
        let blobstore = AsyncDropArc::clone(blobstore);

        let device = make_device(
            blobstore,
            &config(),
            CreateOrLoad::CreateNewFilesystem,
            atime_behavior,
        )
        .await
        .unwrap();

        device
    }

    pub fn totals(&self) -> ActionCounts {
        ActionCounts {
            blobstore: self.blobstore.counts(),
            high_level: self.hl_blockstore.counts(),
            low_level: self.ll_blockstore.counts(),
        }
    }

    pub async fn ops<R>(&self, operation: impl AsyncFnOnce(&FS) -> R) -> R {
        let result = operation(&self.filesystem).await;
        self.blobstore.clear_cache_slow().await.unwrap();
        result
    }

    pub async fn count_ops(&self, operation: impl AsyncFnOnce(&FS)) -> ActionCounts {
        self.blobstore.clear_cache_slow().await.unwrap();
        self.blobstore.get_and_reset_counts();
        self.hl_blockstore.get_and_reset_counts();
        self.ll_blockstore.get_and_reset_counts();
        operation(&self.filesystem).await;
        self.blobstore.clear_cache_slow().await.unwrap();
        ActionCounts {
            blobstore: self.blobstore.get_and_reset_counts(),
            high_level: self.hl_blockstore.get_and_reset_counts(),
            low_level: self.ll_blockstore.get_and_reset_counts(),
        }
    }
}

impl<FS> Drop for FilesystemFixture<FS>
where
    FS: FilesystemDriver,
{
    fn drop(&mut self) {
        futures::executor::block_on(self.filesystem.destroy());
    }
}

fn config() -> CryConfig {
    CryConfig {
        root_blob: "4a7a231be5055939468cb4a17087053e".to_string(),
        enc_key: "4e4f500b608039d5385f9f977f785288522c7f2f7e1af18a1974dce9c454720e".to_string(),
        cipher: "aes-256-gcm".to_string(),
        format_version: FILESYSTEM_FORMAT_VERSION.to_string(),
        created_with_version: CRYFS_VERSION.to_string(),
        last_opened_with_version: CRYFS_VERSION.to_string(),
        blocksize: Byte::from(BLOCKSIZE_BYTES),
        filesystem_id: FilesystemId::from_hex("8de43828c75c9bb10cac251eaf4ad9bd").unwrap(),
        exclusive_client_id: Some(MY_CLIENT_ID.get()),
    }
}

pub fn request_info() -> RequestInfo {
    RequestInfo {
        unique: 0,
        uid: Uid::from(0),
        gid: Gid::from(0),
        pid: 0,
    }
}
