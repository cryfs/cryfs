use async_trait::async_trait;
use core::panic;
#[cfg(feature = "testutils")]
use cryfs_concurrent_store::RequestImmediateDropResult;
use cryfs_concurrent_store::{ConcurrentStore, LoadedEntryGuard};
use cryfs_utils::stream::for_each_unordered;
use cryfs_utils::with_async_drop_2;
use derive_more::{Display, Error};
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
use crate::common::HandleWithGeneration;
use crate::object_based_api::utils::inode_list::handle_forest::{
    DelayedHandleRelease, GetChildOfError, HandleForest, MoveInodeSuccess, TryInsertError,
    TryRemoveResult,
};
use crate::object_based_api::utils::inode_list::inode_tree_node::RefcountInfo;
use crate::{FsError, object_based_api::Device};
use cryfs_utils::path::{PathComponent, PathComponentBuf};

pub const FUSE_ROOT_ID: InodeNumber =
    InodeNumber::from_const(NonZeroU64::new(fuser::FUSE_ROOT_ID).unwrap());
pub const DUMMY_INO: InodeNumber =
    InodeNumber::from_const(NonZeroU64::new(fuser::FUSE_ROOT_ID + 1).unwrap());

mod handle_forest;
pub use handle_forest::MakeOrphanError;

