use fuse_mt::FuseMT;
use std::fmt::Debug;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{backend_adapter::BackendAdapter, RunningFilesystem};
use crate::common::FsError;
use crate::low_level_api::{AsyncFilesystem, IntoFs};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

pub fn mount<Fs>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<()>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let fs = spawn_mount(fs, mountpoint, runtime)?;
    fs.block_until_unmounted();
    Ok(())
}

pub fn spawn_mount<Fs>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
    runtime: tokio::runtime::Handle,
) -> std::io::Result<RunningFilesystem>
where
    Fs: AsyncFilesystem + AsyncDrop<Error = FsError> + Debug + Send + Sync + 'static,
{
    let backend = BackendAdapter::new(fs.into_fs(), runtime);
    let fs = FuseMT::new(backend, num_threads());

    // TODO Fuse args (e.g. filesystem name)
    let session = fuse_mt::spawn_mount(fs, mountpoint, &[])?;
    let session = Arc::new(Mutex::new(Some(session)));

    Ok(RunningFilesystem::new(session))
}

fn num_threads() -> usize {
    std::thread::available_parallelism()
        .unwrap_or(NonZeroUsize::new(2).unwrap())
        .get()
}
