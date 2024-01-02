use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNodeStore};
use cryfs_blockstore::{
    AllowIntegrityViolations, BlockId, BlockStoreReader, BlockStoreWriter, DynBlockStore,
    InMemoryBlockStore, IntegrityConfig, LockingBlockStore, MissingBlockIsIntegrityViolation,
    SharedBlockStore,
};
use cryfs_check::CorruptedError;
use cryfs_cli_utils::setup_blockstore_stack_dyn;
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadResult, FixedPasswordProvider},
    filesystem::fsblobstore::{FsBlob, FsBlobStore},
    localstate::LocalStateDir,
};
use cryfs_utils::async_drop::{AsyncDropGuard, SyncDrop};
use futures::{
    future::{self, BoxFuture},
    stream::{self, BoxStream, StreamExt},
    Future, FutureExt,
};
use rand::{rngs::SmallRng, seq::SliceRandom, SeedableRng};
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;
use tempdir::TempDir;

use super::console::FixtureCreationConsole;
use super::entry_helpers::{
    self, find_an_inner_node_of_a_large_blob, find_an_inner_node_of_a_small_blob,
    find_inner_node_with_distance_from_root, find_leaf_node, find_leaf_node_of_blob, SomeBlobs,
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
        let mut fsblobstore = self.make_blobstore().await;
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
                on_integrity_violation: Box::new(|err| {
                    panic!("integrity violation");
                }),
            },
        )
        .await
        .expect("Failed to setup blockstore stack")
    }

    async fn make_blobstore(
        &self,
    ) -> AsyncDropGuard<FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>> {
        let blockstore = self.make_locking_blockstore().await;

        FsBlobStore::new(
            BlobStoreOnBlocks::new(
                blockstore,
                // TODO Change type in config instead of doing u32::try_from
                u32::try_from(self.config.config.config().blocksize_bytes).unwrap(),
            )
            .await
            .expect("Failed to create BlobStoreOnBlocks"),
        )
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

    pub async fn update_fsblobstore<R>(
        &self,
        update_fn: impl for<'b> FnOnce(
            &'b FsBlobStore<BlobStoreOnBlocks<DynBlockStore>>,
        ) -> BoxFuture<'b, R>,
    ) -> R {
        let mut fsblobstore = self.make_blobstore().await;
        let result = update_fn(&fsblobstore).await;
        fsblobstore.async_drop().await.unwrap();
        result
    }

    pub async fn run_cryfs_check(self) -> Vec<CorruptedError> {
        // TODO Console output is very chaotic here because the progress bars are all displayed. Let's suppress them.
        cryfs_check::check_filesystem(
            self.blockstore.into_inner_dont_drop(),
            &self.tempdir.config_file_path(),
            &self.tempdir.local_state_dir(),
            &FixedPasswordProvider::new(PASSWORD.to_owned()),
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
                let mut root = FsBlob::into_dir(blobstore.load(&root_id).await.unwrap().unwrap())
                    .await
                    .unwrap();
                let result = super::entry_helpers::create_some_blobs(blobstore, &mut root).await;
                root.async_drop().await.unwrap();
                result
            })
        })
        .await
    }

    pub async fn create_empty_file(&self) -> BlobId {
        let root_id = self.root_blob_id;
        self.update_fsblobstore(move |blobstore| {
            Box::pin(async move {
                let mut root = FsBlob::into_dir(blobstore.load(&root_id).await.unwrap().unwrap())
                    .await
                    .unwrap();
                let result =
                    super::entry_helpers::create_empty_file(blobstore, &mut root, "file_name")
                        .await;
                root.async_drop().await.unwrap();
                result.blob_id()
            })
        })
        .await
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

    pub async fn get_descendants_of_dir_blob<'a>(&'a self, dir_blob: BlobId) -> Vec<BlobId> {
        self.update_fsblobstore(move |fsblobstore| {
            Box::pin(
                entry_helpers::get_descendants_of_dir_blob(fsblobstore, dir_blob)
                    .collect::<Vec<BlobId>>(),
            )
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

    pub async fn remove_root_node_of_blob(&self, blob_id: BlobId) -> RemoveInnerNodeResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let blob_root_node = nodestore
                    .load(*blob_id.to_root_block_id())
                    .await
                    .unwrap()
                    .unwrap()
                    .into_inner_node()
                    .expect("test blob too small to have more than one node. We need to change the test and increase its size");
                let orphaned_nodes = blob_root_node.children().collect::<Vec<_>>();
                let inner_node_id = *blob_root_node.block_id();
                assert_eq!(blob_id.to_root_block_id(), blob_root_node.block_id());
                blob_root_node.upcast().remove(nodestore).await.unwrap();
                RemoveInnerNodeResult {
                    removed_node: inner_node_id,
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn corrupt_root_node_of_blob(&self, blob_id: BlobId) -> CorruptInnerNodeResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let blob_root_node = nodestore
                        .load(*blob_id.to_root_block_id())
                        .await
                        .unwrap()
                        .unwrap();
                    assert_eq!(blob_id.to_root_block_id(), blob_root_node.block_id());
                    let orphaned_nodes =
                        if let Some(blob_root_node) = blob_root_node.into_inner_node() {
                            blob_root_node.children().collect::<Vec<_>>()
                        } else {
                            vec![]
                        };

                    CorruptInnerNodeResult {
                        corrupted_node: *blob_id.to_root_block_id(),
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
        blob_id: BlobId,
    ) -> RemoveInnerNodeResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let inner_node = find_an_inner_node_of_a_large_blob(nodestore, &blob_id).await;
                let orphaned_nodes = inner_node.children().collect::<Vec<_>>();
                let inner_node_id = *inner_node.block_id();
                inner_node.upcast().remove(nodestore).await.unwrap();
                RemoveInnerNodeResult {
                    removed_node: inner_node_id,
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn corrupt_an_inner_node_of_a_large_blob(
        &self,
        blob_id: BlobId,
    ) -> CorruptInnerNodeResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let inner_node = find_an_inner_node_of_a_large_blob(nodestore, &blob_id).await;
                    let orphaned_nodes = inner_node.children().collect::<Vec<_>>();
                    let inner_node_id = *inner_node.block_id();
                    CorruptInnerNodeResult {
                        corrupted_node: inner_node_id,
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
        blob_id: BlobId,
    ) -> RemoveInnerNodeResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let inner_node = find_an_inner_node_of_a_small_blob(nodestore, &blob_id).await;
                let orphaned_nodes = inner_node.children().collect::<Vec<_>>();
                let inner_node_id = *inner_node.block_id();
                inner_node.upcast().remove(nodestore).await.unwrap();
                RemoveInnerNodeResult {
                    removed_node: inner_node_id,
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn remove_some_nodes_of_a_large_blob(
        &self,
        blob_id: BlobId,
    ) -> RemoveSomeNodesResult {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let mut rng = SmallRng::seed_from_u64(0);
                let inner_node = find_an_inner_node_of_a_large_blob(nodestore, &blob_id).await;
                let mut children = inner_node.children();
                let child1 = children.next().unwrap();
                let child2 = children.next().unwrap();

                let mut removed_nodes = vec![];
                let mut orphaned_nodes = vec![];

                // for child1, find an inner node A. Remove an inner node below A, a leaf below A, and A itself.
                {
                    let inner_node_a =
                        find_inner_node_with_distance_from_root(nodestore, child1).await;
                    let mut children = inner_node_a.children();
                    let subchild1 = children.next().unwrap();
                    let subchild2 = children.next().unwrap();
                    std::mem::drop(children);

                    let inner_below_a =
                        find_inner_node_with_distance_from_root(nodestore, subchild1).await;
                    orphaned_nodes.extend(inner_below_a.children());
                    removed_nodes.push(*inner_below_a.block_id());
                    inner_below_a.upcast().remove(nodestore).await.unwrap();

                    let leaf_below_a = find_leaf_node(nodestore, subchild2, &mut rng).await;
                    removed_nodes.push(*leaf_below_a.block_id());
                    leaf_below_a.upcast().remove(nodestore).await.unwrap();

                    orphaned_nodes.extend(inner_node_a.children());
                    removed_nodes.push(*inner_node_a.block_id());
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

                    let inner_node_b =
                        find_inner_node_with_distance_from_root(nodestore, subchild1).await;
                    orphaned_nodes.extend(inner_node_b.children());
                    removed_nodes.push(*inner_node_b.block_id());
                    inner_node_b.upcast().remove(nodestore).await.unwrap();

                    let inner_node_c =
                        find_inner_node_with_distance_from_root(nodestore, subchild2).await;
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
                    removed_nodes.push(*inner_node_c.block_id());
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
        blob_id: BlobId,
    ) -> CorruptSomeNodesResult {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let mut rng = SmallRng::seed_from_u64(0);
                    let inner_node = find_an_inner_node_of_a_large_blob(nodestore, &blob_id).await;
                    let mut children = inner_node.children();
                    let child1 = children.next().unwrap();
                    let child2 = children.next().unwrap();

                    let mut corrupted_nodes = vec![];
                    let mut orphaned_nodes = vec![];

                    // for child1, find an inner node A. Corrupt an inner node below A, a leaf below A, and A itself.
                    {
                        let inner_node_a =
                            find_inner_node_with_distance_from_root(nodestore, child1).await;
                        let mut children = inner_node_a.children();
                        let subchild1 = children.next().unwrap();
                        let subchild2 = children.next().unwrap();
                        std::mem::drop(children);

                        let inner_below_a =
                            find_inner_node_with_distance_from_root(nodestore, subchild1).await;
                        orphaned_nodes.extend(inner_below_a.children());
                        corrupted_nodes.push(*inner_below_a.block_id());

                        let leaf_below_a = find_leaf_node(nodestore, subchild2, &mut rng).await;
                        corrupted_nodes.push(*leaf_below_a.block_id());

                        orphaned_nodes.extend(inner_node_a.children());
                        corrupted_nodes.push(*inner_node_a.block_id());
                    }

                    // for child2, find an inner node A. Corrupt an inner node B below A. Also corrupt an inner node C below A and its direct child. Don't corrupt A.
                    {
                        let inner_node_a =
                            find_inner_node_with_distance_from_root(nodestore, child2).await;
                        let mut children = inner_node_a.children();
                        let subchild1 = children.next().unwrap();
                        let subchild2 = children.next().unwrap();
                        std::mem::drop(children);

                        let inner_node_b =
                            find_inner_node_with_distance_from_root(nodestore, subchild1).await;
                        orphaned_nodes.extend(inner_node_b.children());
                        corrupted_nodes.push(*inner_node_b.block_id());

                        let inner_node_c =
                            find_inner_node_with_distance_from_root(nodestore, subchild2).await;
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
                        corrupted_nodes.push(*child_of_c.block_id());

                        orphaned_nodes.extend(inner_node_c.children());
                        corrupted_nodes.push(*inner_node_c.block_id());
                    }

                    CorruptSomeNodesResult {
                        corrupted_nodes,
                        orphaned_nodes,
                    }
                })
            })
            .await;

        for node in &result.corrupted_nodes {
            self.corrupt_block(*node).await;
        }
        result
    }

    pub async fn remove_a_leaf_node(&self, blob_id: BlobId) -> BlockId {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let leaf_node = find_leaf_node_of_blob(nodestore, &blob_id).await;
                let leaf_node_id = *leaf_node.block_id();
                leaf_node.upcast().remove(nodestore).await.unwrap();
                leaf_node_id
            })
        })
        .await
    }

    pub async fn corrupt_a_leaf_node(&self, blob_id: BlobId) -> BlockId {
        let result = self
            .update_nodestore(|nodestore| {
                Box::pin(async move {
                    let leaf_node = find_leaf_node_of_blob(nodestore, &blob_id).await;
                    let leaf_node_id = *leaf_node.block_id();
                    leaf_node_id
                })
            })
            .await;
        self.corrupt_block(result).await;
        result
    }

    pub async fn add_entries_to_make_dir_large(&self, blob_id: BlobId) {
        self.update_fsblobstore(|fsblobstore| {
            Box::pin(async move {
                let mut dir = FsBlob::into_dir(fsblobstore.load(&blob_id).await.unwrap().unwrap())
                    .await
                    .unwrap();
                entry_helpers::add_entries_to_make_dir_large(fsblobstore, &mut dir).await;
                dir.async_drop().await.unwrap();
            })
        })
        .await;
    }
}

pub struct RemoveInnerNodeResult {
    pub removed_node: BlockId,
    pub orphaned_nodes: Vec<BlockId>,
}

pub struct CorruptInnerNodeResult {
    pub corrupted_node: BlockId,
    pub orphaned_nodes: Vec<BlockId>,
}

pub struct RemoveSomeNodesResult {
    pub removed_nodes: Vec<BlockId>,
    pub orphaned_nodes: Vec<BlockId>,
}

pub struct CorruptSomeNodesResult {
    pub corrupted_nodes: Vec<BlockId>,
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
