use async_trait::async_trait;
use cryfs_rustfs::{Dir, DirEntry, FsResult, Gid, Mode, NodeAttrs, NumBytes, Uid};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use super::device::InMemoryDevice;
use super::file::InMemoryFile;
use super::node::IsInMemoryNode;

pub struct InMemoryDirEntry {}

pub struct InMemoryDir {
    metadata: Mutex<NodeAttrs>,
    entries: HashMap<String, InMemoryDirEntry>,
}

impl InMemoryDir {
    pub fn new() -> Self {
        Self {
            metadata: Mutex::new(NodeAttrs {
                // TODO What are the right dir attributes here for the rootdir?
                nlink: 1,
                mode: Mode::from(0),
                uid: Uid::from(0),
                gid: Gid::from(0),
                num_bytes: NumBytes::from(0),
                blocks: 1,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
            }),
            entries: HashMap::new(),
        }
    }
}

#[async_trait]
impl Dir for InMemoryDir {
    type Device = InMemoryDevice;

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        todo!()
    }

    async fn create_child_dir(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        todo!()
    }

    async fn remove_child_dir(&self, name: &str) -> FsResult<()> {
        todo!()
    }

    async fn create_child_symlink(
        &self,
        name: &str,
        target: &Path,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        todo!()
    }

    async fn remove_child_file_or_symlink(&self, name: &str) -> FsResult<()> {
        todo!()
    }

    async fn create_and_open_file(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, InMemoryFile)> {
        todo!()
    }

    async fn rename_child(&self, old_name: &str, new_path: &Path) -> FsResult<()> {
        todo!()
    }
}

impl IsInMemoryNode for InMemoryDir {
    fn metadata(&self) -> NodeAttrs {
        *self.metadata.lock().unwrap()
    }

    fn update_metadata(&self, callback: impl FnOnce(&mut NodeAttrs)) {
        let mut metadata = self.metadata.lock().unwrap();
        callback(&mut metadata);
    }
}
