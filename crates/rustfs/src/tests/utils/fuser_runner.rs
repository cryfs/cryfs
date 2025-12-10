use std::sync::OnceLock;
use tempdir::TempDir;

use cryfs_utils::async_drop::{AsyncDropArc, AsyncDropGuard, SyncDrop};

use super::filesystem_driver::FilesystemDriver;
use super::mock_low_level_api::MockAsyncFilesystemLL;
use crate::{
    backend::fuser::{RunningFilesystem, spawn_mount},
    tests::utils::mock_low_level_api::MockFilesystem,
};

pub struct Runner {
    // Order of members is important. We need to Drop `running_filesystem` before `mountpoint` and `implementation`.
    _running_filesystem: RunningFilesystem,
    mountpoint: TempDir,
    // We keep an Arc to the mock here so that it doesn't get dropped within the fuser thread.
    // If it got dropped within the fuser thread, the error may not correctly fail the test.
    // But if it gets dropped later in `Runner::drop`, then it's on the main thread and
    // correctly fails.
    _implementation: SyncDrop<AsyncDropArc<MockAsyncFilesystemLL>>,
}

impl Runner {
    pub async fn start(mock_fs: MockFilesystem) -> Self {
        LOG_INIT.get_or_init(|| {
            env_logger::builder()
                .filter_level(log::LevelFilter::Debug)
                .is_test(true)
                .try_init()
                .unwrap()
        });

        let implementation = SyncDrop::new(AsyncDropArc::new(AsyncDropGuard::new(mock_fs.fs)));

        let runtime = tokio::runtime::Handle::current();
        let mountpoint = TempDir::new("rustfs-test-mock-mount").unwrap();
        let running_filesystem = spawn_mount(
            AsyncDropArc::clone(implementation.inner()),
            mountpoint.path(),
            runtime,
            &[],
        )
        .await
        .expect("Failed to spawn filesystem");

        // Wait for the filesystem to be fully mounted (otherwise some tests may already destroy the runner before the filesystem is mounted,
        // causing flaky behavior where sometimes `init` is called and sometimes not)
        mock_fs.on_init_complete.wait().await;

        Self {
            _running_filesystem: running_filesystem,
            mountpoint,
            _implementation: implementation,
        }
    }

    pub fn driver(&self) -> FilesystemDriver {
        FilesystemDriver::new(self.mountpoint.path().to_owned().try_into().unwrap())
    }
}

static LOG_INIT: OnceLock<()> = OnceLock::new();

#[cfg(test)]
mod tests {
    use crate::common::FsError;
    use crate::tests::utils::{Runner, make_mock_filesystem};

    #[tokio::test]
    async fn setup_doesnt_panic() {
        // This test is here to demonstrate that basic setup of a file system works as expected.
        let mock_filesystem = make_mock_filesystem();
        let _runner = Runner::start(mock_filesystem).await;
    }

    #[tokio::test]
    #[should_panic(
        expected = "MockAsyncFilesystemLL::mkdir: Expectation(<anything>) called 0 time(s) which is fewer than expected 1"
    )]
    async fn mock_expectations_work_correctly() {
        // This test is here to demonstrate that the mock expectations work correctly.
        // This is necessary because file systems might run in other threads and failed mock
        // expectations might cause those threads to panic which would not fail the test.
        let mut mock_filesystem = make_mock_filesystem();
        mock_filesystem
            .fs
            .expect_mkdir()
            .once()
            .returning(|_, _, _, _, _| Err(FsError::NotImplemented));
        let _runner = Runner::start(mock_filesystem).await;
    }
}
