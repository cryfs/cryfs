use async_trait::async_trait;

use super::errors::{IoResultExt, NixResultExt};
use cryfs_rustfs::{
    object_based_api::Device, AbsolutePath, AbsolutePathBuf, FsError, FsResult, Statfs,
};
use cryfs_utils::async_drop::AsyncDropGuard;

use super::dir::PassthroughDir;
use super::file::PassthroughFile;
use super::node::PassthroughNode;
use super::openfile::PassthroughOpenFile;
use super::symlink::PassthroughSymlink;

pub struct PassthroughDevice {
    basedir: AbsolutePathBuf,
}

impl PassthroughDevice {
    pub fn new(basedir: AbsolutePathBuf) -> Self {
        Self { basedir }
    }

    fn apply_basedir(&self, path: &AbsolutePath) -> AbsolutePathBuf {
        self.basedir.clone().push_all(path)
    }
}

#[async_trait]
impl Device for PassthroughDevice {
    type Node = PassthroughNode;
    type Dir<'a> = PassthroughDir;
    type Symlink<'a> = PassthroughSymlink;
    type File<'a> = PassthroughFile;
    type OpenFile = PassthroughOpenFile;

    async fn lookup(&self, path: &AbsolutePath) -> FsResult<AsyncDropGuard<Self::Node>> {
        let path = self.apply_basedir(path);
        Ok(PassthroughNode::new(path))
    }

    async fn rename(&self, from_path: &AbsolutePath, to_path: &AbsolutePath) -> FsResult<()> {
        // TODO Build AbsolutePathBuf::join(&self, &AbsolutePath) and join_all, which can be more efficient because clone+push likely causes two reallocations.
        //      Then grep the codebase for the clone().push{_all} pattern and replate it
        let old_path = self.basedir.clone().push_all(from_path);
        let new_path = self.basedir.clone().push_all(to_path);
        tokio::fs::rename(old_path, new_path).await.map_error()?;
        Ok(())
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        let path = self.basedir.clone();
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                let stat = nix::sys::statfs::statfs(path.as_str()).map_error()?;
                Ok(convert_statfs(stat))
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
    }

    async fn destroy(self) {
        // Nothing to do
    }
}

#[cfg(not(target_os = "macos"))]
fn convert_statfs(stat: nix::sys::statfs::Statfs) -> Statfs {
    Statfs {
        // TODO Don't use unwrap
        max_filename_length: u32::try_from(stat.maximum_name_length()).unwrap(),
        blocksize: u32::try_from(stat.block_size()).unwrap(),
        num_total_blocks: stat.blocks(),
        num_free_blocks: stat.blocks_free(),
        num_available_blocks: stat.blocks_available(),
        num_total_inodes: stat.files(),
        num_free_inodes: stat.files_free(),
    }
}

#[cfg(target_os = "macos")]
fn convert_statfs(stat: nix::sys::statfs::Statfs) -> Statfs {
    Statfs {
        // TODO Don't use unwrap
        // TODO What max_filename_length to set in macos?
        max_filename_length: 255,
        blocksize: u32::try_from(stat.block_size()).unwrap(),
        num_total_blocks: stat.blocks(),
        num_free_blocks: stat.blocks_free(),
        num_available_blocks: stat.blocks_available(),
        num_total_inodes: stat.files(),
        num_free_inodes: stat.files_free(),
    }
}
