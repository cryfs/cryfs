use async_trait::async_trait;

use crate::common::{FsResult, NumBytes, OpenFlags};
use cryfs_utils::async_drop::AsyncDropGuard;

#[async_trait]
pub trait File {
    type Device: super::Device;

    async fn open<'a>(
        &'a self,
        flags: OpenFlags,
    ) -> FsResult<AsyncDropGuard<<Self::Device as super::Device>::OpenFile>>;

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()>;
}
