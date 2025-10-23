use byte_unit::Byte;
#[cfg(feature = "slow-tests-any")]
use divrem::DivCeil;
use futures::future::BoxFuture;
#[cfg(feature = "slow-tests-any")]
use futures::{future, join};
#[cfg(feature = "slow-tests-any")]
use iter_chunks::IterChunks;
use std::fmt::Debug;

use super::super::data_node_store::{DataNodeStore, NodeLayout};
use super::{store::DataTreeStore, tree::DataTree};
use cryfs_blockstore::{
    BlockId, BlockStore, InMemoryBlockStore, LLSharedBlockStore, LockingBlockStore,
};
use cryfs_utils::{async_drop::AsyncDrop, data::Data, testutils::data_fixture::DataFixture};

pub const PHYSICAL_BLOCK_SIZE: Byte = Byte::from_u64(128);

pub struct TreeFixture {
    root_id: BlockId,
    data_seed: u64,
    num_bytes: usize,
}

impl TreeFixture {
    pub async fn create_tree_with_data<
        B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync,
    >(
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

    pub async fn create_tree_with_data_and_id<
        B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync,
    >(
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

    pub async fn assert_data_is_still_intact<
        B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync,
    >(
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
    DataFixture::new(seed).get(size).into()
}

pub async fn create_one_leaf_tree<
    B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync,
>(
    store: &DataTreeStore<B>,
) -> DataTree<B> {
    store.create_tree().await.unwrap()
}

pub async fn create_multi_leaf_tree<
    B: BlockStore<Block: Send + Sync> + AsyncDrop + Debug + Send + Sync,
>(
    store: &DataTreeStore<B>,
    num_leaves: u64,
) -> DataTree<B> {
    let mut tree = store.create_tree().await.unwrap();
    tree.resize_num_bytes(num_leaves * store.logical_block_size_bytes().as_u64())
        .await
        .unwrap();
    tree
}

#[cfg(feature = "slow-tests-any")]
pub async fn manually_create_tree<B: LLBlockStore + Send + Sync>(
    nodestore: &DataNodeStore<B>,
    num_full_leaves: u64,
    last_leaf_num_bytes: u64,
    leaf_data: impl Fn(u64, usize) -> Data,
) -> BlockId {
    // First, create all leaves
    let leaves = {
        let full_leaves_future = future::join_all((0..num_full_leaves).map(async |leaf_index| {
            let offset = leaf_index * nodestore.layout().max_bytes_per_leaf() as u64;
            let leaf_data = leaf_data(offset, nodestore.layout().max_bytes_per_leaf() as usize);
            *nodestore
                .create_new_leaf_node(&leaf_data)
                .await
                .unwrap()
                .block_id()
        }));
        let last_leaf_offset = num_full_leaves * nodestore.layout().max_bytes_per_leaf() as u64;
        let last_leaf_data = leaf_data(last_leaf_offset, last_leaf_num_bytes as usize);
        let last_leaf_future = async {
            *nodestore
                .create_new_leaf_node(&last_leaf_data)
                .await
                .unwrap()
                .block_id()
        };
        let (mut leaves, last_leaf) = join!(full_leaves_future, last_leaf_future);
        leaves.push(last_leaf);
        leaves
    };

    // Second, combine them into inner nodes until there's only one root node left
    let mut nodes = leaves;
    let mut depth = 1;
    while nodes.len() > 1 {
        let mut inner_nodes = Vec::with_capacity(DivCeil::div_ceil(
            nodes.len(),
            nodestore.layout().max_children_per_inner_node() as usize,
        ));
        let mut chunks = nodes
            .into_iter()
            .chunks(nodestore.layout().max_children_per_inner_node() as usize);
        while let Some(chunk) = chunks.next() {
            let children = chunk.collect::<Vec<_>>();
            if children.len() < nodestore.layout().max_children_per_inner_node() as usize {
                assert!(chunks.next().is_none());
                if children.is_empty() {
                    break;
                }
            }
            inner_nodes.push(async move {
                *nodestore
                    .create_new_inner_node(depth, &children)
                    .await
                    .unwrap()
                    .block_id()
            });
        }
        nodes = future::join_all(inner_nodes).await;
        depth += 1;
    }
    nodes[0]
}

pub async fn with_treestore(
    f: impl FnOnce(&DataTreeStore<LockingBlockStore<InMemoryBlockStore>>) -> BoxFuture<'_, ()>,
) {
    with_treestore_with_blocksize(PHYSICAL_BLOCK_SIZE, f).await
}

pub async fn with_treestore_with_blocksize(
    blocksize_bytes: Byte,
    f: impl FnOnce(&DataTreeStore<LockingBlockStore<InMemoryBlockStore>>) -> BoxFuture<'_, ()>,
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
        &'a DataTreeStore<LockingBlockStore<LLSharedBlockStore<InMemoryBlockStore>>>,
        &'a DataNodeStore<LockingBlockStore<LLSharedBlockStore<InMemoryBlockStore>>>,
    ) -> BoxFuture<'a, ()>,
) {
    with_treestore_and_nodestore_with_blocksize(PHYSICAL_BLOCK_SIZE, f).await
}

pub async fn with_treestore_and_nodestore_with_blocksize(
    blocksize: Byte,
    f: impl for<'a> FnOnce(
        &'a DataTreeStore<LockingBlockStore<LLSharedBlockStore<InMemoryBlockStore>>>,
        &'a DataNodeStore<LockingBlockStore<LLSharedBlockStore<InMemoryBlockStore>>>,
    ) -> BoxFuture<'a, ()>,
) {
    let blockstore = LLSharedBlockStore::new(InMemoryBlockStore::new());
    let mut nodestore = DataNodeStore::new(
        LockingBlockStore::new(LLSharedBlockStore::clone(&blockstore)),
        blocksize,
    )
    .await
    .unwrap();
    let mut treestore = DataTreeStore::new(LockingBlockStore::new(blockstore), blocksize)
        .await
        .unwrap();
    f(&treestore, &nodestore).await;
    treestore.async_drop().await.unwrap();
    nodestore.async_drop().await.unwrap();
}

// TODO Replace with std::num::div_ceil once it's stable
const fn div_ceil(numerator: u64, denominator: u64) -> u64 {
    (numerator + denominator - 1) / denominator
}

pub const fn expected_num_nodes_for_num_leaves(num_leaves: u64, layout: NodeLayout) -> u64 {
    let mut num_nodes = 0;
    let mut num_nodes_current_level = num_leaves;
    while num_nodes_current_level > 1 {
        num_nodes += num_nodes_current_level;
        num_nodes_current_level = div_ceil(
            num_nodes_current_level,
            layout.max_children_per_inner_node() as u64,
        );
    }
    if 1 != num_nodes_current_level {
        // TODO Use assert_eq instead once that works in const fn
        panic!("assertion failed");
    }
    num_nodes += 1;
    num_nodes
}

pub const fn expected_depth_for_num_leaves(num_leaves: u64, layout: NodeLayout) -> u8 {
    assert!(num_leaves > 0);
    let mut depth = 0;
    let mut num_nodes_current_level = num_leaves;
    while num_nodes_current_level > 1 {
        depth += 1;
        num_nodes_current_level = div_ceil(
            num_nodes_current_level,
            layout.max_children_per_inner_node() as u64,
        );
    }
    if 1 != num_nodes_current_level {
        // TODO Use assert_eq instead once that works in const fn
        panic!("assertion failed");
    }
    depth
}

#[cfg(feature = "slow-tests-4")]
pub fn expected_depth_for_num_bytes(num_bytes: u64, layout: NodeLayout) -> u8 {
    let num_leaves = DivCeil::div_ceil(num_bytes, layout.max_bytes_per_leaf() as u64).max(1);
    expected_depth_for_num_leaves(num_leaves, layout)
}
