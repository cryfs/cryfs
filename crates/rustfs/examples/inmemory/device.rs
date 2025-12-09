use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};

use cryfs_rustfs::{
    FsError, FsResult, Gid, Mode, Statfs, Uid,
    object_based_api::{Device, Node},
};
use cryfs_utils::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    mutex::lock_in_ptr_order,
    path::AbsolutePath,
    with_async_drop_2,
};

use super::dir::{DirInode, InMemoryDirRef};
use super::file::{InMemoryFileRef, InMemoryOpenFileRef};
use super::node::InMemoryNodeRef;
use super::symlink::InMemorySymlinkRef;

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

    fn _node(&self) -> InMemoryDirRef {
        InMemoryDirRef::from_inode(Arc::clone(&self.rootdir))
    }

    pub fn load_node(
        self: &Arc<Self>,
        path: &AbsolutePath,
    ) -> FsResult<AsyncDropGuard<InMemoryNodeRef>> {
        let mut current_node = InMemoryNodeRef::Dir(self._node());
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
        Ok(AsyncDropGuard::new(current_node))
    }
}

pub struct InMemoryDevice {
    rootdir: Arc<RootDir>,
}

impl InMemoryDevice {
    pub fn new(uid: Uid, gid: Gid) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            rootdir: RootDir::new(uid, gid),
        })
    }
}

impl Debug for InMemoryDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryDevice").finish()
    }
}

#[async_trait]
impl AsyncDrop for InMemoryDevice {
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // Nothing to do
        Ok(())
    }
}

#[async_trait]
impl Device for InMemoryDevice {
    type Node = InMemoryNodeRef;
    type Dir<'a> = InMemoryDirRef;
    type Symlink<'a> = InMemorySymlinkRef;
    type File<'a> = InMemoryFileRef;
    type OpenFile = InMemoryOpenFileRef;

    async fn rootdir(&self) -> FsResult<AsyncDropGuard<InMemoryDirRef>> {
        Ok(AsyncDropGuard::new(self.rootdir._node()))
    }

    fn rename(
        &self,
        from_path: &AbsolutePath,
        to_path: &AbsolutePath,
    ) -> impl Future<Output = FsResult<()>> {
        async move {
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
            let new_parent = self.rootdir.load_node(new_parent_path)?;
            with_async_drop_2!(new_parent, {
                let new_parent = new_parent.as_dir().await?;
                with_async_drop_2!(new_parent, {
                    if old_parent_path == new_parent_path {
                        // We're just renaming it within one directory
                        new_parent.rename(old_name, new_name)
                    } else {
                        let source_parent = self.rootdir.load_node(old_parent_path)?;
                        with_async_drop_2!(source_parent, {
                            let source_parent = source_parent.as_dir().await?;
                            with_async_drop_2!(source_parent, {
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
                                    let insert_result =
                                        target_entries.insert(new_name.to_owned(), old_entry);
                                    assert!(
                                        insert_result.is_none(),
                                        "We checked above that `new_name` doesn't exist in the map. Inserting it shouldn't fail."
                                    );
                                    Ok(())
                                }
                            })
                        })
                    }
                })
            })
        }
    }

    async fn statfs(&self) -> FsResult<Statfs> {
        todo!()
    }
}