mod inode_tree_node;
use inode_tree_node::InodeTreeNode;

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
    //     * Note: For entries in `inode_forest`, this is ensured by HandleForest. So we only need to ensure it for entries in `inodes` that are not in `inode_forest` (see invariant C2).
    //       This is done by DelayedHandleRelease when dropping such entries.
    // * E: Refcount:
    //     * E1: The refcount of an entry is
    //       * the number of times its InodeNumber was returned (or if it is mid loading that counts as well, even though it wasn't returned yet)
    //       * minus the number of times it was forgotten
    //       * plus the number of times it was added as a parent pointer to a child inode in `inode_forest`.
    //         (note that this can be different from the number of children stored in the inode, since orphaned nodes can still point to their parents but the parents dont have them in their children array)
    //     * E2: All entries in `inode_forest` have refcount > 0, otherwise they would have been deleted.
    // * F: Each entry in self.inodes has at most one guard active, which is stored in self.inode_forest.
    // TODO Node here holds a reference to the ConcurrentFsBlob, which blocks the blob from being removed. This would be a deadlock in unlink/rmdir if we store a reference to the self blob in NodeInfo.
    //      Right now, we only store a reference to the parent blob and that's fine because child inodes are forgotten before the parent can be removed.
    inodes: AsyncDropGuard<ConcurrentStore<InodeNumber, Fs::Node, FsError>>,

    // All the inode numbers we've given to the kernel, with a corresponding refcount. These references ensure that the inodes are being kept alive in self.inodes.
    // On top of this refcount, each InodeInfo also remmbers its parent InodeNumber. Overall, we guarantee that inodes only get removed once the kernel has
    // released it and all its children.
    // TODO Use Vec or slab instead of HashMap since InodeNumber is mostly contiguous?
    inode_forest: AsyncDropGuard<HandleForest<InodeNumber, PathComponentBuf, InodeTreeNode<Fs>>>,
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
                inodes,
                inode_forest,
            }),
        })
        // Fulfilling invariants:
        //  * A not fulfilled yet, [Self::insert_rootdir] must be called first.
        //  * B, C, D, E, F trivially fulfilled
    }

    fn block_invalid_handles(
        inode_forest: &mut HandleForest<InodeNumber, PathComponentBuf, InodeTreeNode<Fs>>,
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
        // * E: Root inode has a refcount == 1
        // * F: Root inode has exactly one guard active in inode_forest
    }

    async fn _insert_rootdir(
        &self,
        inner: &mut MutexGuard<'_, InodeListInner<Fs>>,
        rootdir: AsyncDropGuard<Fs::Node>,
    ) {
        let inserted = inner
            .inodes
            .try_insert_loaded(FUSE_ROOT_ID, rootdir)
            .expect("Root dir entry already exists");
        inner
            .inode_forest
            .try_insert_root_with_specific_handle(
                FUSE_ROOT_ID,
                InodeTreeNode::new(AsyncDropShared::new(
                    future::ready(AsyncDropResult::new(Ok(inserted))).boxed(),
                )),
            )
            .expect("Failed to insert rootdir because it already exists in the forest");

        log::debug!("Inode {FUSE_ROOT_ID}: Added rootdir inode");
    }

    fn _lookup_node(
        inner: &MutexGuard<'_, InodeListInner<Fs>>,
        ino: InodeNumber,
    ) -> FsResult<AsyncDropGuard<LoadedEntryGuard<InodeNumber, Fs::Node, FsError>>> {
        inner
            .inodes
            .get_if_loading_or_loaded(ino)
            .wait_until_loaded()
            .now_or_never()
            .ok_or_else(|| {
                // Invariant B violated, but this can happen if the kernel gives us wrong inode numbers, so treating it as InvalidOperation
                log::error!("inode {ino}: Tried to load inode but it is still loading");
                FsError::InvalidOperation
            })??
            .ok_or_else(|| {
                log::error!("inode {ino}: Tried to load inode but inode number isn't assigned");
                FsError::InvalidOperation
            })
    }

    pub async fn get_node_and_parent_ino(
        &self,
        ino: InodeNumber,
    ) -> FsResult<(AsyncDropGuard<AsyncDropArc<Fs::Node>>, InodeNumber)> {
        let inner = self.inner.lock().await;
        let inode_tree_node = inner.inode_forest.get(&ino).ok_or_else(|| {
            log::error!("inode {ino}: Tried to get inode info for unknown inode");
            FsError::InvalidOperation
        })?;
        let parent_ino = inode_tree_node
            .parent_handle()
            .copied()
            .unwrap_or(PARENT_OF_ROOT_INO);
        let node = Self::_get_node(&inner, ino).await?;
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
        let inodes = inner.inodes.clone_ref();
        with_async_drop_2!(inodes, {
            let insert_result = inner
                .inode_forest
                .try_insert(parent_ino, name, node, async |node, new_child_ino| {
                    let inserted_node = inodes
                        .try_insert_loaded(new_child_ino.handle, node)
                        .expect("Invariant D violated: A new (i.e. not blocked) inode number was already in use.");
                    InodeTreeNode::new(AsyncDropShared::new(
                        future::ready(AsyncDropResult::new(Ok(inserted_node))).boxed(),
                    ))
                })
                .await;

            match insert_result {
                Err(TryInsertError::ParentNotFound) => {
                    log::error!(
                        "Inode: {parent_ino}: Tried to add inode under unknown parent inode {parent_ino}"
                    );
                    return Err(FsError::InvalidOperation);
                }
                Err(TryInsertError::AlreadyExists) => {
                    log::error!(
                        "Inode: {parent_ino}: Tried to add already existng inode {name_clone} under parent inode {parent_ino}"
                    );
                    return Err(FsError::NodeAlreadyExists);
                }
                Ok((new_child_ino, _new_node)) => {
                    // Adjust refcount of parent for invariant E1.
                    inner
                        .inode_forest
                        .get_mut(&parent_ino)
                        .expect("We checked above that parent exists")
                        .value_mut()
                        .increment_refcount();
                    log::debug!(
                        "Inode {new_child_ino}: Added with parent={parent_ino}, name={name_clone}"
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
        // * E: The new entry got a refcount of 1 because we are returning it now, and its parent got its refcount incremented by 1, if adding was successful.
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
                    "Inode {parent_ino}: Tried to add/increment inode under unknown parent inode {parent_ino}"
                );
                Err(FsError::InvalidOperation)
            }
            Err(GetChildOfError::ChildNotFound) => {
                // Child doesn't exist yet, create it

                let name_clone = name.clone();
                let (child_ino, node) = self._add_new(inner, parent_ino, name, loading_fn).await?;
                log::debug!("Inode {child_ino}: Added with parent={parent_ino}, name={name_clone}");
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

                log::debug!(
                    "Inode {child_ino}: Found existing with parent={parent_ino}, name={name}"
                );
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

        // Adjust refcount of parent for invariant E1.
        inner
            .inode_forest
            .get_mut(&parent_ino)
            .expect("We checked above that parent exists")
            .value_mut()
            .increment_refcount();

        // TODO This Arc::clone is only necessary because MutexGuard can't project and get &mut on both inner.inodes and inner.inode_forest at the same time. Once Rust supports that, we can avoid this clone.
        let inodes = inner.inodes.clone_ref();
        let (new_child_ino, new_node) = with_async_drop_2!(inodes, {
            let insert_result = inner
                .inode_forest
                .try_insert(
                    parent_ino,
                    name,
                    parent_node,
                    async |parent_node, new_child_ino| {
                        let inserting = inodes
                            .try_insert_loading(new_child_ino.handle, async move || {
                                // It's ok to capture the parent_node in this lambda, because
                                // * If try_insert returns Ok, it always executes the lambda and we async_drop it here
                                // * If try_insert returns Err, the lambda is never executed, but we panic below anyways.
                                with_async_drop_2!(parent_node, {
                                    let node = loading_fn(&parent_node).await?;
                                    Ok(node)
                                })
                            })
                            .expect(
                                "Invariant D violated: entry for a new inode number already exists",
                            );

                        InodeTreeNode::new(
                            AsyncDropShared::new(
                                async move {
                                    AsyncDropResult::new(inserting.wait_until_inserted().await)
                                }
                                .boxed(),
                            ),
                        )
                    },
                )
                .await;

            match insert_result {
                Ok((new_child_ino, new_node)) => Ok((new_child_ino, new_node)),
                Err(TryInsertError::ParentNotFound) => {
                    panic!("We just looked up the parent above, it must exist.");
                }
                Err(TryInsertError::AlreadyExists) => {
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
        // * E: The new entry got a refcount of 1 because it is mid loading, and its parent got its refcount incremented by 1, if adding was successful.
        // * F: The new entry has exactly one guard active in inode_forest.

        let node = Self::_wait_for_node_loaded(inserting).await;
        match node {
            Ok(node) => Ok((new_child_ino, node)),
            Err(err) => {
                // If loading failed, then ConcurrentStore already removed it from self.inodes, and our future in inode_forest is now invalid.
                // Let's just remove it from inode_forest as well to keep things consistent and re-establish invariant E1.
                let mut inner = self.inner.lock().await;
                log::debug!(
                    "Inode {new_child_ino}: Loading failed, cleaned up its entry from InodeList"
                );
                assert!(
                    // If loading failed, then ConcurrentStore already removed the entry before completing the loading future
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
                match remove_result {
                    TryRemoveResult::NoParent => {
                        panic!("Parent entry vanished while we were adding it, this can't happen");
                    }
                    TryRemoveResult::ParentStillHasChildren { parent_handle }
                    | TryRemoveResult::JustRemovedLastChildOfParent { parent_handle }
                    | TryRemoveResult::ParentDidntHaveRemovedNodeAsChild { parent_handle } => {
                        let parent_inode = inner
                            .inode_forest
                            .get_mut(&parent_handle)
                            .expect("We just checked above that the parent exists")
                            .value_mut();
                        // We removed a child node pointing to this parent, so decrement its refcount for invariant E1.
                        match parent_inode.decrease_refcount(1) {
                            RefcountInfo::RefcountNotZero => {
                                // Refcount is still > 0, nothing more to do
                            }
                            RefcountInfo::RefcountZero => {
                                panic!(
                                    "Invariant E1 violated: Parent inode's refcount went to zero even though we have its InodeNumber"
                                );
                            }
                        }
                    }
                }
                drop_result?;
                log::debug!("Inode {new_child_ino}: cleaned up");
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
        // * E: If loading succeeded, refcount of the node is still 1 and we're now returning its InodeNumber. Refcount of parent remains
        //      to its incrementd value if loading succeeded, because we now have a child pointing to it, and got decremented back down
        //      if loading failed and the child node got removed.
        // * F: No change here
    }

    pub async fn forget(&self, ino: InodeNumber, nlookup: u64) -> FsResult<()> {
        if ino == FUSE_ROOT_ID {
            log::error!("Tried to forget root inode");
            return Err(FsError::InvalidOperation);
        }

        let inner = self.inner.lock().await;
        log::debug!("Inode {ino}: Forgetting nlookup={nlookup}");
        self._decrease_refcount(inner, ino, nlookup)
            .await
            .map_err(|err| match err {
                DecrementRefcountError::NodeNotFound => {
                    // Kernel gave us a wrong inode number
                    FsError::InvalidOperation
                }
                DecrementRefcountError::ErrorWhileDroppingNode(err) => err,
            })
    }

    async fn _decrease_refcount(
        &self,
        mut inner: MutexGuard<'_, InodeListInner<Fs>>,
        ino: InodeNumber,
        nlookup: u64,
    ) -> Result<(), DecrementRefcountError> {
        let Some(inode) = inner.inode_forest.get_mut(&ino) else {
            log::error!("Inode {ino}: Tried to forget unknown inode");
            return Err(DecrementRefcountError::NodeNotFound);
        };

        match inode.value_mut().decrease_refcount(nlookup) {
            RefcountInfo::RefcountNotZero => {
                // Refcount is still > 0, nothing more to do
                // Fulfilling invariants:
                // * A, B, C, D, F: No change here
                // * E: Refcount is still > 0
                return Ok(());
            }
            RefcountInfo::RefcountZero => {
                // Continue to remove the inode

                assert!(
                    !inode.has_children(),
                    "Invariant E2 violated: tried to forget inode {ino:?} whose refcount went to zero but still has children"
                );

                let parent_ino = *inode
                    .parent_handle()
                    .expect("Tried to forget inode but it doesn't have a parent");

                let mut to_async_drop = Vec::new();
                self._remove_inode(&mut inner, ino, parent_ino, &mut to_async_drop);
                let (removed_inos, removed_inodes, delayed_handle_releases): (
                    Vec<InodeNumber>,
                    Vec<AsyncDropGuard<InodeTreeNode<Fs>>>,
                    Vec<DelayedHandleRelease<InodeNumber>>,
                ) = multiunzip(to_async_drop.into_iter());

                // Fulfilling invariant when dropping the lock: See comments in [Self::_remove_inode].

                // Now inodes lock is released and we can drop all removed inodes
                std::mem::drop(inner);
                let drop_result =
                    for_each_unordered(removed_inodes.into_iter(), |mut inode| async move {
                        inode.async_drop().await?;
                        // We dropped the guard and concurrent_store::LoadedEntryGuard ensures that the entry is removed from self.inodes now.
                        // Even if it's async_drop fails, it still removes the entry.
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

                drop_result.map_err(DecrementRefcountError::ErrorWhileDroppingNode)?;

                // Fulfilling invariants:
                // * A, B, E, F are fulfilled after the call to[Self::_remove_inode].
                // * C: Invariant C2 is re-established because the async_drop of the guard removes the entries from self.inodes
                //      And we have an assertion above checking this.
                // * D: We only released the inode numbers after the async drop completed, so invariant D is upheld.
            }
        }

        Ok(())
    }

    // Precondition: The inode's refcount is zero
    fn _remove_inode<'a>(
        &self,
        inner: &mut MutexGuard<'_, InodeListInner<Fs>>,
        mut child_ino: InodeNumber,
        mut parent_ino: InodeNumber,
        to_async_drop: &mut Vec<(
            InodeNumber,
            AsyncDropGuard<InodeTreeNode<Fs>>,
            DelayedHandleRelease<InodeNumber>,
        )>,
    ) {
        while child_ino != FUSE_ROOT_ID {
            // First remove the node itself (dropping the guard in self.inode_forest later when to_async_drop is processed will also remove it from self.inodes)
            let (removed_entry, remove_result, delayed_handle_release) = inner
                .inode_forest
                .try_remove(child_ino)
                .expect("Inode reference disappeared or still has children");
            to_async_drop.push((child_ino, removed_entry, delayed_handle_release));

            let parent_inode = inner.inode_forest.get_mut(&parent_ino).expect(
                "Tried to remove inode but its parent vanished while we were removing the child",
            );
            let parent_decr_refcount_result = parent_inode.value_mut().decrease_refcount(1);

            match remove_result {
                TryRemoveResult::NoParent => {
                    panic!(
                        "Tried to remove inode but its parent vanished while we were removing the child"
                    );
                }
                TryRemoveResult::ParentStillHasChildren { parent_handle } => {
                    assert_eq!(parent_handle, parent_ino);
                    // Our parent still has other children, in which case it must have refcount>0 because of invariant E1.
                    match parent_decr_refcount_result {
                        RefcountInfo::RefcountNotZero => { /* All good */ }
                        RefcountInfo::RefcountZero => {
                            panic!(
                                "Invariant E1 violated: Parent inode's refcount went to zero even though it still has children"
                            );
                        }
                    }
                    break;
                }
                TryRemoveResult::JustRemovedLastChildOfParent { parent_handle }
                | TryRemoveResult::ParentDidntHaveRemovedNodeAsChild { parent_handle } => {
                    assert_eq!(parent_handle, parent_ino);
                    // Parent might not have more children. Let's check its refcount to see if we can remove it as well.
                    let parent_inode = inner.inode_forest.get_mut(&parent_handle).expect(
                        "Tried to remove inode but its parent vanished while we were removing the child",
                    );
                    match parent_decr_refcount_result {
                        RefcountInfo::RefcountNotZero => {
                            // Parent still has references, we can stop here.
                            break;
                        }
                        RefcountInfo::RefcountZero => {
                            // Continue removing parent as well
                        }
                    }
                    assert_eq!(
                        0,
                        parent_inode.num_children(),
                        "Invariant E1 violated: Parent inode has refcount == 0 but still has children"
                    );

                    log::debug!(
                        "Inode {parent_ino}: Refcount went to zero after removing child {child_ino}, continuing removal up the tree"
                    );

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

        // Fulfilling invariants:
        // * A: No change here
        // * B: No change here.
        // * C: We're removing entries from self.inode_forest here. We store the guard in to_async_drop to drop it later,
        //      which will then free the corresponding entry in self.inodes as well (invariant F). So eventually, both entries are removed.
        //      This is temporarily violating invariant C2, but re-established by the async drop.
        // * D: We're using DelayedHandleRelease to only free the inode numbers after the async drop completed, so invariant D is upheld.
        // * E1: We're decrementing the refcount of parent inodes when removing child inodes, so invariant E1 is upheld.
        // * E2: We're only removing inodes that have refcount 0, so invariant E2 is upheld.
        // * F: We're dropping the only active guard for each removed inode, so invariant F is upheld.
    }

    /// Remove an inode from its parent's childrn map. The inode itself stays alive, but becomes an orphan in the inode forest.
    pub async fn make_into_orphan(
        &self,
        parent_ino: InodeNumber,
        name: &PathComponent,
    ) -> Result<(), MakeOrphanError> {
        // TODO After unlink, the kernel could try to re-enter a node ad parent_ino/name again.
        //      We support that here, but does CryNode support that correctly? The previous child node might still exist and stored in the orphaned inode.
        let mut inner = self.inner.lock().await;
        log::debug!("Inode {parent_ino} / {name}: Making into orphan");
        inner.inode_forest.make_node_into_orphan(&parent_ino, name)

        // Fulfilling invariants:
        // A, C, D, F: No change here
        // B1: No change here
        // B2: We're removing a parent pointer, which makes this invariant strictly easier to fulfil
        // E1: We're leaving the child orphaned, but its parent pointer still points to the parent.
        //     So no need to adjust the parents refcount for invariant E1.
        // E2: No changes to refcounts
    }

    /// Move an inode from one parent to another, possibly renaming it.
    /// precondition: The inode isn't yet loaded under new_parent_ino/new_name.
    pub async fn move_inode(
        &self,
        old_parent_ino: InodeNumber,
        old_name: &PathComponent,
        new_parent_ino: InodeNumber,
        new_name: PathComponentBuf,
    ) -> Result<(), MoveInodeError> {
        let mut inner = self.inner.lock().await;
        log::debug!(
            "Inode {old_parent_ino} / {old_name} -> {new_parent_ino} / {new_name}: Moving inode"
        );
        let move_result = inner
            .inode_forest
            .move_node(old_parent_ino, old_name, new_parent_ino, new_name)
            .map_err(|err| match err {
                handle_forest::MoveInodeError::OldParentNotFound => {
                    MoveInodeError::OldParentNotFound
                }
                handle_forest::MoveInodeError::NewParentNotFound => {
                    MoveInodeError::NewParentNotFound
                }
                handle_forest::MoveInodeError::ChildNotFound => MoveInodeError::ChildNotFound,
            })?;

        // We only get here if the move succeeded. Error cases, including the 'benign' error case of the child inode not being loaded, have already returned above.

        // According to invariant E1, each child's parent pointer counts towards the refcount of the parent.
        // Since we just redirected that pointer to a new parent, we need to adjust the refcounts.
        if old_parent_ino != new_parent_ino {
            match move_result {
                MoveInodeSuccess::OrphanedExistingChildInNewParent
                | MoveInodeSuccess::AddedAsNewChildToNewParent => {
                    // Whether we replaced an existing (now orphaned) child or whether we added a new child, we need to increment
                    // the refcount of the new parent for invariant E1, because we count the number of nodes pointing to it, not the number of nodes in is children map.
                    inner
                        .inode_forest
                        .get_mut(&new_parent_ino)
                        .expect("We already checked that it exists when we moved the node above")
                        .value_mut()
                        .increment_refcount();
                }
            }
            self._decrease_refcount(inner, old_parent_ino, 1)
                .await
                .map_err(|err| match err {
                    DecrementRefcountError::NodeNotFound => {
                        panic!("We already checked that it exists when we moved the node above")
                    }
                    DecrementRefcountError::ErrorWhileDroppingNode(err) => {
                        MoveInodeError::ErrorWhileDroppingNode(err)
                    }
                })?;
        } else {
            match move_result {
                MoveInodeSuccess::OrphanedExistingChildInNewParent => {
                    panic!(
                        "If parents are the same, we should never hit the case of orphaning an existing child."
                    )
                }
                MoveInodeSuccess::AddedAsNewChildToNewParent => {
                    // everything ok
                }
            }
        }

        // Fulfilling invariants:
        // * A, C, D, F: No change here
        // * B1: No change here
        // * B2: We assigned a new parent pointer to the node, but with B1+B2, we know that that parent is fully loaded.
        // * E1: If parent didn't change, we didn't change refcounts.
        //       If parent changed, we incremented new parent's refcount and decremented old parent's refcount.
        // * E2: We used [Self::_decrement_refcount] to decrement old parent's refcount, which would drop the parent if its refcount went to zero.
        Ok(())
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
        let inodes = self.inner.lock().await.inodes.all_loading_or_loaded();
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

        // We don't assert that all inodes were forgotten because the fuse kernl doesn't guarantee that on shutdown.
        // TODO But maybe we still want to assert it when an InodeInfo is dropped in a non-shutdown scenario. Maybe we should add the assertion here and add a [InodeInfo::drop_on_shutdown(self)] that deals with the shutdown case?
        // assert_eq!(1, inner.inode_forest.num_nodes());
        // assert!(inner.inode_forest.get(&FUSE_ROOT_ID).is_some());

        let result = inner.inode_forest.async_drop().await;
        // And after all of its references are dropped, we can drop the inode list itself
        inner.inodes.async_drop().await.infallible_unwrap();

        result
    }
}

#[derive(Error, Debug, Display)]
pub enum DecrementRefcountError {
    NodeNotFound,
    ErrorWhileDroppingNode(FsError),
}

#[derive(Error, Debug, Display)]
pub enum MoveInodeError {
    OldParentNotFound,
    NewParentNotFound,
    ChildNotFound,
    ErrorWhileDroppingNode(FsError),
}
