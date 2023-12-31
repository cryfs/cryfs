use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNodeStore};
use cryfs_blockstore::{
    AllowIntegrityViolations, DynBlockStore, InMemoryBlockStore, IntegrityConfig,
    LockingBlockStore, MissingBlockIsIntegrityViolation, SharedBlockStore,
};
use cryfs_check::CorruptedError;
use cryfs_cli_utils::setup_blockstore_stack_dyn;
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadResult, FixedPasswordProvider},
    filesystem::fsblobstore::{FsBlob, FsBlobStore},
    localstate::LocalStateDir,
};
use cryfs_utils::async_drop::{AsyncDropGuard, SyncDrop};
use futures::{future::BoxFuture, Future};
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use tempdir::TempDir;

use super::console::FixtureCreationConsole;
use super::entry_helpers::SomeBlobs;

const PASSWORD: &str = "mypassword";

pub struct FilesystemFixture {
    root_blob_id: BlobId,
    blockstore: SyncDrop<SharedBlockStore<InMemoryBlockStore>>,
    config: ConfigLoadResult,

    // tempdir should be in last position so it gets dropped last
    tempdir: FixtureTempDir,
}

impl FilesystemFixture {
    pub async fn new() -> Self {
        let tempdir = FixtureTempDir::new();
        let blockstore = SharedBlockStore::new(InMemoryBlockStore::new());
        let config = tempdir.create_config();
        let root_blob_id = BlobId::from_hex(&config.config.config().root_blob).unwrap();
        let result = Self {
            tempdir,
            blockstore: SyncDrop::new(blockstore),
            config,
            root_blob_id,
        };
        result.create_root_dir_blob().await;
        result
    }

    async fn create_root_dir_blob(&self) {
        let mut fsblobstore = self.make_blobstore().await;
        fsblobstore
            .create_root_dir_blob(&self.root_blob_id)
            .await
            .expect("Failed to create rootdir blob");
        fsblobstore.async_drop().await.unwrap();
    }

    async fn make_locking_blockstore(&self) -> AsyncDropGuard<LockingBlockStore<DynBlockStore>> {
        setup_blockstore_stack_dyn(
            SharedBlockStore::clone(&self.blockstore),
            &self.config,
            &self.tempdir.local_state_dir(),
            IntegrityConfig {
                allow_integrity_violations: AllowIntegrityViolations::DontAllowViolations,
                missing_block_is_integrity_violation:
                    MissingBlockIsIntegrityViolation::IsAViolation,
                on_integrity_violation: Box::new(|err| {
                    panic!("integrity violation");
                }),
            },
        )
        .await
        .expect("Failed to setup blockstore stack")
    }

    async fn make_blobstore(
        &self,
    ) -> AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>> {
        let blockstore = self.make_locking_blockstore().await;

        FsBlobStore::new(
            BlobStoreOnBlocks::new(
                blockstore,
                // TODO Change type in config instead of doing u32::try_from
                u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
            )
            .await
            .expect("Failed to create BlobStoreOnBlocks"),
        )
    }

    async fn make_nodestore(&self) -> AsyncDropGuard<DataNodeStore<DynBlockStore>> {
        let blockstore = self.make_locking_blockstore().await;

        DataNodeStore::new(
            blockstore,
            // TODO Change type in config instead of doing u32::try_from
            u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
        )
        .await
        .expect("Failed to create DataNodeStore")
    }

    pub async fn update_blockstore<'s, 'b, 'f, F, R>(
        &'s self,
        update_fn: impl FnOnce(&'b SharedBlockStore<InMemoryBlockStore>) -> F,
    ) -> R
    where
        F: 'f + Future<Output = R>,
        's: 'f + 'b,
        'b: 'f,
    {
        update_fn(&self.blockstore).await
    }

    pub async fn update_nodestore<R>(
        &self,
        update_fn: impl for<'b> FnOnce(&'b DataNodeStore<DynBlockStore>) -> BoxFuture<'b, R>,
    ) -> R {
        let mut nodestore = self.make_nodestore().await;
        let result = update_fn(&nodestore).await;
        nodestore.async_drop().await.unwrap();
        result
    }

    pub async fn update_fsblobstore<R>(
        &self,
        update_fn: impl for<'b> FnOnce(
            &'b FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>,
        ) -> BoxFuture<'b, R>,
    ) -> R {
        let mut fsblobstore = self.make_blobstore().await;
        let result = update_fn(&fsblobstore).await;
        fsblobstore.async_drop().await.unwrap();
        result
    }

    pub async fn run_cryfs_check(self) -> Vec<CorruptedError> {
        cryfs_check::check_filesystem(
            self.blockstore.into_inner_dont_drop(),
            &self.tempdir.config_file_path(),
            &self.tempdir.local_state_dir(),
            &FixedPasswordProvider::new(PASSWORD.to_owned()),
        )
        .await
        .expect("Failed to run cryfs-check")
    }

    pub fn root_blob_id(&self) -> BlobId {
        self.root_blob_id
    }

    pub async fn create_some_blobs(&self) -> SomeBlobs {
        let root_id = self.root_blob_id;
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let mut root = FsBlob::into_dir(blobstore.load(&root_id).await.unwrap().unwrap())
                    .await
                    .unwrap();
                let result = super::entry_helpers::create_some_blobs(blobstore, &mut root).await;
                root.async_drop().await.unwrap();
                result
            })
        })
        .await
    }

    pub async fn get_children_of_dir_blob(&self, dir_blob: BlobId) -> Vec<BlobId> {
        self.update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let blob = fsblobstore.load(&dir_blob).await.unwrap().unwrap();
                let mut blob = FsBlob::into_dir(blob).await.unwrap();
                let children = blob
                    .entries()
                    .map(|entry| *entry.blob_id())
                    .collect::<Vec<_>>();
                blob.async_drop().await.unwrap();
                children
            })
        })
        .await
    }
}

impl Debug for FilesystemFixture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilesystemFixture")
            .field("tempdir", &self.tempdir)
            .finish()
    }
}

#[derive(Debug)]
struct FixtureTempDir {
    tempdir: TempDir,
}

impl FixtureTempDir {
    pub fn new() -> Self {
        let tempdir = TempDir::new("cryfs-check-fixture").expect("Couldn't create tempdir");
        let result = Self { tempdir };
        std::fs::create_dir(result.local_state_dir_path())
            .expect("Failed to create local state dir");
        result
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.tempdir.path().join("cryfs.config")
    }

    pub fn local_state_dir_path(&self) -> PathBuf {
        self.tempdir.path().join("local_state_dir")
    }

    pub fn local_state_dir(&self) -> LocalStateDir {
        LocalStateDir::new(self.local_state_dir_path())
    }

    pub fn create_config(&self) -> ConfigLoadResult {
        cryfs_cryfs::config::create(
            self.config_file_path().to_owned(),
            &FixedPasswordProvider::new(PASSWORD.to_owned()),
            &FixtureCreationConsole,
            &CommandLineFlags {
                missing_block_is_integrity_violation: Some(false),
                expected_cipher: None,
            },
            &self.local_state_dir(),
        )
        .expect("Failed to create config")
    }
}
