use async_trait::async_trait;
use cryfs_rustfs::{
    object_based_api::{Dir, Node},
    DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, Uid,
};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use super::device::PassthroughDevice;
use super::errors::{IoResultExt, NixResultExt};
use super::node::PassthroughNode;
use super::openfile::PassthroughOpenFile;
use super::utils::{apply_basedir, convert_metadata};

pub struct PassthroughDir {
    basedir: PathBuf,
    path: PathBuf,
}

impl PassthroughDir {
    pub fn new(basedir: PathBuf, path: PathBuf) -> Self {
        Self { basedir, path }
    }
}

#[async_trait]
impl Dir for PassthroughDir {
    type Device = PassthroughDevice;

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&self.path).await.map_error()?;
        while let Some(entry) = dir.next_entry().await.map_error()? {
            let name = entry.file_name();
            let name = name.to_string_lossy().into_owned(); // TODO Is to_string_lossy the best way to convert from OsString to String?
            let node_type = entry.file_type().await.map_error()?;
            let kind = if node_type.is_file() {
                NodeKind::File
            } else if node_type.is_dir() {
                NodeKind::Dir
            } else if node_type.is_symlink() {
                NodeKind::Symlink
            } else {
                panic!(
                    "Unknown node type in {path:?} : {entry:?}",
                    path = self.path,
                );
            };
            entries.push(DirEntry { name, kind });
        }
        Ok(entries)
    }

    async fn create_child_dir(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let path = self.path.join(name);
        let path_clone = path.clone();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                // TODO Don't use unwrap
                nix::unistd::mkdir(
                    &path_clone,
                    nix::sys::stat::Mode::from_bits(mode.into()).unwrap(),
                )
                .map_error()?;
                nix::unistd::chown(
                    &path_clone,
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        // TODO Return value directly without another call but make sure it returns the same value
        PassthroughNode::new(path).getattr().await
    }

    async fn remove_child_dir(&self, name: &str) -> FsResult<()> {
        let path = self.path.join(name);
        tokio::fs::remove_dir(path).await.map_error()?;
        Ok(())
    }

    async fn create_child_symlink(
        &self,
        name: &str,
        target: &Path,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let path = self.path.join(name);
        let path_clone = path.clone();
        let target = target.to_owned();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                std::os::unix::fs::symlink(&target, &path_clone).map_error()?;
                nix::unistd::fchownat(
                    None,
                    &path_clone,
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                    nix::unistd::FchownatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        // TODO Return value directly without another call but make sure it returns the same value
        PassthroughNode::new(path).getattr().await
    }

    async fn remove_child_file_or_symlink(&self, name: &str) -> FsResult<()> {
        let path = self.path.join(name);
        tokio::fs::remove_file(path).await.map_error()?;
        Ok(())
    }

    async fn create_and_open_file(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, PassthroughOpenFile)> {
        let path = self.path.join(name);
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                let open_file = std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(mode.into())
                    .open(&path)
                    .map_error()?;
                // TODO Can we compute the Metadata without asking the underlying file system? We just created the file after all.
                let metadata = open_file.metadata().map_error()?;
                nix::unistd::fchownat(
                    None,
                    &path,
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                    nix::unistd::FchownatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok((
                    convert_metadata(metadata)?,
                    PassthroughOpenFile::new(tokio::fs::File::from_std(open_file)),
                ))
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }

    async fn rename_child(&self, old_name: &str, new_path: &Path) -> FsResult<()> {
        let old_path = self.path.join(old_name);
        let new_path = apply_basedir(&self.basedir, new_path);
        tokio::fs::rename(old_path, new_path).await.map_error()?;
        Ok(())
    }
}
