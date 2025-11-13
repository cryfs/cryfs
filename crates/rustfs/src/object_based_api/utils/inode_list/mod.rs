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
//       * Invariant: If an inode exists in the children mapping, it also exists in the main mapping.
//                    If an inode exists in the main mapping with refcount > 0, it may or may not exist in the children mapping, depending on whether it was deleted.

use async_trait::async_trait;
use cryfs_utils::concurrent_store::{
    ConcurrentStore, LoadedEntryGuard, RequestImmediateDropResult,
};
use cryfs_utils::stream::for_each_unordered;
use cryfs_utils::with_async_drop_2;
use futures::future::BoxFuture;
use futures::{FutureExt, future};
use itertools::multiunzip;
use lockable::InfallibleUnwrap as _;
use std::fmt::Debug;
use std::num::NonZeroU64;
use tokio::sync::{Mutex, MutexGuard};

use cryfs_utils::async_drop::AsyncDropGuard;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropResult, AsyncDropShared};

use crate::FsResult;
use crate::InodeNumber;
use crate::PathComponentBuf;
use crate::common::HandleWithGeneration;
use crate::object_based_api::utils::inode_list::handle_forest::{
    DelayedHandleRelease, GetChildOfError, HandleForest, TryInsertError2, TryRemoveResult,
};
use crate::object_based_api::utils::inode_list::inode_info::RefcountInfo;
use crate::{FsError, object_based_api::Device};

pub const FUSE_ROOT_ID: InodeNumber =
    InodeNumber::from_const(NonZeroU64::new(fuser::FUSE_ROOT_ID).unwrap());
pub const DUMMY_INO: InodeNumber =
    InodeNumber::from_const(NonZeroU64::new(fuser::FUSE_ROOT_ID + 1).unwrap());

mod handle_forest;

mod inode_info;
use inode_info::InodeInfo;

const PARENT_OF_ROOT_INO: InodeNumber = FUSE_ROOT_ID; // Root inode's parent is itself

/// [InodeList] keeps track of all inodes that have been registered with FUSE
pub struct InodeList<Fs>
where
    Fs: Device + Debug + 'static,
    Fs::Node: 'static,
{
    inner: Mutex<InodeListInner<Fs>>, // TODO std::sync::Mutex instead of tokio?
}

