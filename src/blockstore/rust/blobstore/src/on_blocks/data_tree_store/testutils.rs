use futures::future::BoxFuture;
use rand::{rngs::StdRng, RngCore, SeedableRng};

use cryfs_blockstore::{
    BlockId, BlockStore, InMemoryBlockStore, LockingBlockStore, SharedBlockStore,
};
use cryfs_utils::data::Data;

use super::super::data_node_store::DataNodeStore;
use super::{store::DataTreeStore, tree::DataTree};

pub const PHYSICAL_BLOCK_SIZE_BYTES: u32 = 1024;

pub struct TreeFixture {
    root_id: BlockId,
    data_seed: u64,
    num_bytes: usize,
}

impl TreeFixture {
    pub async fn create_tree_with_data<B: BlockStore + Send + Sync>(
        store: &DataTreeStore<B>,
        num_bytes: usize,
        data_seed: u64,
    ) -> Self {
        let mut tree = store.create_tree().await.unwrap();
        tree.write_bytes(data(num_bytes, data_seed).as_ref(), 0)
            .await
            .unwrap();
        TreeFixture {
            root_id: *tree.root_node_id(),
            data_seed,
            num_bytes,
        }
    }

    pub async fn create_tree_with_data_and_id<B: BlockStore + Send + Sync>(
        store: &DataTreeStore<B>,
        id: BlockId,
        num_bytes: usize,
        data_seed: u64,
    ) -> Self {
        let mut tree = store.try_create_tree(id).await.unwrap().unwrap();
        tree.write_bytes(data(num_bytes, data_seed).as_ref(), 0)
            .await
            .unwrap();
        TreeFixture {
            root_id: *tree.root_node_id(),
            data_seed,
            num_bytes,
        }
    }

    pub async fn assert_data_is_still_intact<B: BlockStore + Send + Sync>(
        &self,
        store: &DataTreeStore<B>,
    ) {
        let mut tree = store.load_tree(self.root_id).await.unwrap().unwrap();
        assert_eq!(self.num_bytes as u64, tree.num_bytes().await.unwrap());
        let mut target = vec![0; self.num_bytes];
        tree.read_bytes(0, target.as_mut()).await.unwrap();
        assert_eq!(data(self.num_bytes, self.data_seed).as_ref(), &target);
    }
}

// TODO We have this function copied over several times in the code base.
//      We should probably move it to a common place.
fn data(size: usize, seed: u64) -> Data {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut res = vec![0; size];
    rng.fill_bytes(&mut res);
    res.into()
}

pub async fn create_one_leaf_tree<B: BlockStore + Send + Sync>(
    store: &DataTreeStore<B>,
) -> DataTree<B> {
    store.create_tree().await.unwrap()
}

pub async fn create_multi_leaf_tree<B: BlockStore + Send + Sync>(
    store: &DataTreeStore<B>,
    num_leaves: u64,
) -> DataTree<B> {
    let mut tree = store.create_tree().await.unwrap();
    tree.resize_num_bytes(num_leaves * store.virtual_block_size_bytes() as u64)
        .await
        .unwrap();
    tree
}

pub async fn with_treestore(
    f: impl FnOnce(&DataTreeStore<InMemoryBlockStore>) -> BoxFuture<'_, ()>,
) {
    with_treestore_with_blocksize(PHYSICAL_BLOCK_SIZE_BYTES, f).await
}

pub async fn with_treestore_with_blocksize(
    blocksize_bytes: u32,
    f: impl FnOnce(&DataTreeStore<InMemoryBlockStore>) -> BoxFuture<'_, ()>,
) {
    let mut treestore = DataTreeStore::new(
        LockingBlockStore::new(InMemoryBlockStore::new()),
        blocksize_bytes,
    )
    .await
    .unwrap();
    f(&treestore).await;
    treestore.async_drop().await.unwrap();
}

pub async fn with_treestore_and_nodestore(
    f: impl for<'a> FnOnce(
        &'a DataTreeStore<SharedBlockStore<InMemoryBlockStore>>,
        &'a DataNodeStore<SharedBlockStore<InMemoryBlockStore>>,
    ) -> BoxFuture<'a, ()>,
) {
    let blockstore = SharedBlockStore::new(InMemoryBlockStore::new());
    let mut nodestore = DataNodeStore::new(
        LockingBlockStore::new(SharedBlockStore::clone(&blockstore)),
        PHYSICAL_BLOCK_SIZE_BYTES,
    )
    .await
    .unwrap();
    let mut treestore = DataTreeStore::new(
        LockingBlockStore::new(blockstore),
        PHYSICAL_BLOCK_SIZE_BYTES,
    )
    .await
    .unwrap();
    f(&treestore, &nodestore).await;
    treestore.async_drop().await.unwrap();
    nodestore.async_drop().await.unwrap();
}
