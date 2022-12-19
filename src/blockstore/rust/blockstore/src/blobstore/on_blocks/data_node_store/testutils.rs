use anyhow::Result;
use futures::{future::BoxFuture, join};

use super::{DataInnerNode, DataNode, DataLeafNode, DataNodeStore};
use crate::blockstore::{BlockId, high_level::LockingBlockStore, low_level::inmemory::InMemoryBlockStore};

pub const PHYSICAL_BLOCK_SIZE_BYTES: u32 = 1024;

pub async fn new_leaf_node(nodestore: &DataNodeStore<InMemoryBlockStore>) -> Result<DataLeafNode<InMemoryBlockStore>> {
    let data = vec![0u8; nodestore.layout().max_bytes_per_leaf() as usize].into();
    nodestore.create_new_leaf_node(&data).await
}

pub async fn new_inner_node(nodestore: &DataNodeStore<InMemoryBlockStore>) -> Result<DataInnerNode<InMemoryBlockStore>> {
    let leaf1_data = vec![0u8; nodestore.layout().max_bytes_per_leaf() as usize].into();
    let leaf2_data = vec![0u8; 5].into();
    let (leaf1, leaf2) = join!(
        nodestore.create_new_leaf_node(&leaf1_data),
        nodestore.create_new_leaf_node(&leaf2_data),
    );
    nodestore.create_new_inner_node(1, &[*leaf1?.block_id(), *leaf2?.block_id()]).await
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
