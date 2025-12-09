use async_trait::async_trait;
use std::fmt::Debug;

use crate::common::{FsResult, OpenInFlags};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

#[async_trait]
pub trait File: AsyncDrop + Debug + Sized {
    type Device: super::Device;

    async fn into_open(
        this: AsyncDropGuard<Self>,
        flags: OpenInFlags,
    ) -> FsResult<AsyncDropGuard<<Self::Device as super::Device>::OpenFile>>;
}
