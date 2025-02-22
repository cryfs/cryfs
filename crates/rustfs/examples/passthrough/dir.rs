use async_trait::async_trait;
use cryfs_rustfs::{
    AbsolutePathBuf, DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, PathComponent,
    Uid,
    object_based_api::{Dir, Node},
};
use cryfs_utils::async_drop::AsyncDropGuard;
use std::os::unix::fs::OpenOptionsExt;

use super::device::PassthroughDevice;
use super::errors::{IoResultExt, NixResultExt};
use super::node::PassthroughNode;
use super::openfile::PassthroughOpenFile;
use super::symlink::PassthroughSymlink;
use super::utils::convert_metadata;

pub struct PassthroughDir {
    path: AbsolutePathBuf,
}

impl PassthroughDir {
    pub fn new(path: AbsolutePathBuf) -> Self {
        Self { path }
    }
}

#[async_trait]
impl Dir for PassthroughDir {
    type Device = PassthroughDevice;

    fn as_node(&self) -> AsyncDropGuard<PassthroughNode> {
        PassthroughNode::new(self.path.clone())
    }

    async fn lookup_child(
        &self,
        name: &PathComponent,
    ) -> FsResult<AsyncDropGuard<PassthroughNode>> {
        // TODO cloning path and then pushing is inefficient. Allow a way to do this with just one allocation.
        let path = self.path.clone();
        let path = path.push(name);
        Ok(PassthroughNode::new(path))
    }

    async fn rename_child(&self, oldname: &PathComponent, newname: &PathComponent) -> FsResult<()> {
        let old_path = self.path.clone().push(oldname);
        let new_path = self.path.clone().push(newname);
        tokio::fs::rename(old_path, new_path).await.map_error()
    }

    async fn move_child_to(
        &self,
        oldname: &PathComponent,
        newparent: Self,
        newname: &PathComponent,
    ) -> FsResult<()> {
        let old_path = self.path.clone().push(oldname);
        let new_path = newparent.path.clone().push(newname);
        tokio::fs::rename(old_path, new_path).await.map_error()
    }

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(&self.path).await.map_error()?;
        while let Some(entry) = dir.next_entry().await.map_error()? {
            // TODO Do we need to filter out '.' and '..'?
            let name = entry
                .file_name()
                .into_string()
                .map_err(|err| FsError::CorruptedFilesystem {
                    message: format!("{err:?}"),
                })?
                .try_into()
                .map_err(|err| FsError::CorruptedFilesystem {
                    message: format!("{err:?}"),
                })?;
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
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, PassthroughDir)> {
        let path = self.path.clone().push(name);
        let path_clone = path.clone();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                // TODO Don't use unwrap
                nix::unistd::mkdir(path_clone.as_str(), convert_mode(mode.into())).map_error()?;
                nix::unistd::chown(
                    path_clone.as_str(),
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                )
                .map_error()?;
                Ok(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        let node = PassthroughDir::new(path.clone());
        // TODO Return value directly without another call but make sure it returns the same value
        let attrs = PassthroughNode::new(path).getattr().await?;
        Ok((attrs, node))
    }

    async fn remove_child_dir(&self, name: &PathComponent) -> FsResult<()> {
        let path = self.path.clone().push(name);
        tokio::fs::remove_dir(path).await.map_error()?;
        Ok(())
    }

    async fn create_child_symlink(
        &self,
        name: &PathComponent,
        target: &str,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, PassthroughSymlink)> {
        let path = self.path.clone().push(name);
        let path_clone = path.clone();
        let target = target.to_owned();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                std::os::unix::fs::symlink(&target, &path_clone).map_error()?;
                nix::unistd::fchownat(
                    None,
                    path_clone.as_str(),
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
        let node = PassthroughNode::new(path);
        let attrs = node.getattr().await?;
        let symlink = node.as_symlink().await?;
        Ok((attrs, symlink))
    }

    async fn remove_child_file_or_symlink(&self, name: &PathComponent) -> FsResult<()> {
        let path = self.path.clone().push(name);
        tokio::fs::remove_file(path).await.map_error()?;
        Ok(())
    }

    async fn create_and_open_file(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(
        NodeAttrs,
        AsyncDropGuard<PassthroughNode>,
        AsyncDropGuard<PassthroughOpenFile>,
    )> {
        let path = self.path.clone().push(name);
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
                    path.as_str(),
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                    nix::unistd::FchownatFlags::NoFollowSymlink,
                )
                .map_error()?;
                Ok((
                    convert_metadata(metadata)?,
                    PassthroughNode::new(path),
                    PassthroughOpenFile::new(tokio::fs::File::from_std(open_file)),
                ))
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }
}

fn convert_mode(mode: u32) -> nix::sys::stat::Mode {
    use nix::sys::stat::{Mode, mode_t};
    // Most systems seems ot use u32 for [mode_t], but MacOS seems to use u16.
    let mode = mode_t::try_from(mode).unwrap();
    Mode::from_bits(mode.into()).unwrap()
}
