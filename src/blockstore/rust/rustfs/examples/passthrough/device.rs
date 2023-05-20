use async_trait::async_trait;
use cryfs_rustfs::{object_based_api::Device, FsError, FsResult, Statfs};
use std::path::{Path, PathBuf};

use super::dir::PassthroughDir;
use super::file::PassthroughFile;
use super::node::PassthroughNode;
use super::openfile::PassthroughOpenFile;
use super::symlink::PassthroughSymlink;

use super::errors::NixResultExt;
use super::utils::apply_basedir;

pub struct PassthroughDevice {
    basedir: PathBuf,
}

impl PassthroughDevice {
    pub fn new(basedir: PathBuf) -> Self {
        Self { basedir }
    }

    fn apply_basedir(&self, path: &Path) -> PathBuf {
        apply_basedir(&self.basedir, path)
    }
}

#[async_trait]
impl Device for PassthroughDevice {
    type Node = PassthroughNode;
    type Dir = PassthroughDir;
    type Symlink = PassthroughSymlink;
    type File = PassthroughFile;
    type OpenFile = PassthroughOpenFile;

    async fn load_node(&self, path: &Path) -> FsResult<Self::Node> {
        let path = self.apply_basedir(path);
        Ok(PassthroughNode::new(path))
    }

    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir> {
        let path = self.apply_basedir(path);
        Ok(PassthroughDir::new(self.basedir.clone(), path))
    }

    async fn load_symlink(&self, path: &Path) -> FsResult<Self::Symlink> {
        let path = self.apply_basedir(path);
        Ok(PassthroughSymlink::new(path))
    }

    async fn load_file(&self, path: &Path) -> FsResult<Self::File> {
        let path = self.apply_basedir(path);
        Ok(PassthroughFile::new(path))
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        let path = self.basedir.clone();
        tokio::runtime::Handle::current()
            .spawn_blocking(move || {
                // TODO Make this platform independent
                let stat = nix::sys::statfs::statfs(&path).map_error()?;
                Ok(convert_statfs(stat))
            })
            .await
            .map_err(|_: tokio::task::JoinError| FsError::UnknownError)?
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
