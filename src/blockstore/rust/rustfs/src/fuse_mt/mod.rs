//! This module allows running a file system using the [fuse-mt] library.

use fuse_mt::FuseMT;
use std::path::Path;

use crate::interface::Device;

mod running_filesystem;
pub use running_filesystem::RunningFilesystem;

mod fs_adapter;
use fs_adapter::FsAdapter;

pub fn mount<D>(fs: D, mountpoint: impl AsRef<Path>) -> std::io::Result<()>
where
    D: Device + Sync + Send + 'static,
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    D::OpenFile: Send + Sync,
{
    // TODO Ctrl+C doesn't do a clean unmount
    // TODO Num threads
    let fs = FuseMT::new(FsAdapter::new(fs), 1);
    // TODO Fuse args (e.g. filesystem name)
    fuse_mt::mount(fs, mountpoint, &[])
}

pub fn spawn_mount<D>(fs: D, mountpoint: impl AsRef<Path>) -> std::io::Result<RunningFilesystem>
where
    D: Device + Sync + Send + 'static,
    // TODO Is this send+sync bound only needed because fuse_mt goes multi threaded or would it also be required for fuser?
    D::OpenFile: Send + Sync,
{
    // TODO Num threads
    let fs = FuseMT::new(FsAdapter::new(fs), 1);
    // TODO Fuse args (e.g. filesystem name)
    let handle = fuse_mt::spawn_mount(fs, mountpoint, &[])?;
    Ok(RunningFilesystem::new(handle))
}
