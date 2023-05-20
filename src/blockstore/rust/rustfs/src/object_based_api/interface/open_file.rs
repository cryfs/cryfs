use async_trait::async_trait;
use cryfs_utils::data::Data;

use crate::common::{FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid};
use std::time::SystemTime;

#[async_trait]
pub trait OpenFile {
    async fn getattr(&self) -> FsResult<NodeAttrs>;
    async fn chmod(&self, mode: Mode) -> FsResult<()>;
    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()>;
    async fn truncate(&self, new_size: NumBytes) -> FsResult<()>;
    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()>;
    // TODO Is it a better API to return a &[u8] from `read` by having the implementation pass &[u8] to a callback instead of returning a Data object? Might reduce copies. fuse-mt does this.
    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data>;
    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()>;
    async fn flush(&self) -> FsResult<()>;
    async fn fsync(&self, datasync: bool) -> FsResult<()>;
}
