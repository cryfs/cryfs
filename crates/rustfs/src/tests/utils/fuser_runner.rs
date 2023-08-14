use cryfs_utils::async_drop::AsyncDropGuard;
use tempdir::TempDir;

use super::filesystem_driver::FilesystemDriver;
use super::mock_low_level_api::MockAsyncFilesystemLL;
use crate::backend::fuser::{spawn_mount, RunningFilesystem};

pub struct Runner {
    mountpoint: TempDir,
    // TODO Better alternative to Option? e.g. Cell, RefCell?
    running_filesystem: Option<RunningFilesystem>,
}

impl Runner {
    pub fn start(implementation: MockAsyncFilesystemLL) -> Self {
        let runtime = tokio::runtime::Handle::current();
        let mountpoint = TempDir::new("rustfs-test-mock-mount").unwrap();
        let implementation = AsyncDropGuard::new(implementation);
        let running_filesystem = Some(
            spawn_mount(implementation, mountpoint.path(), runtime)
                .expect("Failed to spawn filesystem"),
        );
        Self {
            mountpoint,
            running_filesystem,
        }
    }

    pub fn driver(&self) -> FilesystemDriver {
        FilesystemDriver::new(self.mountpoint.path().to_owned())
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        // We need to drop running_filesystem before we drop TempDir, otherwise we deadlock.
        std::mem::drop(self.running_filesystem.take());
    }
}
