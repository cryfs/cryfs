use async_trait::async_trait;
use cryfs_rustfs::{File, FsError, FsResult, NumBytes, OpenFlags};
use std::path::PathBuf;

use super::device::PassthroughDevice;
use super::errors::{IoResultExt, NixResultExt};
use super::openfile::PassthroughOpenFile;

pub struct PassthroughFile {
    path: PathBuf,
}

impl PassthroughFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl File for PassthroughFile {
    type Device = PassthroughDevice;

    async fn open(&self, openflags: OpenFlags) -> FsResult<PassthroughOpenFile> {
        let mut options = tokio::fs::OpenOptions::new();
        match openflags {
            OpenFlags::Read => options.read(true),
            OpenFlags::Write => options.write(true),
            OpenFlags::ReadWrite => options.read(true).write(true),
        };
        let open_file = options.open(&self.path).await.map_error()?;
        Ok(PassthroughOpenFile::new(open_file))
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        let path = self.path.clone();
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                nix::unistd::truncate(&path, u64::from(new_size) as libc::off_t).map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }
}
