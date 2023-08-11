use async_trait::async_trait;
use std::fmt::Debug;
use std::time::SystemTime;

use crate::common::{FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid};
use cryfs_utils::data::Data;

#[async_trait]
pub trait OpenFile: Debug {
    async fn getattr(&self) -> FsResult<NodeAttrs>;
    async fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs>;

    // TODO Is it a better API to return a &[u8] from `read` by having the implementation pass &[u8] to a callback instead of returning a Data object? Might reduce copies. fuse-mt does this.
    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data>;
    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()>;
    async fn flush(&self) -> FsResult<()>;
    async fn fsync(&self, datasync: bool) -> FsResult<()>;
}
