use cryfs_blockstore::{
    tests::Fixture, AllowIntegrityViolations, InMemoryBlockStore, IntegrityConfig,
    MissingBlockIsIntegrityViolation, SharedBlockStore,
};
use cryfs_cli_utils::setup_blockstore_stack;
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult, FixedPasswordProvider},
    localstate::LocalStateDir,
};
use cryfs_utils::async_drop::{AsyncDropGuard, SyncDrop};
use std::path::PathBuf;
use tempdir::TempDir;

use super::console::FixtureCreationConsole;
use super::create_filesystem_runner::CreateFilesystemRunner;

const PASSWORD: &str = "mypassword";

pub struct FilesystemFixture {
    tempdir: FixtureTempDir,
    blockstore: SyncDrop<SharedBlockStore<InMemoryBlockStore>>,
}

impl FilesystemFixture {
    pub async fn new() -> Self {
        let tempdir = FixtureTempDir::new();
        let blockstore = SyncDrop::new(Self::create_filesystem(&tempdir).await);
        let result = Self {
            tempdir,
            blockstore,
        };
        result
    }

    async fn create_filesystem(
        tempdir: &FixtureTempDir,
    ) -> AsyncDropGuard<SharedBlockStore<InMemoryBlockStore>> {
        let config = tempdir.create_config();
        let blockstore = SharedBlockStore::new(InMemoryBlockStore::new());
        setup_blockstore_stack(
            SharedBlockStore::clone(&blockstore),
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
            CreateFilesystemRunner {
                config: &config,
            },
        )
        .await
        .expect("Failed to setup blockstore stack");
        blockstore
    }
}

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
