use fuse_mt::FuseMT;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{fs_adapter::BackendAdapter, RunningFilesystem};
use crate::low_level_api::{AsyncFilesystem, IntoFs};

pub fn mount<Fs: AsyncFilesystem + Send + Sync + 'static>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
) -> std::io::Result<()> {
    let fs = spawn_mount(fs, mountpoint)?;
    fs.block_until_unmounted();
    Ok(())
}

pub fn spawn_mount<Fs: AsyncFilesystem + Send + Sync + 'static>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
) -> std::io::Result<RunningFilesystem> {
    let backend = BackendAdapter::new(fs.into_fs());
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
