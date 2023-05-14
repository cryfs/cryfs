use async_trait::async_trait;
use cryfs_rustfs::{
    Dir, DirEntry, File, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, NumBytes, OpenFlags,
    Uid,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::SystemTime;

use super::device::InMemoryDevice;
use super::file::InMemoryFile;
use super::file::InMemoryOpenFile;
use super::node::{InMemoryNode, IsInMemoryNode};
use super::symlink::InMemorySymlink;

struct DirInode {
    metadata: NodeAttrs,
    entries: HashMap<String, InMemoryNode>,
}

impl DirInode {
    pub fn new(mode: Mode, uid: Uid, gid: Gid) -> Self {
        Self {
            metadata: NodeAttrs {
                // TODO What are the right dir attributes here for directories?
                nlink: 1,
                mode,
                uid,
                gid,
                num_bytes: NumBytes::from(0),
                blocks: 1,
                atime: SystemTime::now(),
                mtime: SystemTime::now(),
                ctime: SystemTime::now(),
            },
            entries: HashMap::new(),
        }
    }
}

pub struct InMemoryDir {
    implementation: Mutex<DirInode>,
}

impl InMemoryDir {
    pub fn new(mode: Mode, uid: Uid, gid: Gid) -> Self {
        Self {
            implementation: Mutex::new(DirInode::new(mode, uid, gid)),
        }
    }

    pub fn load_node_relative_path(&self, path: &Path) -> FsResult<InMemoryNode> {
        todo!()
    }
}

#[async_trait]
impl Dir for InMemoryDir {
    type Device = InMemoryDevice;

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        let implementation = self.implementation.lock().unwrap();
        Ok(implementation
            .entries
            .iter()
            .map(|(name, node)| {
                let kind = match node {
                    InMemoryNode::File(_) => NodeKind::File,
                    InMemoryNode::Dir(_) => NodeKind::Dir,
                    InMemoryNode::Symlink(_) => NodeKind::Symlink,
                };
                DirEntry {
                    name: name.clone(),
                    kind,
                }
            })
            .collect())
    }

    async fn create_child_dir(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let mut implementation = self.implementation.lock().unwrap();
        let dir = InMemoryDir::new(mode, uid, gid);
        let metadata = dir.metadata();
        // TODO Use try_insert once that is stable
        match implementation.entries.entry(name.to_string()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNode::Dir(dir));
            }
        }
        Ok(metadata)
    }

    async fn remove_child_dir(&self, name: &str) -> FsResult<()> {
        let mut implementation = self.implementation.lock().unwrap();
        // TODO Use try_insert once that is stable
        match implementation.entries.entry(name.to_string()) {
            std::collections::hash_map::Entry::Occupied(entry) => match entry.get() {
                InMemoryNode::File(_) | InMemoryNode::Symlink(_) => {
                    return Err(FsError::NodeIsNotADirectory);
                }
                InMemoryNode::Dir(dir) => {
                    entry.remove();
                    Ok(())
                }
            },
            std::collections::hash_map::Entry::Vacant(_) => Err(FsError::NodeDoesNotExist),
        }
    }

    async fn create_child_symlink(
        &self,
        name: &str,
        target: &Path,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let mut implementation = self.implementation.lock().unwrap();
        let symlink = InMemorySymlink::new(target.to_owned(), uid, gid);
        let metadata = symlink.metadata();
        // TODO Use try_insert once that is stable
        match implementation.entries.entry(name.to_string()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNode::Symlink(symlink));
            }
        }
        Ok(metadata)
    }

    async fn remove_child_file_or_symlink(&self, name: &str) -> FsResult<()> {
        let mut implementation = self.implementation.lock().unwrap();
        // TODO Use try_insert once that is stable
        match implementation.entries.entry(name.to_string()) {
            std::collections::hash_map::Entry::Occupied(entry) => match entry.get() {
                InMemoryNode::File(_) | InMemoryNode::Symlink(_) => {
                    entry.remove();
                    Ok(())
                }
                InMemoryNode::Dir(_) => return Err(FsError::NodeIsADirectory),
            },
            std::collections::hash_map::Entry::Vacant(_) => Err(FsError::NodeDoesNotExist),
        }
    }

    async fn create_and_open_file(
        &self,
        name: &str,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, InMemoryOpenFile)> {
        let mut implementation = self.implementation.lock().unwrap();
        let file = InMemoryFile::new(mode, uid, gid);
        let openfile = file.open_sync(OpenFlags::ReadWrite);
        let metadata = file.metadata();
        // TODO Use try_insert once that is stable
        match implementation.entries.entry(name.to_string()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNode::File(file));
            }
        }
        Ok((metadata, openfile))
    }

    async fn rename_child(&self, old_name: &str, new_path: &Path) -> FsResult<()> {
        todo!()
    }
}

impl IsInMemoryNode for InMemoryDir {
    fn metadata(&self) -> NodeAttrs {
        self.implementation.lock().unwrap().metadata
    }

    fn update_metadata(&self, callback: impl FnOnce(&mut NodeAttrs)) {
        let mut implementation = self.implementation.lock().unwrap();
        callback(&mut implementation.metadata);
    }
}
