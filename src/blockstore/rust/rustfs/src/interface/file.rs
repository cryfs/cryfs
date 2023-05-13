use async_trait::async_trait;

use super::error::FsResult;
use crate::utils::{NumBytes, OpenFlags};

#[async_trait]
pub trait File {
    type Device: super::Device;

    async fn open(&self, flags: OpenFlags) -> FsResult<<Self::Device as super::Device>::OpenFile>;

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()>;
}
