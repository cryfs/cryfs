use async_trait::async_trait;
use std::fmt::Debug;

use crate::common::{AbsolutePath, FsError, FsResult, Statfs};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

// TODO We only call this `Device` because that's the historical name from the c++ Cryfs version. We should probably rename this to `Filesystem`.
#[async_trait]
pub trait Device {
    type Node<'a>: super::Node
    where
        Self: 'a;
    type Dir<'a>: super::Dir<Device = Self>
    where
        Self: 'a;
    type Symlink<'a>: super::Symlink
    where
        Self: 'a;
    type File<'a>: super::File<Device = Self> + AsyncDrop<Error = FsError> + Debug
    where
        Self: 'a;
    type OpenFile: super::OpenFile + AsyncDrop<Error = FsError> + Send;

    // TODO Here and elsewhere in the interface, std::io::Result is probably the wrong error handling strategy
    async fn load_node<'a>(&'a self, path: &AbsolutePath) -> FsResult<Self::Node<'a>>;
    async fn load_dir<'a>(&'a self, path: &AbsolutePath) -> FsResult<Self::Dir<'a>>;
    async fn load_symlink<'a>(&'a self, path: &AbsolutePath) -> FsResult<Self::Symlink<'a>>;
    async fn load_file<'a>(
        &'a self,
        path: &AbsolutePath,
    ) -> FsResult<AsyncDropGuard<Self::File<'a>>>;
    async fn rename(&self, from: &AbsolutePath, to: &AbsolutePath) -> FsResult<()>;
    async fn statfs(&self) -> FsResult<Statfs>;
    async fn destroy(self);
}
