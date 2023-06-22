use std::fmt::Debug;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{backend_adapter::BackendAdapter, RunningFilesystem};
use crate::common::{FsError, Gid, Uid};
use crate::object_based_api::Device;
use cryfs_utils::async_drop::AsyncDrop;

pub fn mount<Fs>(
    fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<()>
where
    Fs: Device + Send + Sync + 'static,
    <Fs as Device>::Node: Send,
{
    let fs = spawn_mount(fs, mountpoint, runtime)?;
    fs.block_until_unmounted();
    Ok(())
}

pub fn spawn_mount<Fs>(
    fs: impl FnOnce(Uid, Gid) -> Fs + Send + Sync + 'static,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<RunningFilesystem>
where
    Fs: Device + Send + Sync + 'static,
    <Fs as Device>::Node: Send,
{
    let backend = BackendAdapter::new(fs, runtime);

    // TODO Fuse args (e.g. filesystem name)
    let session = fuser::spawn_mount2(backend, mountpoint, &[])?;
    let session = Arc::new(Mutex::new(Some(session)));

    Ok(RunningFilesystem::new(session))
}
