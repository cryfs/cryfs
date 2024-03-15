use cryfs_blobstore::{Blob, BlobId, BlobStore, BlobStoreOnBlocks, DataNodeStore, RemoveResult};
use cryfs_blockstore::{
    AllowIntegrityViolations, BlockId, BlockStoreReader, BlockStoreWriter, DynBlockStore,
    InMemoryBlockStore, IntegrityConfig, LockingBlockStore, MissingBlockIsIntegrityViolation,
    SharedBlockStore,
};
use cryfs_check::{
    BlobReference, BlobReferenceWithId, CorruptedError, MaybeBlobReferenceWithId,
    NodeAndBlobReference, NodeAndBlobReferenceFromReachableBlob, NodeInfoAsSeenByLookingAtNode,
    NodeReference,
};
use cryfs_cli_utils::setup_blockstore_stack_dyn;
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadResult, FixedPasswordProvider},
    filesystem::fsblobstore::{BlobType, FsBlob, FsBlobStore},
    localstate::LocalStateDir,
};
use cryfs_rustfs::AbsolutePathBuf;
use cryfs_utils::{
    async_drop::{AsyncDropGuard, SyncDrop},
    progress::SilentProgressBarManager,
};
use futures::{future::BoxFuture, stream::StreamExt, Future};
use rand::{rngs::SmallRng, SeedableRng};
use std::fmt::{Debug, Formatter};
use std::{collections::BTreeSet, path::PathBuf};
use tempdir::TempDir;

use super::console::FixtureCreationConsole;
use super::entry_helpers::{
    self, find_an_inner_node_of_a_large_blob, find_an_inner_node_of_a_large_blob_with_parent_id,
    find_an_inner_node_of_a_small_blob_with_parent_id, find_inner_node_with_distance_from_root,
    find_inner_node_with_distance_from_root_with_parent_id, find_leaf_node_of_blob_with_parent_id,
    find_leaf_node_with_parent_id, CreatedDirBlob, SomeBlobs,
};

const PASSWORD: &str = "mypassword";

pub struct FilesystemFixture {
    root_blob_id: BlobId,
    blockstore: SyncDrop<SharedBlockStore<InMemoryBlockStore>>,
    config: ConfigLoadResult,

    // tempdir should be in last position so it gets dropped last
    tempdir: FixtureTempDir,
}

impl FilesystemFixture {
    pub async fn new_with_some_blobs() -> (Self, SomeBlobs) {
        let fs_fixture = Self::new().await;
        let some_blobs = fs_fixture.create_some_blobs().await;
        (fs_fixture, some_blobs)
    }

    pub async fn new() -> Self {
        let tempdir = FixtureTempDir::new();
        let blockstore = SharedBlockStore::new(InMemoryBlockStore::new());
        let config = tempdir.create_config();
        let root_blob_id = BlobId::from_hex(&config.config.config().root_blob).unwrap();
        let result = Self {
            tempdir,
            blockstore: SyncDrop::new(blockstore),
            config,
            root_blob_id,
        };
        result.create_root_dir_blob().await;
        result
    }

    async fn create_root_dir_blob(&self) {
        let mut fsblobstore = self.make_fsblobstore().await;
        fsblobstore
            .create_root_dir_blob(&self.root_blob_id)
            .await
            .expect("Failed to create rootdir blob");
        fsblobstore.async_drop().await.unwrap();
    }

    async fn make_locking_blockstore(&self) -> AsyncDropGuard<LockingBlockStore<DynBlockStore>> {
        setup_blockstore_stack_dyn(
            SharedBlockStore::clone(&self.blockstore),
            &self.config,
            &self.tempdir.local_state_dir(),
            IntegrityConfig {
                allow_integrity_violations: AllowIntegrityViolations::DontAllowViolations,
                missing_block_is_integrity_violation:
                    MissingBlockIsIntegrityViolation::IsAViolation,
                on_integrity_violation: Box::new(|_err| {
                    panic!("integrity violation");
                }),
            },
        )
        .await
        .expect("Failed to setup blockstore stack")
    }

    async fn make_nodestore(&self) -> AsyncDropGuard<DataNodeStore<DynBlockStore>> {
        let blockstore = self.make_locking_blockstore().await;

        DataNodeStore::new(
            blockstore,
            // TODO Change type in config instead of doing u32::try_from
            u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
        )
        .await
        .expect("Failed to create DataNodeStore")
    }

    async fn make_blobstore(&self) -> AsyncDropGuard<BlobStoreOnBlocks<DynBlockStore>> {
        let blockstore = self.make_locking_blockstore().await;

        BlobStoreOnBlocks::new(
            blockstore,
            // TODO Change type in config instead of doing u32::try_from
            u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
        )
        .await
        .expect("Failed to create blobstore")
    }

