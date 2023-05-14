use async_trait::async_trait;
use std::path::Path;

use super::error::FsResult;

pub struct Statfs {
    pub max_filename_length: u32,

    /// Optimal transfer block size
    pub blocksize: u32,

    /// Total data blocks in the filesystem
    pub num_total_blocks: u64,

    /// Free blocks in filesystem
    pub num_free_blocks: u64,

    /// Free blocks available to unprivileged user
    pub num_available_blocks: u64,

    /// Total number of inodes in filesystem
    /// TODO Is this supposed to only count files or also directories? It's called `files` in the statvfs struct
    pub num_total_inodes: u64,

    // Free inodes in filesystem
    pub num_free_inodes: u64,
    // Fuse ignores the `f_avail` field of statfs, so we don't have a `num_available_inodes` representing it here.
    // See https://libfuse.github.io/doxygen/structfuse__operations.html#a76d29dba617a64321cf52d62cd969292
}

// TODO We only call this `Device` because that's the historical name from the c++ Cryfs version. We should probably rename this to `Filesystem`.
#[async_trait]
pub trait Device {
    type Node: super::Node;
    type Dir: super::Dir<Device = Self>;
    type Symlink: super::Symlink;
    type File: super::File<Device = Self>;
    type OpenFile: super::OpenFile;

    // TODO Here and elsewhere in the interface, std::io::Result is probably the wrong error handling strategy
    async fn load_node(&self, path: &Path) -> FsResult<Self::Node>;
    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir>;
    async fn load_symlink(&self, path: &Path) -> FsResult<Self::Symlink>;
    async fn load_file(&self, path: &Path) -> FsResult<Self::File>;
    async fn statfs(&self) -> FsResult<Statfs>;
}
