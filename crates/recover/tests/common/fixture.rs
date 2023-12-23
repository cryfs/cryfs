use anyhow::Result;
use async_trait::async_trait;
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks};
use cryfs_blockstore::{
    tests::Fixture, AllowIntegrityViolations, BlockStore, InMemoryBlockStore, IntegrityConfig,
    LockingBlockStore, MissingBlockIsIntegrityViolation, SharedBlockStore,
};
use cryfs_cli_utils::setup_blockstore_stack_dyn;
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult, FixedPasswordProvider},
    filesystem::fsblobstore::FsBlobStore,
    localstate::LocalStateDir,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, SyncDrop};
use futures::Future;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use tempdir::TempDir;

use super::console::FixtureCreationConsole;

const PASSWORD: &str = "mypassword";

pub struct FilesystemFixture {
    root_blob_id: BlobId,
    blockstore: SyncDrop<SharedBlockStore<InMemoryBlockStore>>,
    fsblobstore: SyncDrop<FsBlobStore<BlobStoreOnBlocks<Box<dyn BlockStore + Send + Sync>>>>,

    // tempdir should be in last position so it gets dropped last
    tempdir: FixtureTempDir,
}

impl FilesystemFixture {
    pub async fn new() -> Self {
        let tempdir = FixtureTempDir::new();
        let blockstore = SharedBlockStore::new(InMemoryBlockStore::new());
        let config = tempdir.create_config();
        let root_blob_id = BlobId::from_hex(&config.config.config().root_blob).unwrap();
        let fsblobstore =
            Self::create_blobstore(&config, SharedBlockStore::clone(&blockstore), &tempdir).await;
        let result = Self {
            tempdir,
            blockstore: SyncDrop::new(blockstore),
            fsblobstore: SyncDrop::new(fsblobstore),
            root_blob_id,
        };
        result
    }

    async fn create_blobstore(
        config: &ConfigLoadResult,
        blockstore: AsyncDropGuard<SharedBlockStore<InMemoryBlockStore>>,
        tempdir: &FixtureTempDir,
    ) -> AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<Box<dyn BlockStore + Send + Sync>>>> {
        let blockstore = setup_blockstore_stack_dyn(
            blockstore,
            &config,
            &tempdir.local_state_dir(),
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
        .expect("Failed to setup blockstore stack");

        let blobstore = FsBlobStore::new(
            BlobStoreOnBlocks::new(
                blockstore,
                // TODO Change type in config instead of doing u32::try_from
                u32::try_from(config.config.config().blocksize_bytes).unwrap(),
            )
            .await
            .expect("Failed to create BlobStoreOnBlocks"),
        );

        let root_blob_id = BlobId::from_hex(&config.config.config().root_blob).unwrap();
        blobstore
            .create_root_dir_blob(&root_blob_id)
            .await
            .expect("Failed to create rootdir blob");
        blobstore
    }

    pub async fn update_blockstore<F>(
        &self,
        update_fn: impl FnOnce(&SharedBlockStore<InMemoryBlockStore>) -> F,
    ) where
        F: Future<Output = ()>,
    {
        self.fsblobstore
            .clear_cache_slow()
            .await
            .expect("Failed to clear cache");
        update_fn(&self.blockstore);
    }

    pub fn update_fsblobstore<F>(
        &self,
        update_fn: impl FnOnce(&FsBlobStore<BlobStoreOnBlocks<Box<dyn BlockStore + Send + Sync>>>) -> F,
    ) where
        F: Future<Output = ()>,
    {
        update_fn(&self.fsblobstore);
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
        let tempdir = TempDir::new("cryfs-recover-fixture").expect("Couldn't create tempdir");
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
