use async_trait::async_trait;
use cryfs_rustfs::{
    object_based_api::Dir, DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, NumBytes,
    OpenFlags, PathComponent, PathComponentBuf, Uid,
};
use cryfs_utils::async_drop::AsyncDropGuard;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::device::InMemoryDevice;
use super::file::InMemoryFileRef;
use super::file::InMemoryOpenFileRef;
use super::inode_metadata::{chmod, chown, utimens};
use super::node::InMemoryNodeRef;
use super::symlink::InMemorySymlinkRef;

// Inode is in separate module so we can ensure class invariant through public/private boundaries
mod inode {
    use super::*;

    pub struct DirInode {
        metadata: NodeAttrs,
        entries: HashMap<PathComponentBuf, InMemoryNodeRef>,
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
                    num_blocks: None,
                    atime: SystemTime::now(),
                    mtime: SystemTime::now(),
                    ctime: SystemTime::now(),
                },
                entries: HashMap::new(),
            }
        }

        pub fn metadata(&self) -> &NodeAttrs {
            &self.metadata
        }

        pub fn chmod(&mut self, mode: Mode) {
            chmod(&mut self.metadata, mode);
        }

        pub fn chown(&mut self, uid: Option<Uid>, gid: Option<Gid>) {
            chown(&mut self.metadata, uid, gid);
        }

        pub fn utimens(
            &mut self,
            last_access: Option<SystemTime>,
            last_modification: Option<SystemTime>,
        ) {
            utimens(&mut self.metadata, last_access, last_modification);
        }

        pub fn entries(&self) -> &HashMap<PathComponentBuf, InMemoryNodeRef> {
            &self.entries
        }

        // TODO Once we have an invariant that depends on the number of entries
        //      (e.g. metadata.num_bytes),
        //      we can't offer `entries_mut` as a function anymore because it could
        //      violate that invariant.
        pub fn entries_mut(&mut self) -> &mut HashMap<PathComponentBuf, InMemoryNodeRef> {
            &mut self.entries
        }

        pub fn rename(&mut self, from: &PathComponent, to: &PathComponent) -> FsResult<()> {
            if self.entries.contains_key(to) {
                // TODO Some forms of overwriting are actually ok, we don't need to block them all
                Err(FsError::NodeAlreadyExists)
            } else {
                let old_entry = match self.entries.remove(from) {
                    Some(node) => node,
                    None => {
                        return Err(FsError::NodeDoesNotExist);
                    }
                };
                // TODO Use try_insert once stable
                let insert_result = self.entries.insert(to.to_owned(), old_entry);
                assert!(insert_result.is_none(), "We checked above that `new_name` doesn't exist in the map. Inserting it shouldn't fail.");
                Ok(())
            }
        }
    }
}
pub use inode::DirInode;

pub struct InMemoryDirRef {
    inode: Arc<Mutex<DirInode>>,
}

impl InMemoryDirRef {
    pub fn new(mode: Mode, uid: Uid, gid: Gid) -> Self {
        Self {
            inode: Arc::new(Mutex::new(DirInode::new(mode, uid, gid))),
        }
    }

    pub fn from_inode(inode: Arc<Mutex<DirInode>>) -> Self {
        Self { inode }
    }

    pub fn clone_ref(&self) -> Self {
        Self {
            inode: Arc::clone(&self.inode),
        }
    }

    pub fn metadata(&self) -> NodeAttrs {
        let inode = self.inode.lock().unwrap();
        *inode.metadata()
    }

    pub fn get_child(&self, name: &PathComponent) -> FsResult<InMemoryNodeRef> {
        let inode = self.inode.lock().unwrap();
        match inode.entries().get(name) {
            Some(node) => Ok(node.clone_ref()),
            None => Err(FsError::NodeDoesNotExist),
        }
    }

    pub fn chmod(&self, mode: Mode) {
        self.inode.lock().unwrap().chmod(mode);
    }

    pub fn chown(&self, uid: Option<Uid>, gid: Option<Gid>) {
        self.inode.lock().unwrap().chown(uid, gid);
    }

    pub fn utimens(&self, last_access: Option<SystemTime>, last_modification: Option<SystemTime>) {
        self.inode
            .lock()
            .unwrap()
            .utimens(last_access, last_modification);
    }

