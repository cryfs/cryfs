// TODO Will lookup() be called multiple times with the same parent+name, before the previous one is forgotten, and is it ok to give the second call a different inode while the first call is still ongoing?
//      Seems not, `cp -R ~/mountdir/.cargo ~/.cargo.copy` (after `cp -R ~/.cargo ~/mountdir/`) is complaining when inodes change between stat and opening the file, see https://github.com/coreutils/coreutils/blob/b8675fe98cc38e9a49afec52f4102d330d9aaa27/src/copy.c#L775
//      Note that fuse-mt also increases lookup count and ensures that inodes for the same path stay consistent until the inode is forgotten
//      Main differences to fuse-mt:
//       * keeps path -> inode reverse mapping and reuses inode assignments if they're looked up again. Remembers lookup count and only frees inode if it goes to zero
//         * at unlink time, it removes the reverse map but keeps the inode around until it's forgotten. Need to think about why.
//         * also needs to deal with rename since the reverse mapping changes
//         * but we may have to do it in a tree maybe? because we don't have a path mapping but parent_ino+name mapping.
//      Most likely, we'll have to:
//       * keep a ino -> (parent_ino, node_info, refcount, children) mapping
//         * children: name->ino mapping for each loaded inode that is a direct child
//       * forget entries from the mapping only if refcount goes to zero AND children is empty (i.e. they are all unloaded)
//       * when forgetting an entry, also remove it from its parent's children mapping
//       * update name mapping on rename
//       * remove entry from `children` mapping of its parent ino on unlink/rmdir, but keep the inode itself in existence if refcount > 0
//          * see also https://github.com/wfraser/fuse-mt/issues/48 for rmdir
//       * Invariant: If an inode exists in the children mapping, it also exists in the main mapping.
//                    If an inode exists in the main mapping with refcount > 0, it may or may not exist in the children mapping, depending on whether it was deleted.

use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Mutex;

use cryfs_utils::async_drop::AsyncDropGuard;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc};

use crate::FsResult;
use crate::PathComponent;
use crate::common::HandleWithGeneration;
use crate::{FsError, object_based_api::Device};
use crate::{InodeNumber, common::HandleMap};

pub const FUSE_ROOT_ID: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID);
pub const DUMMY_INO: InodeNumber = InodeNumber::from_const(fuser::FUSE_ROOT_ID + 1);

mod inode_info;
use inode_info::InodeInfo;

/// [InodeList] keeps track of all inodes that have been registered with FUSE
pub struct InodeList<Fs>
where
    Fs: Device + Debug,
{
    // Invariants:
    // * inodes always contains an entry for FUSE_ROOT_ID (established by calling insert_rootdir() after new())
    inodes: Mutex<AsyncDropGuard<HandleMap<InodeNumber, InodeInfo<Fs>>>>,
}

impl<Fs> InodeList<Fs>
where
    Fs: Device + Debug,
{
    /// After calling [Self::new], and before calling anything else, you must also call [Self::insert_rootdir] to establish the invariants and get a usable [InodeList].
    pub fn new() -> AsyncDropGuard<Self> {
        let mut inodes = HandleMap::new();

        Self::block_invalid_handles(&mut inodes);
        AsyncDropGuard::new(Self {
            inodes: Mutex::new(inodes),
        })
    }

    fn block_invalid_handles(inodes: &mut HandleMap<InodeNumber, InodeInfo<Fs>>) {
        // We need to block zero because fuse seems to dislike it.
        if fuser::FUSE_ROOT_ID != 0 {
            inodes.block_handle(InodeNumber::from(0));
        }
        inodes.block_handle(DUMMY_INO);
    }

    pub fn insert_rootdir(&self, rootdir: AsyncDropGuard<Fs::Node>) {
        let mut inodes = self.inodes.lock().unwrap();
        inodes.insert(
            FUSE_ROOT_ID,
            InodeInfo::new(
                AsyncDropArc::new(rootdir),
                // The root dir is its own parent
                FUSE_ROOT_ID,
            ),
        );
    }

    pub async fn get_node_and_parent_ino(
        &self,
        ino: InodeNumber,
    ) -> Option<(AsyncDropGuard<AsyncDropArc<Fs::Node>>, InodeNumber)> {
        let inodes = self.inodes.lock().unwrap();
        let inode = inodes.get(ino)?;
        Some((inode.node(), inode.parent_inode()))
    }

    pub async fn add(
        &self,
        parent_ino: InodeNumber,
        node: AsyncDropGuard<Fs::Node>,
        name: &PathComponent,
    ) -> HandleWithGeneration<InodeNumber> {
        let child_ino = self
            .inodes
            .lock()
            .unwrap()
            .add(InodeInfo::new(AsyncDropArc::new(node), parent_ino));
        log::info!("New inode {child_ino:?}: parent={parent_ino:?}, name={name}");
        child_ino
    }

    pub async fn remove(&self, ino: InodeNumber) -> FsResult<()> {
        assert!(ino != FUSE_ROOT_ID, "Cannot remove root inode");
        let mut entry = self.inodes.lock().unwrap().remove(ino);
        entry.async_drop().await?;
        Ok(())
    }

    #[cfg(feature = "testutils")]
    pub async fn fsync_and_clear_all(&self) -> FsResult<()> {
        let mut inodes = self.inodes.lock().unwrap();
        let root_inode = inodes.try_remove(FUSE_ROOT_ID);
        for (_handle, mut object) in inodes.drain() {
            object.fsync().await.unwrap();
            object.async_drop().await.unwrap();
        }
        // Re-add root inode so the InodeList is still usable after this call
        if let Some(root_inode) = root_inode {
            inodes.insert(FUSE_ROOT_ID, root_inode);
        }
        Self::block_invalid_handles(&mut inodes);
        Ok(())
    }

    #[cfg(feature = "testutils")]
    pub async fn clear_all(&self) -> FsResult<()> {
        let mut inodes = self.inodes.lock().unwrap();
        let root_inode = inodes.try_remove(FUSE_ROOT_ID);
        for (_handle, mut object) in inodes.drain() {
            object.async_drop().await.unwrap();
        }
        // Re-add root inode so the InodeList is still usable after this call
        if let Some(root_inode) = root_inode {
            inodes.insert(FUSE_ROOT_ID, root_inode);
        }
        Self::block_invalid_handles(&mut inodes);
        Ok(())
    }

    #[cfg(feature = "testutils")]
    pub async fn fsync_all(&self) -> FsResult<()> {
        let inodes = self.inodes.lock().unwrap();
        for (_handle, object) in inodes.iter() {
            object.fsync().await.unwrap();
        }
        Ok(())
    }
}

impl<Fs> Debug for InodeList<Fs>
where
    Fs: Device + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InodeList").finish()
    }
}

#[async_trait]
impl<Fs> AsyncDrop for InodeList<Fs>
where
    Fs: Device + Debug,
{
    type Error = FsError;
    async fn async_drop_impl(&mut self) -> FsResult<()> {
        let mut inodes = std::mem::replace(
            &mut *self.inodes.lock().unwrap(),
            AsyncDropGuard::new_invalid(),
        );
        inodes.async_drop().await
    }
}
