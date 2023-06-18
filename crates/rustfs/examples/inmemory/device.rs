use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use cryfs_rustfs::{
    object_based_api::Device, AbsolutePath, FsError, FsResult, Gid, Mode, Statfs, Uid,
};
use cryfs_utils::async_drop::AsyncDropGuard;

use super::dir::{DirInode, InMemoryDirRef};
use super::file::{InMemoryFileRef, InMemoryOpenFileRef};
use super::node::InMemoryNodeRef;
use super::symlink::InMemorySymlinkRef;
use crate::utils::lock_in_ptr_order;

pub struct RootDir {
    // We're pointing directly to the [DirInode] instead of using an [InMemoryDirRef] to avoid
    // reference cycles. [InMemoryDirRef] has a reference back to [RootDir].
    rootdir: Arc<Mutex<DirInode>>,
}

impl RootDir {
    pub fn new(uid: Uid, gid: Gid) -> Arc<Self> {
        let mode = Mode::default()
            .add_dir_flag()
            .add_user_read_flag()
            .add_user_write_flag()
            .add_user_exec_flag();
        Arc::new(Self {
            rootdir: Arc::new(Mutex::new(DirInode::new(mode, uid, gid))),
        })
    }

    pub fn load_node(self: &Arc<Self>, path: &AbsolutePath) -> FsResult<InMemoryNodeRef> {
        let mut current_node =
            InMemoryNodeRef::Dir(InMemoryDirRef::from_inode(Arc::clone(&self.rootdir)));
        for component in path.iter() {
            match &current_node {
                InMemoryNodeRef::Dir(dir) => {
                    current_node = dir.get_child(&component)?;
                }
                InMemoryNodeRef::Symlink(_) | InMemoryNodeRef::File(_) => {
                    return Err(FsError::NodeIsNotADirectory);
                }
            }
        }
        Ok(current_node)
    }

    pub fn load_dir(self: &Arc<Self>, path: &AbsolutePath) -> FsResult<InMemoryDirRef> {
        let node = self.load_node(path)?;
        match node {
            InMemoryNodeRef::Dir(dir) => Ok(dir),
            _ => Err(FsError::NodeIsNotADirectory),
        }
    }

    pub fn load_symlink(self: &Arc<Self>, path: &AbsolutePath) -> FsResult<InMemorySymlinkRef> {
        let node = self.load_node(path)?;
        match node {
            InMemoryNodeRef::Symlink(symlink) => Ok(symlink),
            _ => Err(FsError::NodeIsNotASymlink),
        }
    }

    pub fn load_file(
        self: &Arc<Self>,
        path: &AbsolutePath,
    ) -> FsResult<AsyncDropGuard<InMemoryFileRef>> {
        let node = self.load_node(path)?;
        match node {
            InMemoryNodeRef::File(file) => Ok(AsyncDropGuard::new(file)),
            InMemoryNodeRef::Dir(_) => Err(FsError::NodeIsADirectory),
            InMemoryNodeRef::Symlink(_) => {
                // TODO What's the right error here?
                Err(FsError::UnknownError)
            }
        }
    }
}

pub struct InMemoryDevice {
    rootdir: Arc<RootDir>,
}

impl InMemoryDevice {
    pub fn new(uid: Uid, gid: Gid) -> Self {
        Self {
            rootdir: RootDir::new(uid, gid),
        }
    }
}

#[async_trait]
impl Device for InMemoryDevice {
    type Node<'a> = InMemoryNodeRef;
    type Dir<'a> = InMemoryDirRef;
    type Symlink<'a> = InMemorySymlinkRef;
    type File<'a> = InMemoryFileRef;
    type OpenFile = InMemoryOpenFileRef;

    async fn load_node(&self, path: &AbsolutePath) -> FsResult<Self::Node<'_>> {
        self.rootdir.load_node(path)
    }

    async fn load_dir(&self, path: &AbsolutePath) -> FsResult<Self::Dir<'_>> {
        self.rootdir.load_dir(path)
    }

    async fn load_symlink(&self, path: &AbsolutePath) -> FsResult<Self::Symlink<'_>> {
        self.rootdir.load_symlink(path)
    }

    async fn load_file(&self, path: &AbsolutePath) -> FsResult<AsyncDropGuard<Self::File<'_>>> {
        self.rootdir.load_file(path)
    }

    async fn rename(&self, from_path: &AbsolutePath, to_path: &AbsolutePath) -> FsResult<()> {
        // TODO Go through CryNode assertions (C++) and check if we should do them here too,
        //      - moving a directory into a subdirectory of itself
        //      - overwriting a directory with a non-directory
        //      - overwriten a non-empty dir (special case: making a directory into its own ancestor)
        // TODO No unwrap
        let Some((new_parent_path, new_name)) = to_path.split_last() else {
            log::error!("Tried to rename '{from_path}' to the root directory");
            return Err(FsError::InvalidOperation);
        };
        let Some((old_parent_path, old_name)) = from_path.split_last() else {
            log::error!("Tried to rename the root directory to {to_path}");
            return Err(FsError::InvalidOperation);
        };
        let new_parent = self.rootdir.load_dir(new_parent_path)?;
        if old_parent_path == new_parent_path {
            // We're just renaming it within one directory
            new_parent.rename(old_name, new_name)
        } else {
            let source_parent = self.rootdir.load_dir(old_parent_path)?;

            // We're moving it to another directory
            let (mut source_inode, mut target_inode) =
                lock_in_ptr_order(&source_parent.inode(), &new_parent.inode());
            let source_entries = source_inode.entries_mut();
            let target_entries = target_inode.entries_mut();
            if target_entries.contains_key(new_name) {
                // TODO Some forms of overwriting are actually ok, we don't need to block them all
                Err(FsError::NodeAlreadyExists)
            } else {
                let old_entry = match source_entries.remove(old_name) {
                    Some(node) => node,
                    None => {
                        return Err(FsError::NodeDoesNotExist);
                    }
                };
                // TODO Use try_insert once stable
                let insert_result = target_entries.insert(new_name.to_owned(), old_entry);
                assert!(insert_result.is_none(), "We checked above that `new_name` doesn't exist in the map. Inserting it shouldn't fail.");
                Ok(())
            }
        }
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        todo!()
    }

    async fn destroy(self) {
        // Nothing to do
    }
}
