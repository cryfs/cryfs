use std::num::NonZeroU32;

use byte_unit::Byte;
use cryfs_blobstore::BlobStoreOnBlocks;
use cryfs_blockstore::{
    ActionCounts, ClientId, DynBlockStore, InMemoryBlockStore, IntegrityConfig, SharedBlockStore,
    TrackingBlockStore,
};
use cryfs_cli_utils::setup_blockstore_stack_dyn;
use cryfs_filesystem::{
    CRYFS_VERSION,
    config::{CryConfig, FILESYSTEM_FORMAT_VERSION, FilesystemId},
    filesystem::CryDevice,
    localstate::LocalStateDir,
};
use cryfs_runner::{CreateOrLoad, make_device};
use cryfs_rustfs::{
    AtimeUpdateBehavior, Gid, RequestInfo, Uid, low_level_api::AsyncFilesystemLL,
    object_based_api::ObjectBasedFsAdapterLL,
};
use cryfs_utils::async_drop::SyncDrop;
use tempdir::TempDir;

const BLOCKSIZE_BYTES: u64 = 4096;
const MY_CLIENT_ID: NonZeroU32 = NonZeroU32::new(10).unwrap();

pub struct FilesystemFixture {
    // filesystem needs to be dropped before _local_state_tempdir, so it's declared first in the struct
    filesystem: SyncDrop<ObjectBasedFsAdapterLL<CryDevice<BlobStoreOnBlocks<DynBlockStore>>>>,

    blockstore: SyncDrop<SharedBlockStore<TrackingBlockStore<InMemoryBlockStore>>>,

    _local_state_tempdir: TempDir,
}

impl FilesystemFixture {
    pub async fn create_filesystem(atime_behavior: AtimeUpdateBehavior) -> Self {
        let fixture = Self::create_uninitialized_filesystem(atime_behavior).await;
        fixture.filesystem.init(&request_info()).await.unwrap();
        fixture
    }

    pub async fn create_uninitialized_filesystem(atime_behavior: AtimeUpdateBehavior) -> Self {
        let blockstore = InMemoryBlockStore::new();
        let blockstore = TrackingBlockStore::new(blockstore);
        let blockstore = SharedBlockStore::new(blockstore);

        let local_state_tempdir = TempDir::new("cryfs-e2e-perf-tests").unwrap();

        let locking_blockstore = setup_blockstore_stack_dyn(
            SharedBlockStore::clone(&blockstore),
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

        let device = make_device(
            locking_blockstore,
            &config(),
            CreateOrLoad::CreateNewFilesystem,
            atime_behavior,
        )
        .await
        .unwrap();

        let device = |_uid, _gid| device;

        // TODO Test both low level and high level API (i.e. ObjectBasedFsAdapterLL for fuser and ObjectBasedFsAdapter for fuse_mt)
        let filesystem = ObjectBasedFsAdapterLL::new(device);

        Self {
            filesystem: SyncDrop::new(filesystem),
            blockstore: SyncDrop::new(blockstore),
            _local_state_tempdir: local_state_tempdir,
        }
    }

    pub fn totals(&self) -> ActionCounts {
        self.blockstore.totals()
    }

    pub async fn run_operation(
        &self,
        operation: impl AsyncFnOnce(
            &ObjectBasedFsAdapterLL<CryDevice<BlobStoreOnBlocks<DynBlockStore>>>,
        ),
    ) -> ActionCounts {
        self.blockstore.get_and_reset_totals();
        operation(&self.filesystem).await;
        self.blockstore.get_and_reset_totals()
    }
}

impl Drop for FilesystemFixture {
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
