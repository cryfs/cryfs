use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{backend_adapter::BackendAdapter, RunningFilesystem};
use crate::low_level_api::AsyncFilesystemLL;

pub fn mount<Fs>(
    fs: Fs,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<()>
where
    Fs: AsyncFilesystemLL + Send + Sync + 'static,
{
    let fs = spawn_mount(fs, mountpoint, runtime)?;
    fs.block_until_unmounted();
    Ok(())
}

pub fn spawn_mount<Fs>(
    fs: Fs,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<RunningFilesystem>
where
    Fs: AsyncFilesystemLL + Send + Sync + 'static,
{
    let backend = BackendAdapter::new(fs, runtime);

    // TODO Fuse args (e.g. filesystem name)
    let session = fuser::spawn_mount2(backend, mountpoint, &[])?;
    let session = Arc::new(Mutex::new(Some(session)));

    Ok(RunningFilesystem::new(session))
}
