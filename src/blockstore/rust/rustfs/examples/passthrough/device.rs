use async_trait::async_trait;
use cryfs_rustfs::{
    object_based_api::Device, AbsolutePath, AbsolutePathBuf, FsError, FsResult, Statfs,
};

use super::dir::PassthroughDir;
use super::file::PassthroughFile;
use super::node::PassthroughNode;
use super::openfile::PassthroughOpenFile;
use super::symlink::PassthroughSymlink;

use super::errors::NixResultExt;

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
    type Node<'a> = PassthroughNode;
    type Dir<'a> = PassthroughDir;
    type Symlink<'a> = PassthroughSymlink;
    type File<'a> = PassthroughFile;
    type OpenFile = PassthroughOpenFile;

    async fn load_node(&self, path: &AbsolutePath) -> FsResult<Self::Node<'_>> {
        let path = self.apply_basedir(path);
        Ok(PassthroughNode::new(path))
    }

    async fn load_dir(&self, path: &AbsolutePath) -> FsResult<Self::Dir<'_>> {
        let path = self.apply_basedir(path);
        Ok(PassthroughDir::new(self.basedir.clone(), path))
    }

    async fn load_symlink(&self, path: &AbsolutePath) -> FsResult<Self::Symlink<'_>> {
        let path = self.apply_basedir(path);
        Ok(PassthroughSymlink::new(path))
    }

    async fn load_file(&self, path: &AbsolutePath) -> FsResult<Self::File<'_>> {
        let path = self.apply_basedir(path);
        Ok(PassthroughFile::new(path))
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
