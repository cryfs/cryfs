use cryfs_utils::async_drop::AsyncDropGuard;
use std::time::SystemTime;

use crate::common::{FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid};

pub trait Node {
    type Device: super::Device;

    fn as_file(
        &self,
    ) -> impl Future<Output = FsResult<AsyncDropGuard<<Self::Device as super::Device>::File<'_>>>> + Send;
    fn as_dir(
        &self,
    ) -> impl Future<Output = FsResult<AsyncDropGuard<<Self::Device as super::Device>::Dir<'_>>>> + Send;
    fn as_symlink(
        &self,
    ) -> impl Future<Output = FsResult<AsyncDropGuard<<Self::Device as super::Device>::Symlink<'_>>>>
    + Send;

    fn getattr(&self) -> impl Future<Output = FsResult<NodeAttrs>> + Send;
    fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> impl Future<Output = FsResult<NodeAttrs>> + Send;

    #[cfg(feature = "testutils")]
    fn fsync(&self, datasync: bool) -> impl Future<Output = FsResult<()>> + Send;

    /// Called when the node was moved to a different directory, or removed from its directory by unlink, rmdir,
    /// or a rename that overwrote it, while the node is still loaded, e.g. because the kernel still has its inode.
    /// A node that keeps its parent directory loaded can release it here, so it doesn't stand in the way of
    /// removing that directory.
    fn on_moved_out_or_removed(&self) -> impl Future<Output = FsResult<()>> + Send {
        async { Ok(()) }
    }
}
