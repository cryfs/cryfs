use async_trait::async_trait;
use std::path::Path;

use super::error::FsResult;

// TODO We only call this `Device` because that's the historical name from the c++ Cryfs version. We should probably rename this to `Filesystem`.
#[async_trait]
pub trait Device {
    type Node: super::Node;
    type Dir: super::Dir;
    type Symlink: super::Symlink;

    // TODO Here and elsewhere in the interface, std::io::Result is probably the wrong error handling strategy
    async fn load_node(&self, path: &Path) -> FsResult<Self::Node>;

    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir>;

    async fn load_symlink(&self, path: &Path) -> FsResult<Self::Symlink>;
}
