use std::fmt::Debug;
use std::time::SystemTime;

use crate::common::{FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid};
use cryfs_utils::data::Data;

pub trait OpenFile: Debug {
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

    // TODO Is it a better API to return a &[u8] from `read` by having the implementation pass &[u8] to a callback instead of returning a Data object? Might reduce copies. fuse-mt does this.
    fn read(&self, offset: NumBytes, size: NumBytes)
    -> impl Future<Output = FsResult<Data>> + Send;
    fn write(&self, offset: NumBytes, data: Data) -> impl Future<Output = FsResult<()>> + Send;
    fn flush(&self) -> impl Future<Output = FsResult<()>> + Send;
    fn fsync(&self, datasync: bool) -> impl Future<Output = FsResult<()>> + Send;
}