    async fn make_fsblobstore(
        &self,
    ) -> AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>> {
        let blobstore = self.make_blobstore().await;

        FsBlobStore::new(blobstore)
    }

    pub async fn update_blockstore<'s, 'b, 'f, F, R>(
        &'s self,
        update_fn: impl FnOnce(&'b SharedBlockStore<InMemoryBlockStore>) -> F,
    ) -> R
    where
        F: 'f + Future<Output = R>,
        's: 'f + 'b,
        'b: 'f,
    {
        update_fn(&self.blockstore).await
    }

    pub async fn update_nodestore<R>(
        &self,
        update_fn: impl for<'b> FnOnce(&'b DataNodeStore<DynBlockStore>) -> BoxFuture<'b, R>,
    ) -> R {
        let mut nodestore = self.make_nodestore().await;
        let result = update_fn(&nodestore).await;
        nodestore.async_drop().await.unwrap();
        result
    }

    pub async fn update_blobstore<R>(
        &self,
        update_fn: impl for<'b> FnOnce(&'b BlobStoreOnBlocks<DynBlockStore>) -> BoxFuture<'b, R>,
    ) -> R {
        let mut blobstore = self.make_blobstore().await;
        let result = update_fn(&blobstore).await;
        blobstore.async_drop().await.unwrap();
        result
    }

    pub async fn update_fsblobstore<R>(
        &self,
        update_fn: impl for<'b> FnOnce(
            &'b FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>,
        ) -> BoxFuture<'b, R>,
    ) -> R {
        let mut fsblobstore = self.make_fsblobstore().await;
        let result = update_fn(&fsblobstore).await;
        fsblobstore.async_drop().await.unwrap();
        result
    }

    pub async fn run_cryfs_check(self) -> Vec<CorruptedError> {
        cryfs_check::check_filesystem(
            self.blockstore.into_inner_dont_drop(),
            &self.tempdir.config_file_path(),
            &self.tempdir.local_state_dir(),
            &FixedPasswordProvider::new(PASSWORD.to_owned()),
            SilentProgressBarManager,
        )
        .await
        .expect("Failed to run cryfs-check")
    }

    pub fn root_blob_id(&self) -> BlobId {
        self.root_blob_id
    }

    pub async fn create_some_blobs(&self) -> SomeBlobs {
        let root_id = self.root_blob_id;
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let root = FsBlob::into_dir(blobstore.load(&root_id).await.unwrap().unwrap())
                    .await
                    .unwrap();
                let mut root = CreatedDirBlob::new(root, AbsolutePathBuf::root());
                let result = super::entry_helpers::create_some_blobs(blobstore, &mut root).await;
                root.async_drop().await.unwrap();
                result
            })
        })
        .await
    }

    fn root_blob_info(&self) -> BlobReferenceWithId {
        BlobReferenceWithId {
            blob_id: self.root_blob_id,
            referenced_as: BlobReference::root_dir(),
        }
    }

    pub async fn create_empty_file(&self) -> BlobReferenceWithId {
        self.create_empty_file_in_parent(self.root_blob_info(), "file_name")
            .await
    }

    pub async fn create_empty_file_in_parent(
        &self,
        parent: BlobReferenceWithId,
        name: &str,
    ) -> BlobReferenceWithId {
        let name = name.to_owned();
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let parent_blob =
                    FsBlob::into_dir(blobstore.load(&parent.blob_id).await.unwrap().unwrap())
                        .await
                        .unwrap();
                let mut parent = CreatedDirBlob::new(parent_blob, parent.referenced_as.path);
                let result =
                    super::entry_helpers::create_empty_file(blobstore, &mut parent, &name).await;
                let result = (&result).into();
                parent.async_drop().await.unwrap();
                result
            })
        })
        .await
    }

    pub async fn create_empty_dir(&self) -> BlobReferenceWithId {
        self.create_empty_dir_in_parent(self.root_blob_info(), "dir_name")
            .await
    }

    pub async fn create_empty_dir_in_parent(
        &self,
        parent: BlobReferenceWithId,
        name: &str,
    ) -> BlobReferenceWithId {
        let name = name.to_owned();
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let parent_blob =
                    FsBlob::into_dir(blobstore.load(&parent.blob_id).await.unwrap().unwrap())
                        .await
                        .unwrap();
                let mut parent = CreatedDirBlob::new(parent_blob, parent.referenced_as.path);
                let mut created_dir =
                    super::entry_helpers::create_empty_dir(blobstore, &mut parent, &name).await;
                let result = (&*created_dir).into();
                created_dir.async_drop().await.unwrap();
                parent.async_drop().await.unwrap();
                result
            })
        })
        .await
    }

    pub async fn create_symlink(&self, target: &str) -> BlobReferenceWithId {
        self.create_symlink_in_parent(self.root_blob_info(), "symlink_name", target)
            .await
    }

    pub async fn create_symlink_in_parent(
        &self,
        parent: BlobReferenceWithId,
        name: &str,
        target: &str,
    ) -> BlobReferenceWithId {
        let name = name.to_owned();
        let target = target.to_owned();
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let parent_blob =
                    FsBlob::into_dir(blobstore.load(&parent.blob_id).await.unwrap().unwrap())
                        .await
                        .unwrap();
                let mut parent_blob = CreatedDirBlob::new(parent_blob, parent.referenced_as.path);
                let result = super::entry_helpers::create_symlink(
                    blobstore,
                    &mut parent_blob,
                    &name,
                    &target,
                )
                .await;
                let result = (&result).into();
                parent_blob.async_drop().await.unwrap();
                result
            })
        })
        .await
    }

    pub async fn add_file_entry_to_dir(&self, parent: BlobId, name: &str, blob_id: BlobId) {
        let name = name.to_owned();
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let mut parent = FsBlob::into_dir(blobstore.load(&parent).await.unwrap().unwrap())
                    .await
                    .unwrap();
                super::entry_helpers::add_file_entry(&mut parent, &name, blob_id);
                parent.async_drop().await.unwrap();
            })
        })
        .await;
    }

    pub async fn add_dir_entry_to_dir(&self, parent: BlobId, name: &str, blob_id: BlobId) {
        let name = name.to_owned();
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let mut parent = FsBlob::into_dir(blobstore.load(&parent).await.unwrap().unwrap())
                    .await
                    .unwrap();
                super::entry_helpers::add_dir_entry(&mut parent, &name, blob_id);
                parent.async_drop().await.unwrap();
            })
        })
        .await;
    }

    pub async fn add_symlink_entry_to_dir(&self, parent: BlobId, name: &str, blob_id: BlobId) {
        let name = name.to_owned();
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let mut parent = FsBlob::into_dir(blobstore.load(&parent).await.unwrap().unwrap())
                    .await
                    .unwrap();
                super::entry_helpers::add_symlink_entry(&mut parent, &name, blob_id);
                parent.async_drop().await.unwrap();
            })
        })
        .await;
    }

    pub async fn get_children_of_dir_blob(&self, dir_blob: BlobId) -> Vec<BlobId> {
        self.update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let blob = fsblobstore.load(&dir_blob).await.unwrap().unwrap();
                let mut blob = FsBlob::into_dir(blob).await.unwrap();
                let children = blob
                    .entries()
                    .map(|entry| *entry.blob_id())
                    .collect::<Vec<_>>();
                blob.async_drop().await.unwrap();
                children
            })
        })
        .await
    }

    pub async fn get_descendants_if_dir_blob<'a>(&'a self, maybe_dir_blob: BlobId) -> Vec<BlobId> {
        self.update_fsblobstore(move |fsblobstore| {
            Box::pin(
                entry_helpers::get_descendants_if_dir_blob(fsblobstore, maybe_dir_blob)
                    .collect::<Vec<BlobId>>(),
            )
        })
        .await
    }

    pub async fn get_descendants_of_dir_blob<'a>(&'a self, dir_blob: BlobId) -> Vec<BlobId> {
        self.update_fsblobstore(move |fsblobstore| {
            Box::pin(
                entry_helpers::get_descendants_of_dir_blob(fsblobstore, dir_blob)
                    .collect::<Vec<BlobId>>(),
            )
        })
        .await
    }

    pub async fn get_node_depth<'a>(&'a self, node_id: BlockId) -> u8 {
        self.update_nodestore(move |nodestore| {
            Box::pin(async move {
                let node = nodestore.load(node_id).await.unwrap().unwrap();
                node.depth()
            })
        })
        .await
    }

    pub async fn is_dir_blob(&self, blob_id: BlobId) -> bool {
        self.update_fsblobstore(move |fsblobstore| {
            Box::pin(async move {
                let mut blob = fsblobstore.load(&blob_id).await.unwrap().unwrap();
                let result = matches!(&*blob, FsBlob::Directory(_));
                blob.async_drop().await.unwrap();
                result
            })
        })
        .await
    }

    pub async fn corrupt_block(&self, block_id: BlockId) {
        self.update_blockstore(|blockstore| {
            Box::pin(async move {
                let mut block = blockstore.load(&block_id).await.unwrap().unwrap();
                let byte_index = 100 % block.len();
                block[byte_index] = block[byte_index].overflowing_add(1).0;
                blockstore.store(&block_id, &block).await.unwrap();
            })
        })
        .await;
    }

    pub async fn increment_format_version_of_blob(&self, blob_id: BlobId) {
        self.update_blobstore(|blobstore| {
            Box::pin(async move {
                let mut blob = blobstore.load(&blob_id).await.unwrap().unwrap();
                // The first u16 is the format version. Increase it by 1 to make the blob invalid
                let mut format_version = [0u8; 2];
                blob.read(&mut format_version, 0).await.unwrap();
                format_version[1] += 1;
                blob.write(&format_version, 0).await.unwrap();
            })
        })
        .await;
    }

    pub async fn corrupt_blob_type(&self, blob_id: BlobId) {
        self.update_blobstore(|blobstore| {
            Box::pin(async move {
                let mut blob = blobstore.load(&blob_id).await.unwrap().unwrap();
                // The third byte is for the blob type (dir/file/symlink). Set it to an invalid value.
                blob.write(&[10u8], 2).await.unwrap();
            })
        })
        .await;
    }

    pub async fn remove_root_node_of_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> RemoveInnerNodeResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let blob_root_node = nodestore
                    .load(*blob_info.blob_id.to_root_block_id())
                    .await
                    .unwrap()
                    .unwrap()
                    .into_inner_node()
                    .expect("test blob too small to have more than one node. We need to change the test and increase its size");
                let orphaned_nodes = blob_root_node.children().collect::<Vec<_>>();
                let inner_node_id = *blob_root_node.block_id();
                assert_eq!(blob_info.blob_id.to_root_block_id(), blob_root_node.block_id());
                blob_root_node.upcast().remove(nodestore).await.unwrap();
                RemoveInnerNodeResult {
                    removed_node: inner_node_id,
                    removed_node_info: NodeAndBlobReferenceFromReachableBlob {
                        node_info: NodeReference::RootNode,
                        blob_info: BlobReferenceWithId {
                            blob_id: blob_info.blob_id,
                            referenced_as: blob_info.referenced_as,
                        }
                    },
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn remove_blob(&self, blob_info: BlobReferenceWithId) {
        self.update_blobstore(move |blobstore| {
            Box::pin(async move {
                assert_eq!(
                    RemoveResult::SuccessfullyRemoved,
                    blobstore.remove_by_id(&blob_info.blob_id).await.unwrap(),
                );
            })
        })
        .await;
    }

    pub async fn corrupt_root_node_of_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> CorruptInnerNodeResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let blob_root_node = nodestore
                        .load(*blob_info.blob_id.to_root_block_id())
                        .await
                        .unwrap()
                        .unwrap();
                    assert_eq!(
                        blob_info.blob_id.to_root_block_id(),
                        blob_root_node.block_id()
                    );
                    let orphaned_nodes =
                        if let Some(blob_root_node) = blob_root_node.into_inner_node() {
                            blob_root_node.children().collect::<Vec<_>>()
                        } else {
                            vec![]
                        };

                    CorruptInnerNodeResult {
                        corrupted_node: *blob_info.blob_id.to_root_block_id(),
                        corrupted_node_info: NodeAndBlobReferenceFromReachableBlob {
                            node_info: NodeReference::RootNode,
                            blob_info: BlobReferenceWithId {
                                blob_id: blob_info.blob_id,
                                referenced_as: blob_info.referenced_as,
                            },
                        },
                        orphaned_nodes,
                    }
                })
            })
            .await;
        self.corrupt_block(result.corrupted_node).await;
        result
    }

    pub async fn remove_an_inner_node_of_a_large_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> RemoveInnerNodeResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let (inner_node, parent_id) = find_an_inner_node_of_a_large_blob_with_parent_id(
                    nodestore,
                    &blob_info.blob_id,
                )
                .await;
                let depth = inner_node.depth();
                let orphaned_nodes = inner_node.children().collect::<Vec<_>>();
                let inner_node_id = *inner_node.block_id();
                inner_node.upcast().remove(nodestore).await.unwrap();
                RemoveInnerNodeResult {
                    removed_node: inner_node_id,
                    removed_node_info: NodeAndBlobReferenceFromReachableBlob {
                        blob_info: BlobReferenceWithId {
                            blob_id: blob_info.blob_id,
                            referenced_as: blob_info.referenced_as,
                        },
                        node_info: NodeReference::NonRootInnerNode { depth, parent_id },
                    },
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn corrupt_an_inner_node_of_a_large_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> CorruptInnerNodeResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let (inner_node, inner_node_parent_id) =
                        find_an_inner_node_of_a_large_blob_with_parent_id(
                            nodestore,
                            &blob_info.blob_id,
                        )
                        .await;
                    let orphaned_nodes = inner_node.children().collect::<Vec<_>>();
                    let inner_node_id = *inner_node.block_id();
                    CorruptInnerNodeResult {
                        corrupted_node: inner_node_id,
                        corrupted_node_info: NodeAndBlobReferenceFromReachableBlob {
                            blob_info: BlobReferenceWithId {
                                blob_id: blob_info.blob_id,
                                referenced_as: blob_info.referenced_as,
                            },
                            node_info: NodeReference::NonRootInnerNode {
                                depth: inner_node.depth(),
                                parent_id: inner_node_parent_id,
                            },
                        },
                        orphaned_nodes,
                    }
                })
            })
            .await;
        self.corrupt_block(result.corrupted_node).await;
        result
    }

    pub async fn remove_an_inner_node_of_a_small_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> RemoveInnerNodeResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let (inner_node, parent_id) = find_an_inner_node_of_a_small_blob_with_parent_id(
                    nodestore,
                    &blob_info.blob_id,
                )
                .await;
                let depth = inner_node.depth();
                let orphaned_nodes = inner_node.children().collect::<Vec<_>>();
                let inner_node_id = *inner_node.block_id();
                inner_node.upcast().remove(nodestore).await.unwrap();
                RemoveInnerNodeResult {
                    removed_node: inner_node_id,
                    removed_node_info: NodeAndBlobReferenceFromReachableBlob {
                        node_info: NodeReference::NonRootInnerNode { depth, parent_id },
                        blob_info: BlobReferenceWithId {
                            blob_id: blob_info.blob_id,
                            referenced_as: blob_info.referenced_as,
                        },
                    },
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn corrupt_an_inner_node_of_a_small_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> CorruptInnerNodeResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let (inner_node, inner_node_parent_id) =
                        find_an_inner_node_of_a_small_blob_with_parent_id(
                            nodestore,
                            &blob_info.blob_id,
                        )
                        .await;
                    let orphaned_nodes = inner_node.children().collect::<Vec<_>>();
                    let inner_node_id = *inner_node.block_id();
                    CorruptInnerNodeResult {
                        corrupted_node: inner_node_id,
                        corrupted_node_info: NodeAndBlobReferenceFromReachableBlob {
                            node_info: NodeReference::NonRootInnerNode {
                                depth: inner_node.depth(),
                                parent_id: inner_node_parent_id,
                            },
                            blob_info: BlobReferenceWithId {
                                blob_id: blob_info.blob_id,
                                referenced_as: blob_info.referenced_as,
                            },
                        },
                        orphaned_nodes,
                    }
                })
            })
            .await;
        self.corrupt_block(result.corrupted_node).await;
        result
    }

    pub async fn remove_some_nodes_of_a_large_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> RemoveSomeNodesResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let inner_node =
                    find_an_inner_node_of_a_large_blob(nodestore, &blob_info.blob_id).await;
                let mut children = inner_node.children();
                let child1 = children.next().unwrap();
                let child2 = children.next().unwrap();

                let belongs_to_blob = MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                    blob_id: blob_info.blob_id,
                    referenced_as: blob_info.referenced_as,
                };

                let mut removed_nodes = vec![];
                let mut orphaned_nodes = vec![];

                // for child1, find an inner node A. Remove an inner node below A, a leaf below A, and A itself.
                {
                    let (inner_node_a, inner_node_a_parent_id) =
                        find_inner_node_with_distance_from_root_with_parent_id(nodestore, child1)
                            .await;
                    let mut children = inner_node_a.children();
                    let subchild1 = children.next().unwrap();
                    let subchild2 = children.next().unwrap();
                    std::mem::drop(children);

                    let (inner_below_a, inner_below_a_parent_id) =
                        find_inner_node_with_distance_from_root_with_parent_id(
                            nodestore, subchild1,
                        )
                        .await;
                    orphaned_nodes.extend(inner_below_a.children());
                    removed_nodes.push((
                        *inner_below_a.block_id(),
                        NodeAndBlobReference::NonRootInnerNode {
                            // `belongs_to_blob` is `UnreachableFromFilesystemRoot` because `inner_node_a` gets removed as well, so when cryfs-check is running, it won't be able to figure out which blob the removed node belonged to
                            belongs_to_blob:
                                MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                            depth: inner_below_a.depth(),
                            parent_id: inner_below_a_parent_id,
                        },
                    ));
                    inner_below_a.upcast().remove(nodestore).await.unwrap();

                    let (leaf_below_a, leaf_below_a_parent_id) =
                        find_leaf_node_with_parent_id(nodestore, subchild2, &mut rng).await;
                    removed_nodes.push((
                        *leaf_below_a.block_id(),
                        NodeAndBlobReference::NonRootLeafNode {
                            // `belongs_to_blob` is `UnreachableFromFilesystemRoot` because `inner_node_a` gets removed as well, so when cryfs-check is running, it won't be able to figure out which blob the removed node belonged to
                            belongs_to_blob:
                                MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot,
                            parent_id: leaf_below_a_parent_id,
                        },
                    ));
                    leaf_below_a.upcast().remove(nodestore).await.unwrap();

                    orphaned_nodes.extend(inner_node_a.children());
                    removed_nodes.push((
                        *inner_node_a.block_id(),
                        NodeAndBlobReference::NonRootInnerNode {
                            belongs_to_blob: belongs_to_blob.clone(),
                            depth: inner_node_a.depth(),
                            parent_id: inner_node_a_parent_id,
                        },
                    ));
                    inner_node_a.upcast().remove(nodestore).await.unwrap();
                }

                // for child2, find an inner node A. Remove an inner node B below A. Also remove an inner node C below A and its direct child. Don't remove A.
                {
                    let inner_node_a =
                        find_inner_node_with_distance_from_root(nodestore, child2).await;
                    let mut children = inner_node_a.children();
                    let subchild1 = children.next().unwrap();
                    let subchild2 = children.next().unwrap();
                    std::mem::drop(children);

                    let (inner_node_b, inner_node_b_parent_id) =
                        find_inner_node_with_distance_from_root_with_parent_id(
                            nodestore, subchild1,
                        )
                        .await;
                    orphaned_nodes.extend(inner_node_b.children());
                    removed_nodes.push((
                        *inner_node_b.block_id(),
                        NodeAndBlobReference::NonRootInnerNode {
                            belongs_to_blob: belongs_to_blob.clone(),
                            depth: inner_node_b.depth(),
                            parent_id: inner_node_b_parent_id,
                        },
                    ));
                    inner_node_b.upcast().remove(nodestore).await.unwrap();

                    let (inner_node_c, inner_node_c_parent_id) =
                        find_inner_node_with_distance_from_root_with_parent_id(
                            nodestore, subchild2,
                        )
                        .await;
                    let mut children_of_c = inner_node_c.children();
                    let child_of_c_id = children_of_c.next().unwrap();
                    std::mem::drop(children_of_c);

                    let child_of_c = nodestore
                        .load(child_of_c_id)
                        .await
                        .unwrap()
                        .unwrap()
                        .into_inner_node()
                        .unwrap();
                    orphaned_nodes.extend(child_of_c.children());
                    // Don't add child_of_c to removed_nodes because its parent is removed too so it's not referenced&&missing
                    child_of_c.upcast().remove(nodestore).await.unwrap();

                    orphaned_nodes.extend(
                        inner_node_c
                            .children()
                            .filter(|node_id| *node_id != child_of_c_id),
                    );
                    removed_nodes.push((
                        *inner_node_c.block_id(),
                        NodeAndBlobReference::NonRootInnerNode {
                            belongs_to_blob,
                            depth: inner_node_c.depth(),
                            parent_id: inner_node_c_parent_id,
                        },
                    ));
                    inner_node_c.upcast().remove(nodestore).await.unwrap();
                }

                RemoveSomeNodesResult {
                    removed_nodes,
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn corrupt_some_nodes_of_a_large_blob(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> CorruptSomeNodesResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let mut rng = SmallRng::seed_from_u64(0);
                    let inner_node =
                        find_an_inner_node_of_a_large_blob(nodestore, &blob_info.blob_id).await;
                    let mut children = inner_node.children();
                    let child1 = children.next().unwrap();
                    let child2 = children.next().unwrap();

                    let mut corrupted_nodes = vec![];
                    let mut orphaned_nodes = vec![];

                    let belongs_to_blob = MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
                        blob_id: blob_info.blob_id,
                        referenced_as: blob_info.referenced_as,
                    };

                    // for child1, find an inner node A. Corrupt an inner node below A, a leaf below A, and A itself.
                    {
                        let (inner_node_a, inner_node_a_parent_id) =
                            find_inner_node_with_distance_from_root_with_parent_id(
                                nodestore, child1,
                            )
                            .await;
                        let mut children = inner_node_a.children();
                        let subchild1 = children.next().unwrap();
                        let subchild2 = children.next().unwrap();
                        std::mem::drop(children);

                        let (inner_below_a, inner_below_a_parent_id) =
                            find_inner_node_with_distance_from_root_with_parent_id(
                                nodestore, subchild1,
                            )
                            .await;
                        orphaned_nodes.extend(inner_below_a.children());
                        corrupted_nodes.push((
                            *inner_below_a.block_id(),
                            [NodeAndBlobReference::NonRootInnerNode {
                                belongs_to_blob:
                                    MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot, // This is `UnreachableFromFilesystemRoot` because `inner_node_a` gets removed as well, so when cryfs-check is running, it won't be able to reach this from any blob
                                depth: inner_below_a.depth(),
                                parent_id: inner_below_a_parent_id,
                            }]
                            .into_iter()
                            .collect(),
                        ));

                        let (leaf_below_a, leaf_below_a_parent_id) =
                            find_leaf_node_with_parent_id(nodestore, subchild2, &mut rng).await;
                        // node_info is empty `[]` because `inner_node_a` gets removed as well, so when cryfs-check is running, it won't be able to figure out which node referenced the corrupted one
                        corrupted_nodes.push((
                            *leaf_below_a.block_id(),
                            [NodeAndBlobReference::NonRootLeafNode {
                                belongs_to_blob:
                                    MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot, // This is `UnreachableFromFilesystemRoot` because `inner_node_a` gets removed as well, so when cryfs-check is running, it won't be able to reach this from any blob
                                parent_id: leaf_below_a_parent_id,
                            }]
                            .into_iter()
                            .collect(),
                        ));

                        orphaned_nodes.extend(inner_node_a.children());
                        corrupted_nodes.push((
                            *inner_node_a.block_id(),
                            [NodeAndBlobReference::NonRootInnerNode {
                                belongs_to_blob: belongs_to_blob.clone(),
                                depth: inner_node_a.depth(),
                                parent_id: inner_node_a_parent_id,
                            }]
                            .into_iter()
                            .collect(),
                        ));
                    }

                    // for child2, find an inner node A. Corrupt an inner node B below A. Also corrupt an inner node C below A and its direct child. Don't corrupt A.
                    {
                        let inner_node_a =
                            find_inner_node_with_distance_from_root(nodestore, child2).await;
                        let mut children = inner_node_a.children();
                        let subchild1 = children.next().unwrap();
                        let subchild2 = children.next().unwrap();
                        std::mem::drop(children);

                        let (inner_node_b, inner_node_b_parent_id) =
                            find_inner_node_with_distance_from_root_with_parent_id(
                                nodestore, subchild1,
                            )
                            .await;
                        orphaned_nodes.extend(inner_node_b.children());
                        corrupted_nodes.push((
                            *inner_node_b.block_id(),
                            [NodeAndBlobReference::NonRootInnerNode {
                                belongs_to_blob: belongs_to_blob.clone(),
                                depth: inner_node_b.depth(),
                                parent_id: inner_node_b_parent_id,
                            }]
                            .into_iter()
                            .collect(),
                        ));

                        let (inner_node_c, inner_node_c_parent_id) =
                            find_inner_node_with_distance_from_root_with_parent_id(
                                nodestore, subchild2,
                            )
                            .await;
                        let mut children_of_c = inner_node_c.children();
                        let child_of_c_id = children_of_c.next().unwrap();
                        std::mem::drop(children_of_c);

                        let child_of_c = nodestore
                            .load(child_of_c_id)
                            .await
                            .unwrap()
                            .unwrap()
                            .into_inner_node()
                            .unwrap();
                        orphaned_nodes.extend(child_of_c.children());
                        // node_info is empty `[]` because `inner_node_c` gets removed as well, so when cryfs-check is running, it won't be able to figure out which node referenced the corrupted one
                        corrupted_nodes.push((*child_of_c.block_id(), [].into_iter().collect()));

                        orphaned_nodes.extend(inner_node_c.children());
                        corrupted_nodes.push((
                            *inner_node_c.block_id(),
                            [NodeAndBlobReference::NonRootInnerNode {
                                belongs_to_blob,
                                depth: inner_node_c.depth(),
                                parent_id: inner_node_c_parent_id,
                            }]
                            .into_iter()
                            .collect(),
                        ));
                    }

                    CorruptSomeNodesResult {
                        corrupted_nodes,
                        orphaned_nodes,
                    }
                })
            })
            .await;

        for node in &result.corrupted_nodes {
            self.corrupt_block(node.0).await;
        }
        result
    }

    pub async fn remove_a_leaf_node(&self, blob_info: BlobReferenceWithId) -> RemoveLeafNodeResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let (leaf_node, leaf_node_parent_id) =
                    find_leaf_node_of_blob_with_parent_id(nodestore, &blob_info.blob_id).await;
                let leaf_node_id = *leaf_node.block_id();
                leaf_node.upcast().remove(nodestore).await.unwrap();
                RemoveLeafNodeResult {
                    removed_node: leaf_node_id,
                    removed_node_info: NodeAndBlobReferenceFromReachableBlob {
                        blob_info: BlobReferenceWithId {
                            blob_id: blob_info.blob_id,
                            referenced_as: blob_info.referenced_as,
                        },
                        node_info: NodeReference::NonRootLeafNode {
                            parent_id: leaf_node_parent_id,
                        },
                    },
                }
            })
        })
        .await
    }

    pub async fn corrupt_a_leaf_node(
        &self,
        blob_info: BlobReferenceWithId,
    ) -> CorruptLeafNodeResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let (leaf_node, leaf_node_parent_id) =
                        find_leaf_node_of_blob_with_parent_id(nodestore, &blob_info.blob_id).await;
                    let leaf_node_id = *leaf_node.block_id();
                    CorruptLeafNodeResult {
                        corrupted_node: leaf_node_id,
                        corrupted_node_info: NodeAndBlobReferenceFromReachableBlob {
                            blob_info: BlobReferenceWithId {
                                blob_id: blob_info.blob_id,
                                referenced_as: blob_info.referenced_as,
                            },
                            node_info: NodeReference::NonRootLeafNode {
                                parent_id: leaf_node_parent_id,
                            },
                        },
                    }
                })
            })
            .await;
        self.corrupt_block(result.corrupted_node).await;
        result
    }

    pub async fn add_entries_to_make_dir_large(&self, blob_info: BlobReferenceWithId) {
        assert_eq!(BlobType::Dir, blob_info.referenced_as.blob_type);
        self.update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let dir =
                    FsBlob::into_dir(fsblobstore.load(&blob_info.blob_id).await.unwrap().unwrap())
                        .await
                        .unwrap();
                let mut dir = CreatedDirBlob::new(dir, blob_info.referenced_as.path);
                entry_helpers::add_entries_to_make_dir_large(fsblobstore, &mut dir).await;
                dir.async_drop().await.unwrap();
            })
        })
        .await;
    }

    pub async fn load_node_infos(
        &self,
        node_ids: impl Iterator<Item = BlockId> + Send + 'static,
    ) -> impl Iterator<Item = (BlockId, NodeInfoAsSeenByLookingAtNode)> {
        self.update_nodestore(move |nodestore| {
            Box::pin(async move {
                futures::future::join_all(node_ids.map(|child| async move {
                    let node_info = match nodestore.load(child).await {
                        Ok(Some(node)) => entry_helpers::load_node_info(&node),
                        Ok(None) => panic!("Node not found"),
                        Err(_) => NodeInfoAsSeenByLookingAtNode::Unreadable,
                    };
                    (child, node_info)
                }))
                .await
            })
        })
        .await
        .into_iter()
    }

    pub async fn load_node_info(&self, node_id: BlockId) -> NodeInfoAsSeenByLookingAtNode {
        self.update_nodestore(move |nodestore| {
            Box::pin(async move {
                entry_helpers::load_node_info(&nodestore.load(node_id).await.unwrap().unwrap())
            })
        })
        .await
    }
}

