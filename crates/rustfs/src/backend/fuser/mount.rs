use std::fmt::Debug;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{backend_adapter::BackendAdapter, RunningFilesystem};
use crate::common::FsError;
use crate::low_level_api::{AsyncFilesystemLL, IntoFsLL};
use cryfs_utils::async_drop::AsyncDrop;

pub fn mount<Fs>(
    fs: impl IntoFsLL<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
    on_successfully_mounted: impl FnOnce(),
) -> std::io::Result<()>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let fs = spawn_mount(fs, mountpoint, runtime)?;
    on_successfully_mounted();
    fs.block_until_unmounted();
    Ok(())
}

pub fn spawn_mount<Fs>(
    fs: impl IntoFsLL<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<RunningFilesystem>
where
    Fs: AsyncFilesystemLL + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let backend = BackendAdapter::new(fs.into_fs(), runtime);

    // TODO Fuse args (e.g. filesystem name)
    let session = fuser::spawn_mount2(backend, mountpoint, &[])?;
    let session = Arc::new(Mutex::new(Some(session)));

    Ok(RunningFilesystem::new(session))
}
