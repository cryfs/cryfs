use async_trait::async_trait;
use std::time::SystemTime;

use super::Device;
use crate::common::{FsResult, Gid, Mode, NodeAttrs, Uid};
use cryfs_utils::async_drop::AsyncDropGuard;

#[async_trait]
pub trait Node {
    type Device: super::Device;

    async fn as_file(&self) -> FsResult<<Self::Device as super::Device>::File<'_>>;
    async fn as_dir(&self) -> FsResult<<Self::Device as super::Device>::Dir<'_>>;
    async fn as_symlink(&self) -> FsResult<<Self::Device as super::Device>::Symlink<'_>>;

    async fn getattr(&self) -> FsResult<NodeAttrs>;
    async fn chmod(&self, mode: Mode) -> FsResult<()>;
    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()>;
    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()>;
}