pub struct RemoveInnerNodeResult {
    pub removed_node: BlockId,
    pub removed_node_info: NodeAndBlobReferenceFromReachableBlob,
    pub orphaned_nodes: Vec<BlockId>,
}

pub struct CorruptInnerNodeResult {
    pub corrupted_node: BlockId,
    pub corrupted_node_info: NodeAndBlobReferenceFromReachableBlob,
    pub orphaned_nodes: Vec<BlockId>,
}

pub struct RemoveLeafNodeResult {
    pub removed_node: BlockId,
    pub removed_node_info: NodeAndBlobReferenceFromReachableBlob,
}

pub struct CorruptLeafNodeResult {
    pub corrupted_node: BlockId,
    pub corrupted_node_info: NodeAndBlobReferenceFromReachableBlob,
}

pub struct RemoveSomeNodesResult {
    pub removed_nodes: Vec<(BlockId, NodeAndBlobReference)>,
    pub orphaned_nodes: Vec<BlockId>,
}

pub struct CorruptSomeNodesResult {
    pub corrupted_nodes: Vec<(BlockId, BTreeSet<NodeAndBlobReference>)>,
    pub orphaned_nodes: Vec<BlockId>,
}

impl Debug for FilesystemFixture {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilesystemFixture")
            .field("tempdir", &self.tempdir)
            .finish()
    }
}

#[derive(Debug)]
struct FixtureTempDir {
    tempdir: TempDir,
}

impl FixtureTempDir {
    pub fn new() -> Self {
        let tempdir = TempDir::new("cryfs-check-fixture").expect("Couldn't create tempdir");
        let result = Self { tempdir };
        std::fs::create_dir(result.local_state_dir_path())
            .expect("Failed to create local state dir");
        result
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.tempdir.path().join("cryfs.config")
    }

    pub fn local_state_dir_path(&self) -> PathBuf {
        self.tempdir.path().join("local_state_dir")
    }

    pub fn local_state_dir(&self) -> LocalStateDir {
        LocalStateDir::new(self.local_state_dir_path())
    }

    pub fn create_config(&self) -> ConfigLoadResult {
        cryfs_cryfs::config::create(
            self.config_file_path().to_owned(),
            &FixedPasswordProvider::new(PASSWORD.to_owned()),
            &FixtureCreationConsole,
            &CommandLineFlags {
                missing_block_is_integrity_violation: Some(false),
                expected_cipher: None,
            },
            &self.local_state_dir(),
        )
        .expect("Failed to create config")
    }
}
