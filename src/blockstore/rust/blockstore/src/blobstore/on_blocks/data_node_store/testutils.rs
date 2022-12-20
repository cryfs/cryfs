use anyhow::Result;
use futures::{future::BoxFuture, join};
use rand::{rngs::SmallRng, SeedableRng, Rng};

use super::{DataInnerNode, DataNode, NodeLayout, DataLeafNode, DataNodeStore};
use crate::data::Data;
use crate::blockstore::{BlockId, high_level::LockingBlockStore, low_level::inmemory::InMemoryBlockStore};

pub const PHYSICAL_BLOCK_SIZE_BYTES: u32 = 1024;

pub async fn new_leaf_node(nodestore: &DataNodeStore<InMemoryBlockStore>) -> DataLeafNode<InMemoryBlockStore> {
    nodestore.create_new_leaf_node(&full_leaf_data(1)).await.unwrap()
}

pub async fn new_inner_node(nodestore: &DataNodeStore<InMemoryBlockStore>) -> DataInnerNode<InMemoryBlockStore> {
    let leaf1_data = full_leaf_data(1);
    let leaf2_data = half_full_leaf_data(2);
    let (leaf1, leaf2) = join!(
        nodestore.create_new_leaf_node(&leaf1_data),
        nodestore.create_new_leaf_node(&leaf2_data),
    );
    nodestore.create_new_inner_node(1, &[*leaf1.unwrap().block_id(), *leaf2.unwrap().block_id()]).await.unwrap()
}

pub async fn load_node(nodestore: &DataNodeStore<InMemoryBlockStore>, block_id: BlockId) -> DataNode<InMemoryBlockStore> {
    nodestore.load(block_id).await.unwrap().unwrap()
}

pub async fn load_inner_node(nodestore: &DataNodeStore<InMemoryBlockStore>, block_id: BlockId) -> DataInnerNode<InMemoryBlockStore> {
    let DataNode::Inner(inner) =  nodestore.load(block_id).await.unwrap().unwrap() else {
        panic!("Expected to load an inner node but got a leaf node instead");
    };
    inner
}

pub async fn load_leaf_node(nodestore: &DataNodeStore<InMemoryBlockStore>, block_id: BlockId) -> DataLeafNode<InMemoryBlockStore> {
    let DataNode::Leaf(leaf) =  nodestore.load(block_id).await.unwrap().unwrap() else {
        panic!("Expected to load a leaf node but got an inner node instead");
    };
    leaf
}

pub async fn with_nodestore(f: impl FnOnce(&DataNodeStore<InMemoryBlockStore>) -> BoxFuture<'_, ()>)
{
    let mut nodestore = DataNodeStore::new(LockingBlockStore::new(InMemoryBlockStore::new()), PHYSICAL_BLOCK_SIZE_BYTES).unwrap();
    f(&nodestore).await;
    nodestore.async_drop().await.unwrap();
}

pub fn half_full_leaf_data(seed: u64) -> Data {
    let size = NodeLayout{block_size_bytes: PHYSICAL_BLOCK_SIZE_BYTES}.max_bytes_per_leaf() as usize / 2;
    let mut data = Data::from(vec![0u8; size]);
    let mut rng = SmallRng::seed_from_u64(seed);
    rng.fill(data.as_mut());
    data
}

pub fn full_leaf_data(seed: u64) -> Data {
    let size = NodeLayout{block_size_bytes: PHYSICAL_BLOCK_SIZE_BYTES}.max_bytes_per_leaf() as usize;
    let mut data = Data::from(vec![0u8; size]);
    let mut rng = SmallRng::seed_from_u64(seed);
    rng.fill(data.as_mut());
    data
}
