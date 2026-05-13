use std::sync::OnceLock;
use tempfile::TempDir;

use cryfs_utils::async_drop::{AsyncDropArc, AsyncDropGuard, SyncDrop};
use cryfs_utils::safe_panic;

use super::filesystem_driver::FilesystemDriver;
use super::mock_low_level_api::MockAsyncFilesystemLL;
use crate::{
    backend::fuser::{Config, RunningFilesystem, spawn_mount},
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
        let mountpoint = tempfile::Builder::new()
            .prefix("rustfs-test-mock-mount")
            .tempdir()
            .unwrap();
        let running_filesystem = spawn_mount(
            AsyncDropArc::clone(implementation.inner()),
            mountpoint.path(),
            runtime,
            &Config::default(),
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

impl Drop for Runner {
    fn drop(&mut self) {
        // The fuser background thread blocks in `read()` on `/dev/fuse` until
        // the FUSE connection is aborted. When unprivileged, fuser's own drop
        // only does a lazy `umount2(MNT_DETACH)`, which defers that abort until
        // the kernel evicts the dcache entries pinned by our `LOOKUP`/`MKDIR`
        // calls — so the `join()` in `RunningFilesystem::drop` (the
        // `_running_filesystem` field, dropped right after this body) can hang
        // forever. Force the abort here, while we still hold the mountpoint, so
        // the join completes. This is deliberately a test-only concern:
        // production lets the kernel tear the connection down on its own and
        // must not force-abort in-flight requests on every unmount.
        #[cfg(target_os = "linux")]
        force_abort_fuse_connection(self.mountpoint.path());

        // Unmount explicitly and fail the test loudly on error. The unmount also
        // happens when the `_running_filesystem` field is dropped right after this
        // body, but `RunningFilesystem::Drop` only *logs* unmount/destroy failures
        // (correct for production, where aborting a user's process on a benign
        // unmount hiccup is worse than logging). In tests we want those failures —
        // including a panic in the background thread's `destroy()`, which fuser
        // surfaces as an `Err` from `unmount_join` — to fail the test. The
        // force-abort above ensures this `unmount_join` doesn't itself hang.
        // `safe_panic!` panics normally but degrades to stderr if we're already
        // unwinding (e.g. an assertion already failed), avoiding a double-panic
        // abort that would mask the original failure.
        if let Err(err) = self._running_filesystem.unmount_join() {
            safe_panic!("Test filesystem unmount failed: {err}");
        }
    }
}

/// Writes `"1"` to `/sys/fs/fuse/connections/<id>/abort`, which calls
/// `fuse_abort_conn` directly and makes the background thread's blocked
/// `read()` return regardless of dcache state. Best-effort: a missing control
/// file means the kernel already tore the connection down on its own.
#[cfg(target_os = "linux")]
fn force_abort_fuse_connection(mountpoint: &std::path::Path) {
    let Some(id) = fuse_connection_id_for_mountpoint(mountpoint) else {
        return;
    };
    let path = format!("/sys/fs/fuse/connections/{id}/abort");
    if let Err(err) = std::fs::write(&path, b"1") {
        if err.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Failed to write FUSE abort file {path}: {err}");
        }
    }
}

/// Resolves a FUSE mountpoint to its connection id (= `minor(st_dev)`) via
/// `/proc/self/mountinfo`. Reading mountinfo avoids `stat`-ing the mountpoint,
/// which would issue a `GETATTR` the mock filesystem doesn't expect.
#[cfg(target_os = "linux")]
fn fuse_connection_id_for_mountpoint(mountpoint: &std::path::Path) -> Option<u32> {
    // mountinfo stores kernel-canonicalized absolute paths. Canonicalize the
    // *parent* and re-join the final component so a symlinked `$TMPDIR` still
    // matches, without `stat`-ing the mountpoint itself.
    let mountpoint = match (mountpoint.parent(), mountpoint.file_name()) {
        (Some(parent), Some(name)) => std::fs::canonicalize(parent)
            .map(|p| p.join(name))
            .unwrap_or_else(|_| mountpoint.to_path_buf()),
        _ => mountpoint.to_path_buf(),
    };
    let mountpoint_str = mountpoint.to_str()?;
    let mountinfo = std::fs::read_to_string("/proc/self/mountinfo").ok()?;
    for line in mountinfo.lines() {
        // Format: mount_id parent_id major:minor root mountpoint mount_opts ...
        let mut fields = line.split_whitespace();
        let _mount_id = fields.next()?;
        let _parent_id = fields.next()?;
        let major_minor = fields.next()?;
        let _root = fields.next()?;
        let mp = fields.next()?;
        if mp == mountpoint_str {
            let (_major, minor) = major_minor.split_once(':')?;
            return minor.parse().ok();
        }
    }
    log::warn!(
        "Could not resolve FUSE connection id for mountpoint {mountpoint_str} \
         in /proc/self/mountinfo; teardown will fall back to fuser's lazy \
         unmount, which may hang"
    );
    None
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
