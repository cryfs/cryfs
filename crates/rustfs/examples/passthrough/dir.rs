use async_trait::async_trait;
use cryfs_rustfs::{
    DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, OpenInFlags, Uid,
    object_based_api::{Dir, Node},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    path::{AbsolutePathBuf, PathComponent},
    with_async_drop_2,
};
use nix::fcntl::{AT_FDCWD, AtFlags};
use std::os::unix::fs::OpenOptionsExt;
use tokio::fs::OpenOptions;

use super::device::PassthroughDevice;
use super::errors::{IoResultExt, NixResultExt};
use super::node::PassthroughNode;
use super::openfile::PassthroughOpenFile;
use super::symlink::PassthroughSymlink;
use super::utils::convert_metadata;

#[derive(Debug)]
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

    fn into_node(this: AsyncDropGuard<Self>) -> AsyncDropGuard<PassthroughNode> {
        PassthroughNode::new(this.unsafe_into_inner_dont_drop().path.clone())
    }

    async fn lookup_child(
        &self,
        name: &PathComponent,
    ) -> FsResult<AsyncDropGuard<PassthroughNode>> {
        // TODO cloning path and then pushing is inefficient. Allow a way to do this with just one allocation.
        let child_path = self.path.clone().push(name);
        if !tokio::fs::try_exists(&child_path).await.map_error()? {
            return Err(FsError::NodeDoesNotExist);
        }

        Ok(PassthroughNode::new(child_path))
    }

    async fn rename_child(&self, oldname: &PathComponent, newname: &PathComponent) -> FsResult<()> {
        let old_path = self.path.clone().push(oldname);
        let new_path = self.path.clone().push(newname);
        tokio::fs::rename(old_path, new_path).await.map_error()
    }

    async fn move_child_to(
        &self,
        oldname: &PathComponent,
        newparent: AsyncDropGuard<Self>,
        newname: &PathComponent,
    ) -> FsResult<()> {
        with_async_drop_2!(newparent, {
            let old_path = self.path.clone().push(oldname);
            let new_path = newparent.path.clone().push(newname);
            tokio::fs::rename(old_path, new_path).await.map_error()
        })
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
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<PassthroughDir>)> {
        let path = self.path.clone().push(name);
        let path_clone = path.clone();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                // TODO Don't use unwrap
                nix::unistd::mkdir(
                    path_clone.as_str(),
                    convert_mode(mode.remove_dir_flag().into()),
                )
                .map_error()?;
                nix::unistd::chown(
                    path_clone.as_str(),
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                )
                .map_error()?;
                Ok::<(), FsError>(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        let node = PassthroughDir::new(path.clone());
        // TODO Return value directly without another call but make sure it returns the same value
        let child_node = PassthroughNode::new(path);
        let attrs = with_async_drop_2!(child_node, { child_node.getattr().await })?;
        Ok((attrs, AsyncDropGuard::new(node)))
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
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<PassthroughSymlink>)> {
        let path = self.path.clone().push(name);
        let path_clone = path.clone();
        let target = target.to_owned();
        let _: () = tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                std::os::unix::fs::symlink(&target, &path_clone).map_error()?;
                nix::unistd::fchownat(
                    AT_FDCWD,
                    path_clone.as_str(),
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                    AtFlags::AT_SYMLINK_NOFOLLOW,
                )
                .map_error()?;
                Ok::<(), FsError>(())
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)??;
        // TODO Return value directly without another call but make sure it returns the same value
        let node = PassthroughNode::new(path);
        with_async_drop_2!(node, {
            let attrs = node.getattr().await?;
            let symlink = node.as_symlink().await?;
            Ok((attrs, symlink))
        })
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
        _flags: OpenInFlags,
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
                    AT_FDCWD,
                    path.as_str(),
                    Some(nix::unistd::Uid::from_raw(uid.into())),
                    Some(nix::unistd::Gid::from_raw(gid.into())),
                    AtFlags::AT_SYMLINK_NOFOLLOW,
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

    async fn fsync(&self, datasync: bool) -> FsResult<()> {
        // TODO Is it actually correct to open a directory with OpenOptions to fsync it?
        let dir_file = OpenOptions::new()
            .read(true)
            .open(&self.path)
            .await
            .map_error()?;
        if datasync {
            // sync data and metadata
            dir_file.sync_all().await.map_error()?;
        } else {
            // only sync data, not metadata
            dir_file.sync_data().await.map_error()?;
        }
        Ok(())
    }
}

fn convert_mode(mode: u32) -> nix::sys::stat::Mode {
    use nix::sys::stat::{Mode, mode_t};
    // Most systems seems ot use u32 for [mode_t], but MacOS seems to use u16.
    let mode = mode_t::try_from(mode).unwrap();
    Mode::from_bits(mode.into())
        .ok_or_else(|| anyhow::anyhow!("Invalid mode bits: 0b{mode:b}"))
        .unwrap()
}

#[async_trait]
impl AsyncDrop for PassthroughDir {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        // Nothing to do
        Ok(())
    }
}