    pub fn rename(&self, from: &PathComponent, to: &PathComponent) -> FsResult<()> {
        self.inode.lock().unwrap().rename(from, to)
    }

    // TODO Don't let `Arc` escape here, rather return `&Mutex`
    pub(super) fn inode(&self) -> &Arc<Mutex<DirInode>> {
        &self.inode
    }
}

#[async_trait]
impl Dir for InMemoryDirRef {
    type Device = InMemoryDevice;

    fn as_node(&self) -> AsyncDropGuard<InMemoryNodeRef> {
        AsyncDropGuard::new(InMemoryNodeRef::Dir(self.clone_ref()))
    }

    async fn lookup_child(
        &self,
        name: &PathComponent,
    ) -> FsResult<AsyncDropGuard<InMemoryNodeRef>> {
        Ok(AsyncDropGuard::new(self.get_child(name)?))
    }

    async fn entries(&self) -> FsResult<Vec<DirEntry>> {
        let inode = self.inode.lock().unwrap();
        let entries = inode.entries().iter().map(|(name, node)| {
            let kind: NodeKind = match node {
                InMemoryNodeRef::File(_) => NodeKind::File,
                InMemoryNodeRef::Dir(_) => NodeKind::Dir,
                InMemoryNodeRef::Symlink(_) => NodeKind::Symlink,
            };
            DirEntry {
                name: name.clone(),
                kind,
            }
        });
        Ok(entries.collect())
    }

    async fn create_child_dir(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let mut inode = self.inode.lock().unwrap();
        let dir = InMemoryDirRef::new(mode, uid, gid);
        let metadata = dir.metadata();
        // TODO Use try_insert once that is stable
        match inode.entries_mut().entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNodeRef::Dir(dir));
            }
        }
        Ok(metadata)
    }

    async fn remove_child_dir(&self, name: &PathComponent) -> FsResult<()> {
        let mut inode = self.inode.lock().unwrap();
        // TODO Use try_insert once that is stable
        match inode.entries_mut().entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(entry) => match entry.get() {
                InMemoryNodeRef::File(_) | InMemoryNodeRef::Symlink(_) => {
                    return Err(FsError::NodeIsNotADirectory);
                }
                InMemoryNodeRef::Dir(_) => {
                    entry.remove();
                    Ok(())
                }
            },
            std::collections::hash_map::Entry::Vacant(_) => Err(FsError::NodeDoesNotExist),
        }
    }

    async fn create_child_symlink(
        &self,
        name: &PathComponent,
        target: &str,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<NodeAttrs> {
        let mut inode = self.inode.lock().unwrap();
        let symlink = InMemorySymlinkRef::new(target.to_owned(), uid, gid);
        let metadata = symlink.metadata();
        // TODO Use try_insert once that is stable
        match inode.entries_mut().entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNodeRef::Symlink(symlink));
            }
        }
        Ok(metadata)
    }

    async fn remove_child_file_or_symlink(&self, name: &PathComponent) -> FsResult<()> {
        let mut inode = self.inode.lock().unwrap();
        // TODO Use try_insert once that is stable
        match inode.entries_mut().entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(entry) => match entry.get() {
                InMemoryNodeRef::File(_) | InMemoryNodeRef::Symlink(_) => {
                    entry.remove();
                    Ok(())
                }
                InMemoryNodeRef::Dir(_) => return Err(FsError::NodeIsADirectory),
            },
            std::collections::hash_map::Entry::Vacant(_) => Err(FsError::NodeDoesNotExist),
        }
    }

    async fn create_and_open_file(
        &self,
        name: &PathComponent,
        mode: Mode,
        uid: Uid,
        gid: Gid,
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<InMemoryOpenFileRef>)> {
        let mut inode = self.inode.lock().unwrap();
        let file = InMemoryFileRef::new(mode, uid, gid);
        let openfile = file.open_sync(OpenFlags::ReadWrite);
        let metadata = file.metadata();
        // TODO Use try_insert once that is stable
        match inode.entries_mut().entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNodeRef::File(file));
            }
        }
        Ok((metadata, openfile))
    }
}

impl Debug for InMemoryDirRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let inode = self.inode.lock().unwrap();
        f.debug_struct("InMemoryDirRef").finish()
    }
}
