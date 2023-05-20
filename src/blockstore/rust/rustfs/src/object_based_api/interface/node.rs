use async_trait::async_trait;
use std::time::SystemTime;

use crate::common::{FsResult, Gid, Mode, NodeAttrs, Uid};

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
