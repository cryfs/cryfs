use async_trait::async_trait;
use cryfs_rustfs::{
    DirEntry, FsError, FsResult, Gid, Mode, NodeAttrs, NodeKind, NumBytes, OpenFlags,
    PathComponent, PathComponentBuf, Uid, object_based_api::Dir,
};
use cryfs_utils::async_drop::AsyncDrop;
use cryfs_utils::with_async_drop_2;
use cryfs_utils::{async_drop::AsyncDropGuard, mutex::lock_in_ptr_order};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use super::device::InMemoryDevice;
use super::file::InMemoryFileRef;
use super::file::InMemoryOpenFileRef;
use super::inode_metadata::setattr;
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

        pub fn setattr(
            &mut self,
            mode: Option<Mode>,
            uid: Option<Uid>,
            gid: Option<Gid>,
            atime: Option<SystemTime>,
            mtime: Option<SystemTime>,
            ctime: Option<SystemTime>,
        ) -> FsResult<NodeAttrs> {
            setattr(&mut self.metadata, mode, uid, gid, atime, mtime, ctime)
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
                assert!(
                    insert_result.is_none(),
                    "We checked above that `new_name` doesn't exist in the map. Inserting it shouldn't fail."
                );
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

    pub fn setattr(
        &self,
        mode: Option<Mode>,
        uid: Option<Uid>,
        gid: Option<Gid>,
        size: Option<NumBytes>,
        atime: Option<SystemTime>,
        mtime: Option<SystemTime>,
        ctime: Option<SystemTime>,
    ) -> FsResult<NodeAttrs> {
        // TODO Is setting size actually forbidden?
        assert!(
            size.is_none(),
            "Can't set size using setattr on a directory"
        );
        self.inode
            .lock()
            .unwrap()
            .setattr(mode, uid, gid, atime, mtime, ctime)
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

    fn into_node(this: AsyncDropGuard<Self>) -> AsyncDropGuard<InMemoryNodeRef> {
        AsyncDropGuard::new(InMemoryNodeRef::Dir(
            this.unsafe_into_inner_dont_drop().clone_ref(),
        ))
    }

    async fn lookup_child(
        &self,
        name: &PathComponent,
    ) -> FsResult<AsyncDropGuard<InMemoryNodeRef>> {
        Ok(AsyncDropGuard::new(self.get_child(name)?))
    }

    async fn rename_child(&self, oldname: &PathComponent, newname: &PathComponent) -> FsResult<()> {
        let mut inode = self.inode.lock().unwrap();
        inode.rename(oldname, newname)
    }

    async fn move_child_to(
        &self,
        oldname: &PathComponent,
        newparent: AsyncDropGuard<Self>,
        newname: &PathComponent,
    ) -> FsResult<()> {
        with_async_drop_2!(newparent, {
            // We're moving it to another directory
            let (mut source_inode, mut target_inode) =
                lock_in_ptr_order(&self.inode(), &newparent.inode());
            let source_entries = source_inode.entries_mut();
            let target_entries = target_inode.entries_mut();
            if target_entries.contains_key(newname) {
                // TODO Some forms of overwriting are actually ok, we don't need to block them all
                Err(FsError::NodeAlreadyExists)
            } else {
                let old_entry = match source_entries.remove(oldname) {
                    Some(node) => node,
                    None => {
                        return Err(FsError::NodeDoesNotExist);
                    }
                };
                // TODO Use try_insert once stable
                let insert_result = target_entries.insert(newname.to_owned(), old_entry);
                assert!(
                    insert_result.is_none(),
                    "We checked above that `new_name` doesn't exist in the map. Inserting it shouldn't fail."
                );
                Ok(())
            }
        })
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
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<InMemoryDirRef>)> {
        let mut inode = self.inode.lock().unwrap();
        let dir = InMemoryDirRef::new(mode, uid, gid);
        let metadata = dir.metadata();
        // TODO Use try_insert once that is stable
        match inode.entries_mut().entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNodeRef::Dir(dir.clone_ref()));
            }
        }
        Ok((metadata, AsyncDropGuard::new(dir)))
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
    ) -> FsResult<(NodeAttrs, AsyncDropGuard<InMemorySymlinkRef>)> {
        let mut inode = self.inode.lock().unwrap();
        let symlink = InMemorySymlinkRef::new(target.to_owned(), uid, gid);
        let metadata = symlink.metadata();
        // TODO Use try_insert once that is stable
        match inode.entries_mut().entry(name.to_owned()) {
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err(FsError::NodeAlreadyExists);
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(InMemoryNodeRef::Symlink(symlink.clone_ref()));
            }
        }
        Ok((metadata, AsyncDropGuard::new(symlink)))
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
    ) -> FsResult<(
        NodeAttrs,
        AsyncDropGuard<InMemoryNodeRef>,
        AsyncDropGuard<InMemoryOpenFileRef>,
    )> {
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
                entry.insert(InMemoryNodeRef::File(file.clone_ref()));
            }
        }
        Ok((metadata, file.as_node(), openfile))
    }

    async fn fsync(&self, _datasync: bool) -> FsResult<()> {
        // No need to fsync because we're in-memory
        Ok(())
    }
}

impl Debug for InMemoryDirRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryDirRef").finish()
    }
}

#[async_trait]
impl AsyncDrop for InMemoryDirRef {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        // Nothing to do
        Ok(())
    }
}
