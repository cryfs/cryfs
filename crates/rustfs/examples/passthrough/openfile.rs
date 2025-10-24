use anyhow::Result;
use async_trait::async_trait;
use cryfs_rustfs::{
    Data, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid, object_based_api::OpenFile,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
use std::os::fd::AsFd;
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;

use super::errors::{IoResultExt, NixResultExt};
use super::utils::{convert_metadata, convert_timespec};

#[derive(Debug)]
pub struct PassthroughOpenFile {
    open_file: tokio::fs::File,
}

impl PassthroughOpenFile {
    pub fn new(open_file: tokio::fs::File) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { open_file })
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        let permissions = std::fs::Permissions::from_mode(mode.into());
        self.open_file
            .set_permissions(permissions)
            .await
            .map_error()?;
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        let uid = uid.map(|uid| nix::unistd::Uid::from_raw(uid.into()));
        let gid = gid.map(|gid| nix::unistd::Gid::from_raw(gid.into()));
        // TODO Can we do this without duplicating the file descriptor?
        let open_file = self.open_file.try_clone().await.map_error()?;

        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                nix::unistd::fchown(open_file.as_fd(), uid, gid).map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        self.open_file.set_len(new_size.into()).await.map_error()?;
        Ok(())
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        // TODO Can we do this without duplicating the file descriptor?
        let open_file = self
            .open_file
            .try_clone()
            .await
            .map_error()?
            .into_std()
            .await;
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                let (atime, mtime) = match (last_access, last_modification) {
                    (Some(atime), Some(mtime)) => {
                        // Both atime and mtime are being overwritten, no need to load previous values
                        (atime, mtime)
                    }
                    (atime, mtime) => {
                        // Either atime or mtime are not being overwritten, we need to load the previous values first.
                        let metadata = open_file.metadata().map_error()?;
                        let atime = match atime {
                            Some(atime) => atime,
                            None => metadata.accessed().map_error()?,
                        };
                        let mtime = match mtime {
                            Some(mtime) => mtime,
                            None => metadata.modified().map_error()?,
                        };
                        (atime, mtime)
                    }
                };
                nix::sys::stat::futimens(
                    open_file,
                    &convert_timespec(atime),
                    &convert_timespec(mtime),
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }
}

#[async_trait]
impl OpenFile for PassthroughOpenFile {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        let metadata = self.open_file.metadata().await.map_error()?;
        convert_metadata(metadata)
    }

    async fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs> {
        // TODO Or is setting ctime allowed? What would it mean?
        assert!(ctime.is_none(), "Can't directly set ctime");

        if let Some(mode) = mode {
            self.chmod(mode).await?;
        }
        if uid.is_some() || gid.is_some() {
            self.chown(uid, gid).await?;
        }
        if let Some(size) = size {
            self.truncate(size).await?;
        }
        if atime.is_some() || mtime.is_some() {
            self.utimens(atime, mtime).await?;
        }

        self.getattr().await
    }

    async fn read(&self, offset: NumBytes, size: NumBytes) -> FsResult<Data> {
        let mut buffer: Data = vec![0; usize::try_from(u64::from(size)).unwrap()].into();
        // TODO Is this possible without duplicating the file descriptor?
        let open_file = self.open_file.try_clone().await.map_error()?;
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // Using `pread` instead of `read` because `read` requires a call to `seek` first
                // and there could be a race condition if multiple tasks read from the same file
                // and overwrite each other's seek position.
                let res = nix::sys::uio::pread(
                    open_file.as_fd(),
                    &mut buffer,
                    i64::try_from(u64::from(offset)).unwrap(),
                )
                .map_error();
                match res {
                    Ok(num_read) => {
                        buffer.shrink_to_subregion(0..num_read);
                        Ok(buffer)
                    }
                    Err(err) => Err(err),
                }
            })
            .await
            .expect("Error in spawn_blocking task")
    }

    async fn write(&self, offset: NumBytes, data: Data) -> FsResult<()> {
        // TODO Is this possible without duplicating the file descriptor?
        let open_file = self.open_file.try_clone().await.map_error()?;
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // Using `pwrite` instead of `write` because `write` requires a call to `seek` first
                // and there could be a race condition if multiple tasks write to the same file
                // and overwrite each other's seek position.
                let num_written = nix::sys::uio::pwrite(
                    open_file.as_fd(),
                    data.as_ref(),
                    i64::try_from(u64::from(offset)).unwrap(),
                )
                .map_error()?;
                // TODO Should we handle the case where not all data was written gracefully by retrying to write the rest? The pwrite manpage says it's not an error if not all data gets written.
                assert_eq!(data.len(), num_written, "pwrite did not write all data");
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }

    async fn flush(&self) -> FsResult<()> {
        // flush strictly speaking isn't a request to sync dirty data,
        // but it's a good place to do it because it's usually triggered
        // by a `close` syscall and errors that happen here are reported
        // back as errors by the `close` syscall.
        self.open_file.sync_all().await.map_error()?;
        Ok(())
    }

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
        if datasync {
            // sync data and metadata
            self.open_file.sync_all().await.map_error()?;
        } else {
            // only sync data, not metadata
            self.open_file.sync_data().await.map_error()?;
        }
        Ok(())
    }
}

#[async_trait]
impl AsyncDrop for PassthroughOpenFile {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        // Nothing to do
        Ok(())
    }
}
