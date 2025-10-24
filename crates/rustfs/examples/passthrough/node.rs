use async_trait::async_trait;
use nix::fcntl::{AT_FDCWD, AtFlags};
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;

use cryfs_rustfs::{
    AbsolutePathBuf, FsError, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid, object_based_api::Node,
};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};

use super::errors::{IoResultExt, NixResultExt};
use super::utils::{convert_metadata, convert_timespec};
use super::{
    PassthroughDevice, dir::PassthroughDir, file::PassthroughFile, symlink::PassthroughSymlink,
};

#[derive(Debug)]
pub struct PassthroughNode {
    path: AbsolutePathBuf,
}

impl PassthroughNode {
    pub fn new(path: AbsolutePathBuf) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { path })
    }

    async fn chmod(&self, mode: Mode) -> FsResult<()> {
        let path = self.path.clone().push_all(&self.path);
        let permissions = std::fs::Permissions::from_mode(mode.into());
        tokio::fs::set_permissions(path, permissions)
            .await
            .map_error()?;
        Ok(())
    }

    async fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) -> FsResult<()> {
        let uid = uid.map(|uid| nix::unistd::Uid::from_raw(uid.into()));
        let gid = gid.map(|gid| nix::unistd::Gid::from_raw(gid.into()));
        let path = self.path.clone();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                nix::unistd::fchownat(
                    AT_FDCWD,
                    path.as_str(),
                    uid,
                    gid,
                    AtFlags::AT_SYMLINK_NOFOLLOW,
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
    }

    async fn truncate(&self, new_size: NumBytes) -> FsResult<()> {
        let path = self.path.clone();
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                nix::unistd::truncate(path.as_str(), u64::from(new_size) as libc::off_t)
                    .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }

    async fn utimens(
        &self,
        last_access: Option<SystemTime>,
        last_modification: Option<SystemTime>,
    ) -> FsResult<()> {
        let path = self.path.clone();
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                let (atime, mtime) = match (last_access, last_modification) {
                    (Some(atime), Some(mtime)) => {
                        // Both atime and mtime are being overwritten, no need to load previous values
                        (atime, mtime)
                    }
                    (atime, mtime) => {
                        // Either atime or mtime are not being overwritten, we need to load the previous values first.
                        let metadata =
                            std::path::Path::new(path.as_str()).metadata().map_error()?;
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
                nix::sys::stat::utimensat(
                    AT_FDCWD,
                    path.as_str(),
                    &convert_timespec(atime),
                    &convert_timespec(mtime),
                    nix::sys::stat::UtimensatFlags::NoFollowSymlink,
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
impl Node for PassthroughNode {
    type Device = PassthroughDevice;

    async fn as_file(&self) -> FsResult<AsyncDropGuard<PassthroughFile>> {
        // TODO Fail if it's the wrong type
        Ok(AsyncDropGuard::new(PassthroughFile::new(self.path.clone())))
    }

    async fn as_dir(&self) -> FsResult<AsyncDropGuard<PassthroughDir>> {
        // TODO Fail if it's the wrong type
        Ok(AsyncDropGuard::new(PassthroughDir::new(self.path.clone())))
    }

    async fn as_symlink(&self) -> FsResult<AsyncDropGuard<PassthroughSymlink>> {
        // TODO Fail if it's the wrong type
        Ok(AsyncDropGuard::new(PassthroughSymlink::new(
            self.path.clone(),
        )))
    }

    async fn getattr(&self) -> FsResult<NodeAttrs> {
        let metadata = tokio::fs::symlink_metadata(&self.path).await.map_error()?;
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
}

#[async_trait]
impl AsyncDrop for PassthroughNode {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        // Nothing to do
        Ok(())
    }
}
