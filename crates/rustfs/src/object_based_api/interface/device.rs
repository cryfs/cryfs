use async_trait::async_trait;
use std::fmt::Debug;

use crate::common::{AbsolutePath, FsError, FsResult, Statfs};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

// TODO We only call this `Device` because that's the historical name from the c++ Cryfs version. We should probably rename this to `Filesystem`.
#[async_trait]
pub trait Device {
    type Node: super::Node<Device = Self> + AsyncDrop<Error = FsError> + Debug;
    type Dir<'a>: super::Dir<Device = Self>
    where
        Self: 'a;
    type Symlink<'a>: super::Symlink
    where
        Self: 'a;
    type File<'a>: super::File<Device = Self>
    where
        Self: 'a;
    type OpenFile: super::OpenFile + AsyncDrop<Error = FsError>;

    async fn lookup(&self, path: &AbsolutePath) -> FsResult<AsyncDropGuard<Self::Node>>;
    async fn rename(&self, from: &AbsolutePath, to: &AbsolutePath) -> FsResult<()>;
    async fn statfs(&self) -> FsResult<Statfs>;
    async fn destroy(self);
}
