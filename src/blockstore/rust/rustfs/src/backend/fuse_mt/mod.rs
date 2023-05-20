//! This module allows running a file system using the [fuse-mt] library.

use fuse_mt::FuseMT;
use std::path::Path;

mod running_filesystem;
pub use running_filesystem::RunningFilesystem;

mod fs_adapter;
use fs_adapter::BackendAdapter;

use crate::low_level_api::{AsyncFilesystem, IntoFs};

pub fn mount<Fs: AsyncFilesystem + Send + Sync + 'static>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
) -> std::io::Result<()> {
    // TODO Ctrl+C doesn't do a clean unmount
    // TODO Num threads
    let fs = FuseMT::new(BackendAdapter::new(fs.into_fs()), 1);
    // TODO Fuse args (e.g. filesystem name)
    fuse_mt::mount(fs, mountpoint, &[])
}

pub fn spawn_mount<Fs: AsyncFilesystem + Send + Sync + 'static>(
    fs: impl IntoFs<Fs>,
    mountpoint: impl AsRef<Path>,
) -> std::io::Result<RunningFilesystem> {
    // TODO Num threads
    let fs = FuseMT::new(BackendAdapter::new(fs.into_fs()), 1);
    // TODO Fuse args (e.g. filesystem name)
    let handle = fuse_mt::spawn_mount(fs, mountpoint, &[])?;
    Ok(RunningFilesystem::new(handle))
}
