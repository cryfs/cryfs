use async_trait::async_trait;

use cryfs_rustfs::FsError;
use cryfs_rustfs::{FsResult, OpenInFlags, object_based_api::File};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    path::AbsolutePathBuf,
};

use super::device::PassthroughDevice;
use super::errors::IoResultExt;
use super::openfile::PassthroughOpenFile;

#[derive(Debug)]
pub struct PassthroughFile {
    path: AbsolutePathBuf,
}

impl PassthroughFile {
    pub fn new(path: AbsolutePathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl File for PassthroughFile {
    type Device = PassthroughDevice;

    async fn into_open(
        this: AsyncDropGuard<Self>,
        openflags: OpenInFlags,
    ) -> FsResult<AsyncDropGuard<PassthroughOpenFile>> {
        let this = this.unsafe_into_inner_dont_drop();
        let mut options = tokio::fs::OpenOptions::new();
        match openflags {
            OpenInFlags::Read => options.read(true),
            OpenInFlags::Write => options.write(true),
            OpenInFlags::ReadWrite => options.read(true).write(true),
        };
        let open_file = options.open(&this.path).await.map_error()?;
        Ok(PassthroughOpenFile::new(open_file))
    }
}

#[async_trait]
impl AsyncDrop for PassthroughFile {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        // Nothing to do
        Ok(())
    }
}
