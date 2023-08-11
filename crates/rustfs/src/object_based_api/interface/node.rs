use async_trait::async_trait;
use std::time::SystemTime;

use crate::common::{FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid};

#[async_trait]
pub trait Node {
    type Device: super::Device;

    async fn as_file(&self) -> FsResult<<Self::Device as super::Device>::File<'_>>;
    async fn as_dir(&self) -> FsResult<<Self::Device as super::Device>::Dir<'_>>;
    async fn as_symlink(&self) -> FsResult<<Self::Device as super::Device>::Symlink<'_>>;

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
}
