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
    pub blocks: u64,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub ctime: SystemTime,
}

#[async_trait]
pub trait Node {
    async fn getattr(&self) -> FsResult<NodeAttrs>;
    async fn chmod(&self, mode: Mode) -> FsResult<()>;
    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()>;
}
