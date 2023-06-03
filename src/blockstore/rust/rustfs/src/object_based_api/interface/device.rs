use async_trait::async_trait;
use std::path::Path;

use crate::common::{FsResult, Statfs};

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
    async fn destroy(self);
}
