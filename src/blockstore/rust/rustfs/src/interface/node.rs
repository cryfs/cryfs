use async_trait::async_trait;
use std::time::SystemTime;

use super::error::FsResult;
use crate::utils::{Gid, Mode, NumBytes, Uid};

#[derive(Debug, Clone, Copy)]
pub struct NodeAttrs {
    pub nlink: u32,
    pub mode: Mode,
    pub uid: Uid,
    pub gid: Gid,
    pub num_bytes: NumBytes,

    /// `num_blocks` is the number of 512B blocks allocated for this node.
    /// This is only needed for special cases like files with holes in them.
    /// Otherwise, `num_blocks == num_bytes / 512` is correct and if you're ok
    /// with that default, you can leave this field as `None`.
    pub num_blocks: Option<u64>,

    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
}

#[async_trait]
pub trait Node {
    async fn getattr(&self) -> FsResult<NodeAttrs>;
    async fn chmod(&self, mode: Mode) -> FsResult<()>;
    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()>;
    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()>;
}
