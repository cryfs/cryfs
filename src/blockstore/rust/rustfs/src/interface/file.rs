use async_trait::async_trait;

use super::error::FsResult;
use crate::utils::OpenFlags;

#[async_trait]
pub trait File {
    type Device: super::Device;

    // TODO Do we need openflags as a parameter? C++ used that
    async fn open(&self, flags: OpenFlags) -> FsResult<<Self::Device as super::Device>::OpenFile>;
}
