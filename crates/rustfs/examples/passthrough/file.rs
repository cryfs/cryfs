use async_trait::async_trait;

use cryfs_rustfs::{
    object_based_api::File, AbsolutePathBuf, FsError, FsResult, NumBytes, OpenFlags,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use super::device::PassthroughDevice;
use super::errors::{IoResultExt, NixResultExt};
use super::openfile::PassthroughOpenFile;

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

    async fn open(&self, openflags: OpenFlags) -> FsResult<AsyncDropGuard<PassthroughOpenFile>> {
        let mut options = tokio::fs::OpenOptions::new();
        match openflags {
            OpenFlags::Read => options.read(true),
            OpenFlags::Write => options.write(true),
            OpenFlags::ReadWrite => options.read(true).write(true),
        };
        let open_file = options.open(&self.path).await.map_error()?;
        Ok(PassthroughOpenFile::new(open_file))
    }
}