/// [InodeList] keeps track of all inodes that have been registered with FUSE
struct InodeListInner<Fs>
where
    Fs: Device + Debug + 'static,
    Fs::Node: 'static,
{
    // This is a tree of inodes (or rather forest, see below), each node has a parent pointer and a map from path components to child inode numbers,
    // which can then be used to index back into this map to get the corresponding parent or child InodeInfo.
    //
    // Invariants:
    // * A: inodes and inode_forest always contains an entry for FUSE_ROOT_ID (established by calling insert_rootdir() after new())
    // * B: Loading status
    //    * B1: Any InodeNumber we've given out is already fully loaded in `inodes` and loading was successful. Otherwise, we wouldn't have given out the InodeNumber.
    //    * B2: All of its ancestors up to the root in `inode_forest` are also fully loaded in `inodes` and loading was successful.
    // * C: Eventually, inode numbers are either present in both `inodes` and `inode_forest`, or neither. But there can be temporary exceptions:
    //      * C1: Entry in `inode_forest` but not in `inodes`:
    //        When loading failed, ConcurrentStore automatically removes the entry from `inodes`, but we may still have an entry in `inode_forest`.
    //        In this case, the node future of the entry in `inode_forest` will resolve to an error.
    //        The loading task will eventually clean this up and remove the entry from `inode_forest` as well when it notices the error,
    //        but it'll have to re-lock `inode_forest` to do so, so there may be a short time window where the entry still exists in `inode_forest` but not in `inodes`.
    //        However, their InodeNumbers are blocked from being reused until the drop completes, see invariant D.
    //      * C2: Entry in `inodes` but not in `inode_forest`:
    //        When dropping an inode because its refcount went to zero and it has no children, we first remove it from `inode_forest` to ensure invariant C,
    //        but `inodes` will asynchronously drop the entry later, so there may be a short time window where the entry still exists in `inodes` but not in `inode_forest`.
    //        However, their InodeNumbers are blocked from being reused until the drop completes, see invariant D.
    // * D: All inodes that are either in `inodes` or `inode_forest` are blocked in `inode_forest` from being re-used.
    //     * Note: For entries in `inode_forest`, this is ensured by HandleForest. So we only need to ensure t for entries in `inodes` that are not in `inode_forest` (see invariant C2).
    //       This is done by DelayedHandleRelease when dropping such entries.
    // * E: All entries in `inode_forest` have children or a refcount > 0.
    // * F: Each entry in self.inodes has at most one guard active, which is stored in self.inode_forest.

    // TODO Currently, each call site of ConcurrentStore has to wrap it into an AsyncDropArc themselves. Can we make ConcurrentStore do that internally, simplifying call sites?
    // TODO Node here holds a reference to the ConcurrentFsBlob, which blocks the blob from being removed. This would be a deadlock in unlink/rmdir if we store a reference to the self blob in NodeInfo.
    //      Right now, we only store a reference to the parent blob and that's fine because child inodes are forgotten before the parent can be removed.
    inodes: AsyncDropGuard<AsyncDropArc<ConcurrentStore<InodeNumber, Fs::Node, FsError>>>,

    // All the inode numbers we've given to the kernel, with a corresponding refcount. These references ensure that the inodes are being kept alive in self.inodes.
    // On top of this refcount, each InodeInfo also remmbers its parent InodeNumber. Overall, we guarantee that inodes only get removed once the kernel has
    // released it and all its children.
    // TODO Use Vec or slab instead of HashMap since InodeNumber is mostly contiguous?
    inode_forest: AsyncDropGuard<HandleForest<InodeNumber, PathComponentBuf, InodeInfo<Fs>>>,
}

