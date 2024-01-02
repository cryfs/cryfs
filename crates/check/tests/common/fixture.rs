use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNodeStore};
use cryfs_blockstore::{
    AllowIntegrityViolations, BlockId, DynBlockStore, InMemoryBlockStore, IntegrityConfig,
    LockingBlockStore, MissingBlockIsIntegrityViolation, SharedBlockStore,
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
    self, find_a_leaf_node_of_a_large_blob, find_an_inner_node_of_a_large_blob, find_inner_node,
    find_leaf_node, SomeBlobs,
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

    pub async fn remove_root_node_of_a_large_blob(&self, blob_id: BlobId) -> RemoveInnerNodeResult {
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
                    let inner_node = find_inner_node(nodestore, child1).await;
                    let mut children = inner_node.children();
                    let subchild1 = children.next().unwrap();
                    let subchild2 = children.next().unwrap();
                    std::mem::drop(children);

                    let leaf = find_leaf_node(nodestore, subchild1, &mut rng).await;
                    removed_nodes.push(*leaf.block_id());
                    leaf.upcast().remove(nodestore).await.unwrap();

                    let inner = find_inner_node(nodestore, subchild2).await;
                    orphaned_nodes.extend(inner.children());
                    removed_nodes.push(*inner.block_id());
                    inner.upcast().remove(nodestore).await.unwrap();

                    orphaned_nodes.extend(inner_node.children());
                    removed_nodes.push(*inner_node.block_id());
                    inner_node.upcast().remove(nodestore).await.unwrap();
                }

                // for child2, find an inner node A and remove it.
                {
                    let inner_node = find_inner_node(nodestore, child2).await;
                    orphaned_nodes.extend(inner_node.children());
                    removed_nodes.push(*inner_node.block_id());
                    inner_node.upcast().remove(nodestore).await.unwrap();
                }

                RemoveSomeNodesResult {
                    removed_nodes,
                    orphaned_nodes,
                }
            })
        })
        .await
    }

    pub async fn remove_a_leaf_node_of_a_large_blob(&self, blob_id: BlobId) -> BlockId {
        self.update_nodestore(|nodestore| {
            Box::pin(async move {
                let leaf_node = find_a_leaf_node_of_a_large_blob(nodestore, &blob_id).await;
                let leaf_node_id = *leaf_node.block_id();
                leaf_node.upcast().remove(nodestore).await.unwrap();
                leaf_node_id
            })
        })
        .await
    }
}

pub struct RemoveInnerNodeResult {
    pub removed_node: BlockId,
    pub orphaned_nodes: Vec<BlockId>,
}

pub struct RemoveSomeNodesResult {
    pub removed_nodes: Vec<BlockId>,
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
