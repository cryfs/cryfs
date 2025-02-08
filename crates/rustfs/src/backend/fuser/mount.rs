use std::fmt::Debug;
use std::path::Path;
use std::sync::{Arc, Mutex};
use fuser::MountOption;
use tokio_util::sync::CancellationToken;

use super::{backend_adapter::BackendAdapter, RunningFilesystem};
use crate::common::FsError;
use crate::low_level_api::{AsyncFilesystemLL, IntoFsLL};
use cryfs_utils::async_drop::AsyncDrop;

pub fn mount<Fs>(
    fs: impl IntoFsLL<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
    unmount_trigger: Option<CancellationToken>,
    mount_options: &[MountOption],
    on_successfully_mounted: impl FnOnce(),
) -> std::io::Result<()>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let fs = spawn_mount(fs, mountpoint, runtime, mount_options)?;
    on_successfully_mounted();

    if let Some(unmount_trigger) = unmount_trigger {
        fs.unmount_on_trigger(unmount_trigger);
    }

    fs.block_until_unmounted();
    Ok(())
}

pub fn spawn_mount<Fs>(
    fs: impl IntoFsLL<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
    mount_options: &[MountOption]
) -> std::io::Result<RunningFilesystem>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let backend = BackendAdapter::new(fs.into_fs(), runtime);

    let session = fuser::spawn_mount2(backend, mountpoint, mount_options)?;
    let session = Arc::new(Mutex::new(Some(session)));

    Ok(RunningFilesystem::new(session))
}