impl<Fs> InodeList<Fs>
where
    Fs: Device + Debug + 'static,
    Fs::Node: 'static,
{
    /// After calling [Self::new], and before calling anything else, you must also call [Self::insert_rootdir] to establish the invariants and get a usable [InodeList].
    pub fn new() -> AsyncDropGuard<Self> {
        let mut inode_forest = HandleForest::new();
        let inodes = ConcurrentStore::new();

        Self::block_invalid_handles(&mut inode_forest);
        AsyncDropGuard::new(Self {
            inner: Mutex::new(InodeListInner {
                inodes: AsyncDropArc::new(inodes),
                inode_forest,
            }),
        })
        // Fulfilling invariants:
        //  * A not fulfilled yet, [Self::insert_rootdir] must be called first.
        //  * B, C, D, E, F trivially fulfilled
    }

    fn block_invalid_handles(
        inode_forest: &mut HandleForest<InodeNumber, PathComponentBuf, InodeInfo<Fs>>,
    ) {
        // We don't need to block zero because we use NonZeroU64 in the handle, so it can't be represented anyways
        inode_forest.block_handle(DUMMY_INO);
    }

    // TODO Can we instead pass in the rootdir to [Self::new], getting a better protection for our invariant?
    pub async fn insert_rootdir(&self, rootdir: AsyncDropGuard<Fs::Node>) {
        let mut inner = self.inner.lock().await;
        self._insert_rootdir(&mut inner, rootdir).await;
        // Fulfilling invariants:
        // * A: Now contains an entry for FUSE_ROOT_ID
        // * B: Root inode is fully loaded
        // * C: Root inode is present in both inodes and inode_forest
        // * D: Root inode is blocked from being reused in inode_forest
        // * E: Root inode has a refcount > 0
        // * F: Root inode has exactly one guard active in inode_forest
    }

    async fn _insert_rootdir(
        &self,
        inner: &mut MutexGuard<'_, InodeListInner<Fs>>,
        rootdir: AsyncDropGuard<Fs::Node>,
    ) {
        let inserted = ConcurrentStore::try_insert_loaded(&inner.inodes, FUSE_ROOT_ID, rootdir)
            .await
            .expect("Root dir entry already exists");
        inner
            .inode_forest
            .try_insert_root_with_specific_handle(
                FUSE_ROOT_ID,
                InodeInfo::new(AsyncDropShared::new(
                    future::ready(AsyncDropResult::new(Ok(inserted))).boxed(),
                )),
            )
            .expect("Failed to insert rootdir because it already exists in the forest");
    }

    fn _lookup_node(
        inner: &MutexGuard<'_, InodeListInner<Fs>>,
        ino: InodeNumber,
    ) -> FsResult<AsyncDropGuard<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>>> {
        ConcurrentStore::get_if_loading_or_loaded(&inner.inodes, ino)
            .wait_until_loaded()
            .now_or_never()
            .ok_or_else(|| {
                // Invariant B violated, but this can happen if the kernel gives us wrong inode numbers, so treating it as InvalidOperation
                log::error!("Tried to load inode but inode number {ino:?} is still loading");
                FsError::InvalidOperation
            })??
            .ok_or_else(|| {
                log::error!("Tried to load inode but inode number {ino:?} isn't assigned");
                FsError::InvalidOperation
            })
    }

    pub async fn get_node_and_parent_ino(
        &self,
        ino: InodeNumber,
    ) -> FsResult<(AsyncDropGuard<AsyncDropArc<Fs::Node>>, InodeNumber)> {
        let mut inner = self.inner.lock().await;
        let inode_tree_node = inner.inode_forest.get(&ino).ok_or_else(|| {
            log::error!("Tried to get inode info for unknown inode {ino:?}");
            FsError::InvalidOperation
        })?;
        let parent_ino = inode_tree_node
            .parent_handle()
            .copied()
            .unwrap_or(PARENT_OF_ROOT_INO);
        let node = Self::_get_node(&mut inner, ino).await?;
        // Fulfilling Invariant B: The user gave us `ino`, i.e. we handed it out before and know with B2 that its ancestors are also loaded.
        Ok((node, parent_ino))
    }

    pub async fn get_node(
        &self,
        ino: InodeNumber,
    ) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        let inner = self.inner.lock().await;
        Self::_get_node(&inner, ino).await
    }

    async fn _get_node(
        inner: &MutexGuard<'_, InodeListInner<Fs>>,
        ino: InodeNumber,
    ) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        let node = Self::_lookup_node(inner, ino)?;
        with_async_drop_2!(node, {
            let inode_entry = AsyncDropArc::clone(node.value());
            Ok(inode_entry)
        })
    }

    pub async fn add(
        &self,
        parent_ino: InodeNumber,
        node: AsyncDropGuard<Fs::Node>,
        name: PathComponentBuf,
    ) -> FsResult<HandleWithGeneration<InodeNumber>> {
        let mut inner = self.inner.lock().await;
        let name_clone = name.clone();

        // TODO This Arc::clone is only necessary because MutexGuard can't project and get &mut on both inner.inodes and inner.inode_forest at the same time. Once Rust supports that, we can avoid this clone.
        let inodes = AsyncDropArc::clone(&inner.inodes);
        with_async_drop_2!(inodes, {
            let insert_result = inner
                .inode_forest
                .try_insert(parent_ino, name, async |new_child_ino| {
                    let inserted_node = ConcurrentStore::<
                            InodeNumber,
                            <Fs as Device>::Node,
                            FsError,
                        >::try_insert_loaded(
                            &inodes, new_child_ino.handle, node
                        )
                        // TODO Remove this await. It only triggers async code if the inode happens to be dropping at this exact moment,
                        //      but even so it is better to not wait for it here while having a lock on inner.
                        .await
                        .expect("Invariant D violated: A new (i.e. not blocked) inode number was already in use.");
                    InodeInfo::new(AsyncDropShared::new(
                        future::ready(AsyncDropResult::new(Ok(inserted_node))).boxed(),
                    ))
                })
                .await;

            match insert_result {
                Err(TryInsertError2::ParentNotFound) => {
                    log::error!("Tried to add inode under unknown parent inode {parent_ino:?}");
                    return Err(FsError::InvalidOperation);
                }
                Err(TryInsertError2::AlreadyExists) => {
                    log::error!(
                        "Tried to add already existng inode {name_clone} under parent inode {parent_ino:?}"
                    );
                    return Err(FsError::NodeAlreadyExists);
                }
                Ok((new_child_ino, _new_node)) => {
                    log::info!(
                        "New inode {new_child_ino:?}: parent={parent_ino:?}, name={name_clone}"
                    );
                    Ok(new_child_ino)
                }
            }
        })

        // Fulfilling invariants:
        // * A: No change here
        // * B1: We added an already fully loaded node
        // * B2: The user gave us parent_ino, so by invariant B1+B2, all its ancestors are also fully loaded.
        // * C: self.inode_forest.try_insert() guarantees that the lambda creating the inode in self.inodes is executed
        //      if and only if the node is inserted to self.inode_tree. The entry is either added to both or neither.
        // * D: If we added it to self.inode_forest, HandleForest keeps it blocked. If we didn't add it to inode_forest,
        //      we also didn't add it to self.inodes (see invariant C in previous point), so no need to block it.
        // * E: The new entry got a refcount of 1.
        // * F: The new entry has exactly one guard active in inode_forest.
    }

    pub async fn add_or_increment_refcount<F>(
        &self,
        parent_ino: InodeNumber,
        name: PathComponentBuf,
        loading_fn: impl FnOnce(&AsyncDropGuard<AsyncDropArc<Fs::Node>>) -> F + Send + 'static,
    ) -> FsResult<(
        HandleWithGeneration<InodeNumber>,
        AsyncDropGuard<AsyncDropArc<Fs::Node>>,
    )>
    where
        F: Future<Output = FsResult<AsyncDropGuard<Fs::Node>>> + Send,
    {
        let mut inner = self.inner.lock().await;

        match inner.inode_forest.get_child_of_mut(&parent_ino, &name) {
            Err(GetChildOfError::ParentNotFound) => {
                // Invariant B violated, but this can happen if the kernel gives us wrong inode numbers, so treating it as InvalidOperation
                log::error!(
                    "Tried to add/increment inode under unknown parent inode {parent_ino:?}"
                );
                Err(FsError::InvalidOperation)
            }
            Err(GetChildOfError::ChildNotFound) => {
                // Child doesn't exist yet, create it

                let name_clone = name.clone();
                let (child_ino, node) = self._add_new(inner, parent_ino, name, loading_fn).await?;
                log::info!("New inode: parent={parent_ino:?}, name={name_clone}");
                Ok((child_ino, node))

                // Fulfilling invariants: See comments in [Self::_add_new]
            }
            Ok((child_ino, child_inode)) => {
                // Child already exists, increment its refcount and return it

                child_inode.value_mut().increment_refcount();

                let node_future = AsyncDropShared::clone(child_inode.value().inode_future());

                // Free the lock on inner before waiting for the child node to load
                std::mem::drop(inner);

                let inode = Self::_wait_for_node_loaded(node_future).await?;

                log::info!("Existing inode {child_ino:?}: parent={parent_ino:?}, name={name}");
                Ok((child_ino, inode))

                // Fulfilling invariants:
                // * A, C, D, E, F: No change here
                // * B1: We've just waited for loading to be successful and complete before returning the inode number
                // * B2: The user gave us parent_ino, so by invariant B1+B2, all its ancestors are also fully loaded.
            }
        }
    }

    async fn _wait_for_node_loaded(
        mut node_future: AsyncDropGuard<
            AsyncDropShared<
                AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
                BoxFuture<
                    'static,
                    AsyncDropGuard<
                        AsyncDropResult<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>, FsError>,
                    >,
                >,
            >,
        >,
    ) -> FsResult<AsyncDropGuard<AsyncDropArc<Fs::Node>>> {
        with_async_drop_2!(node_future, {
            let node = (&mut *node_future).await;
            with_async_drop_2!(node, {
                match node.as_inner() {
                    Err(err) => Err(err.clone()),
                    Ok(node) => Ok(AsyncDropArc::clone(node.value())),
                }
            })
        })
    }

    /// Precondition: The inode doesn't exist yet in inner.inode_forest.
    async fn _add_new<F>(
        &self,
        mut inner: MutexGuard<'_, InodeListInner<Fs>>,
        parent_ino: InodeNumber,
        name: PathComponentBuf,
        loading_fn: impl FnOnce(&AsyncDropGuard<AsyncDropArc<Fs::Node>>) -> F + Send + 'static,
    ) -> FsResult<(
        HandleWithGeneration<InodeNumber>,
        AsyncDropGuard<AsyncDropArc<Fs::Node>>,
    )>
    where
        F: Future<Output = FsResult<AsyncDropGuard<Fs::Node>>> + Send,
    {
        let parent_node = {
            let mut parent_node = Self::_lookup_node(&inner, parent_ino)?;
            let parent_node_value = AsyncDropArc::clone(parent_node.value());
            parent_node.async_drop().await?;
            parent_node_value
        };

        // TODO This Arc::clone is only necessary because MutexGuard can't project and get &mut on both inner.inodes and inner.inode_forest at the same time. Once Rust supports that, we can avoid this clone.
        let inodes = AsyncDropArc::clone(&inner.inodes);
        let (new_child_ino, new_node) = with_async_drop_2!(inodes, {
            let insert_result = inner
                .inode_forest
                .try_insert(parent_ino, name, async |new_child_ino| {
                    let inserting = ConcurrentStore::try_insert_loading(
                        &inodes,
                        new_child_ino.handle,
                        async move || {
                            // It's ok to capture the parent_node in this lambda, because
                            // * If try_insert returns Ok, it always executes the lambda and we async_drop it here
                            // * If try_insert returns Err, the lambda is never executed, but we panic below anyways.
                            with_async_drop_2!(parent_node, {
                                let node = loading_fn(&parent_node).await?;
                                Ok(node)
                            })
                        },
                    )
                    // TODO Remove this await. It only triggers async code if the inode happens to be dropping at this exact moment,
                    //      but even so it is better to not wait for it here while having a lock on inner.
                    .await
                    .expect("Invariant D violated: entry for a new inode number already exists");

                    InodeInfo::new(AsyncDropShared::new(
                        async move { AsyncDropResult::new(inserting.wait_until_inserted().await) }
                            .boxed(),
                    ))
                })
                .await;

            match insert_result {
                Ok((new_child_ino, new_node)) => Ok((new_child_ino, new_node)),
                Err(TryInsertError2::ParentNotFound) => {
                    panic!("We just looked up the parent above, it must exist.");
                }
                Err(TryInsertError2::AlreadyExists) => {
                    panic!("We checked above that it doesn't exist yet (see precondition)");
                }
            }
        })
        .infallible_unwrap();

        // Now free the lock on the inner befor waiting for it to be loaded
        let inserting = AsyncDropShared::clone(new_node.value().inode_future());
        std::mem::drop(inner);

        // Fulfilling invariants when the lock is released:
        // * A: No change here
        // * B: We're stll loading but we haven't given out the InodeNumber yet
        // * C: self.inode_forest.try_insert() guarantees that the lambda creating the inode in self.inodes is executed
        //      if and only if the node is inserted to self.inode_tree. The entry is either added to both or neither.
        //      * C1:  Note that loading can fail after we release the lock, which can temporarily violate invariant C1, but we'll fix that below.
        // * D: If we added it to self.inode_forest, HandleForest keeps it blocked. If we didn't add it to inode_forest,
        //      we also didn't add it to self.inodes (see invariant C in previous point), so no need to block it.
        // * E: The new entry got a refcount of 1.
        // * F: The new entry has exactly one guard active in inode_forest.

        let node = Self::_wait_for_node_loaded(inserting).await;
        match node {
            Ok(node) => Ok((new_child_ino, node)),
            Err(err) => {
                // If loading failed, then ConcurrentStore already removed it from self.inodes, and our future in inode_forest is now invalid.
                // Let's just remove it from inode_forest as well to keep things consistent and re-establish invariant E1.
                let mut inner = self.inner.lock().await;
                assert!(
                    // TODO Is this assertion a race condition? Could it be that removal from self.inodes is delayed?
                    inner.inodes.is_fully_absent(&new_child_ino.handle),
                    "We just checked that loading failed, so invariant C1 must be violated here"
                );
                let (mut removed_node, remove_result, delayed_handle_release) = inner
                    .inode_forest
                    .try_remove(new_child_ino.handle)
                    .expect("This should never happen because we just added the child above");
                let drop_result = removed_node.async_drop().await;
                // Release the inode number to be reused.
                delayed_handle_release.release(&mut inner.inode_forest);
                drop_result?;
                match remove_result {
                    TryRemoveResult::NoParent => {
                        panic!("Parent entry vanished while we were adding it, this can't happen");
                    }
                    TryRemoveResult::ParentStillHasChildren { parent_handle }
                    | TryRemoveResult::JustRemovedLastChildOfParent { parent_handle }
                    | TryRemoveResult::ParentDidntHaveRemovedNodeAsChild { parent_handle } => {
                        // Even though we just removed a child, we don't need to consider removing the parent inode because we have its InodeNumber,
                        // so we know the refcount is larger than zero. But let's assert that to be sure.
                        assert!(
                            inner
                                .inode_forest
                                .get(&parent_handle)
                                .expect("We just checked above that the parent exists")
                                .value()
                                .refcount()
                                > 0,
                            "The parent entry existed before our call so it must have refcount > 0"
                        );
                    }
                }
                Err(err)
            }
        }

        // Fulfilling invariants since re-locking:
        // * A: No change here
        // * B1: We're only returning the inode after it was fully loaded successfully
        // * B2: The user gave us parent_ino, so by invariant B1+B2, all its ancestors are also fully loaded.
        // * C: If loading failed, self.inodes already removed the entry, but we then cleaned it up.
        //      C1 is re-established. If loading succeeded, both self.inodes and self.inode_forest contain the entry.
        // * D: We only released the inode number if we also removed it from self.inode_forest after it was removed from self.inodes.
        // * E: No change here
        // * F: No change here
    }

    pub async fn forget(&self, ino: InodeNumber) -> FsResult<()> {
        if ino == FUSE_ROOT_ID {
            log::error!("Tried to forget root inode");
            return Err(FsError::InvalidOperation);
        }

        let mut inner = self.inner.lock().await;
        let Some(inode) = inner.inode_forest.get_mut(&ino) else {
            log::error!("Tried to forget unknown inode {ino:?}");
            return Err(FsError::InvalidOperation);
        };

        match inode.value_mut().decrement_refcount() {
            RefcountInfo::RefcountNotZero => {
                // Refcount is still > 0, nothing more to do
                // Fulfilling invariants:
                // * A, B, C, D, F: No change here
                // * E: Refcount is still > 0
                return Ok(());
            }
            RefcountInfo::RefcountZero => {
                // Continue to remove the inode

                if inode.has_children() {
                    // Still has children, nothing more to do
                    // Fulfilling invariants:
                    // * A, B, C, D, F: No change here
                    // * E: Still has children
                    return Ok(());
                }

                let parent_ino = *inode
                    .parent_handle()
                    .expect("Tried to forget inode but it doesn't have a parent");

                let mut to_async_drop = Vec::new();
                let result = self._remove_inode(&mut inner, ino, parent_ino, &mut to_async_drop);
                let (removed_inos, removed_inodes, delayed_handle_releases): (
                    Vec<InodeNumber>,
                    Vec<AsyncDropGuard<InodeInfo<Fs>>>,
                    Vec<DelayedHandleRelease<InodeNumber>>,
                ) = multiunzip(to_async_drop.into_iter());

                // Fulfilling invariant when dropping the lock: See comments in [Self::_remove_inode].

                // Now inodes lock is released and we can drop all removed inodes
                std::mem::drop(inner);
                let drop_result =
                    for_each_unordered(removed_inodes.into_iter(), |mut inode| async move {
                        inode.async_drop().await?;
                        // TODO What to do if async_drop fails? Did the entry still get removed or ar we in an inconsistent state now?
                        Ok::<(), FsError>(())
                    })
                    .await;

                // Re-lock and verify C2, also release the inode numbers to be reused
                let mut inner = self.inner.lock().await;
                for ino in removed_inos {
                    // Now the guard is dropped, which should also have removed it from self.inode_forest
                    assert!(
                        inner.inodes.is_fully_absent(&ino),
                        "Invariant C2 violated: inode still present in self.inodes after dropping its reference"
                    );
                }

                // And free all InodeNumbers to be reused. To uphold invariant D (see also C2), we do this only after the async drop completed.
                for delayed_handle_release in delayed_handle_releases {
                    delayed_handle_release.release(&mut inner.inode_forest);
                }

                drop_result?;
                result?;

                // Fulfilling invariants:
                // * A, B, E, F are fulfilled after the call to[Self::_remove_inode].
                // * C: Invariant C2 is re-established because the async_drop of the guard removes the entries from self.inodes
                //      And we have an assertion above checking this.
                // * D: We only released the inode numbers after the async drop completed, so invariant D is upheld.
            }
        }

        Ok(())
    }

    fn _remove_inode<'a>(
        &self,
        inner: &mut MutexGuard<'_, InodeListInner<Fs>>,
        mut child_ino: InodeNumber,
        mut parent_ino: InodeNumber,
        to_async_drop: &mut Vec<(
            InodeNumber,
            AsyncDropGuard<InodeInfo<Fs>>,
            DelayedHandleRelease<InodeNumber>,
        )>,
    ) -> FsResult<()> {
        while child_ino != FUSE_ROOT_ID {
            // First remove the node itself (dropping the guard in self.inode_forest later when to_async_drop is processed will also remove it from self.inodes)
            let (removed_entry, remove_result, delayed_handle_release) = inner
                .inode_forest
                // TODO remove_by_name might be faster because we don't have to search the whole map
                .try_remove(child_ino)
                .expect("Inode reference disappeared");
            to_async_drop.push((child_ino, removed_entry, delayed_handle_release));

            match remove_result {
                TryRemoveResult::NoParent
                | TryRemoveResult::ParentDidntHaveRemovedNodeAsChild { .. } => {
                    // Child entry didn't exist. We were a tree root. Everything is ok.
                    break;
                }
                TryRemoveResult::ParentStillHasChildren { .. } => {
                    // Parent inode still has children, we can stop here.
                    break;
                }
                TryRemoveResult::JustRemovedLastChildOfParent { parent_handle } => {
                    // Parent has no more children. Let's check its refcount to see if we can remove it as well.
                    let parent_inode = inner.inode_forest.get_mut(&parent_handle).expect(
                        "Tried to remove inode but its parent vanished while we were removing the child",
                    );
                    if parent_inode.value().refcount() > 0 {
                        // Parent still has references, we can stop here.
                        break;
                    }

                    // It was already removed from the ConcurrentStore because the last reference just got removed.
                    // So we can continue our removal algorithm up the tree.
                    child_ino = parent_ino;
                    parent_ino = *parent_inode
                        .parent_handle()
                        .expect("Tried to remove inode but its parent has no parent");
                    continue;
                }
            }
        }
        Ok(())

        // Fulfilling invariants:
        // * A: No change here
        // * B: No change here.
        // * C: We're removing entries from self.inode_forest here. We store the guard in to_async_drop to drop it later,
        //      which will then free the corresponding entry in self.inodes as well (invariant F). So eventually, both entries are removed.
        //      This is temporarily violating invariant C2, but re-established by the async drop.
        // * D: We're using DelayedHandleRelease to only free the inode numbers after the async drop completed, so invariant D is upheld.
        // * E: We're only removing inodes that have refcount 0 and no children, so invariant E is upheld.
        // * F: We're dropping the only active guard for each removed inode, so invariant F is upheld.
    }

    #[cfg(feature = "testutils")]
    pub async fn clear_all_slow(&self) -> FsResult<()> {
        let mut inner = self.inner.lock().await;
        let RequestImmediateDropResult::ImmediateDropRequested {
            drop_result: root_node_drop_result,
        } = inner
            .inodes
            .request_immediate_drop(FUSE_ROOT_ID, async |n| n)
        else {
            panic!("Tried to clear all inodes but root inode is already dropping");
        };

        for_each_unordered(
            inner.inode_forest.drain(),
            |(_ino, mut inode_info)| async move { inode_info.async_drop().await },
        )
        .await?;
        // Dropping references also drops the corresponding entries in self.inodes
        assert!(inner.inodes.is_empty());

        let root_node = root_node_drop_result
            .await
            .expect("Root node wasn't loaded");

        Self::block_invalid_handles(&mut inner.inode_forest);

        // Re-add root inode so the InodeList is still usable after this call
        self._insert_rootdir(&mut inner, root_node).await;

        // Fulfilling invariants:
        // We setup the data structure from scratch again, so all invariants are established.

        Ok(())
    }

    #[cfg(feature = "testutils")]
    pub async fn fsync_all(&self) -> FsResult<()> {
        // TODO Don't just look at loading or loaded entries, but also at ones that are currently dropping and wait for them to be dropped.
        let inodes = ConcurrentStore::all_loading_or_loaded(&self.inner.lock().await.inodes);
        for_each_unordered(inodes.into_iter(), async |inode| {
            use crate::object_based_api::Node as _;

            let guard = inode.wait_until_loaded().await.unwrap().unwrap();
            with_async_drop_2!(guard, { guard.value().fsync(false).await })
        })
        .await?;
        Ok(())
    }
}

impl<Fs> Debug for InodeList<Fs>
where
    Fs: Device + Debug + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InodeList").finish()
    }
}

#[async_trait]
impl<Fs> AsyncDrop for InodeList<Fs>
where
    Fs: Device + Debug + 'static,
{
    type Error = FsError;
    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        // The kernel doesn't guarantee that it'll forget all inodes on shutdown, so we can't assert that all inodes are forgotten here.
        // Instead, we just drop all remaining inodes.
        let inner = self.inner.get_mut();
        let result = inner.inode_forest.async_drop().await;
        // And after all of its references are dropped, we can drop the inode list itself
        inner.inodes.async_drop().await.infallible_unwrap();

        result

        // We don't assert that refcount is zero because the fuse kernel doesn't guarantee that it'll forget all inodes on shutdown.
        // TODO But maybe we still want to assert it when an InodeInfo is dropped in a non-shutdown scenario. Maybe we should add the assertion here and add a [InodeInfo::drop_on_shutdown(self)] that deals with the shutdown case?
    }
}
