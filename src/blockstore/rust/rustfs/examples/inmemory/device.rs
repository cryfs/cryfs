use async_trait::async_trait;
use cryfs_rustfs::{Device, FsResult, Statfs};
use std::path::Path;

use super::dir::InMemoryDir;
use super::file::InMemoryFile;
use super::node::InMemoryNode;
use super::symlink::InMemorySymlink;

pub struct InMemoryDevice {
    rootdir: InMemoryDir,
}

impl Default for InMemoryDevice {
    fn default() -> Self {
        Self {
            rootdir: InMemoryDir::new(),
        }
    }
}

#[async_trait]
impl Device for InMemoryDevice {
    type Node = InMemoryNode;
    type Dir = InMemoryDir;
    type Symlink = InMemorySymlink;
    type File = InMemoryFile;
    type OpenFile = InMemoryFile;

    async fn load_node(&self, path: &Path) -> FsResult<Self::Node> {
        todo!()
    }

    async fn load_dir(&self, path: &Path) -> FsResult<Self::Dir> {
        todo!()
    }

    async fn load_symlink(&self, path: &Path) -> FsResult<Self::Symlink> {
        todo!()
    }

    async fn load_file(&self, path: &Path) -> FsResult<Self::File> {
        todo!()
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        todo!()
    }
}
