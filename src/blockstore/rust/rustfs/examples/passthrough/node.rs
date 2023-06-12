use async_trait::async_trait;
use cryfs_rustfs::{
    object_based_api::Node, AbsolutePathBuf, FsError, FsResult, Gid, Mode, NodeAttrs, Uid,
};
use std::os::unix::fs::PermissionsExt;
use std::time::SystemTime;

use super::errors::{IoResultExt, NixResultExt};
use super::utils::{convert_metadata, convert_timespec};

pub struct PassthroughNode {
    path: AbsolutePathBuf,
}

impl PassthroughNode {
    pub fn new(path: AbsolutePathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Node for PassthroughNode {
    async fn getattr(&self) -> FsResult<NodeAttrs> {
        let metadata = tokio::fs::symlink_metadata(&self.path).await.map_error()?;
        convert_metadata(metadata)
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
                    None,
                    path.as_str(),
                    uid,
                    gid,
                    nix::unistd::FchownatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        Ok(())
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
                        let metadata = std::path::Path::new(self.path.as_str())
                            .metadata()
                            .map_error()?;
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
                    None,
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
