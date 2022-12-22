use futures::future;
use futures::{future::BoxFuture, join};
use rand::{rngs::SmallRng, Rng, SeedableRng};

use super::{DataInnerNode, DataLeafNode, DataNode, DataNodeStore, NodeLayout};
use cryfs_blockstore::{BlockId, InMemoryBlockStore, LockingBlockStore};
use cryfs_utils::data::Data;

pub const PHYSICAL_BLOCK_SIZE_BYTES: u32 = 1024;

pub async fn new_full_leaf_node(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
) -> DataLeafNode<InMemoryBlockStore> {
    nodestore
        .create_new_leaf_node(&full_leaf_data(1))
        .await
        .unwrap()
}

pub async fn new_empty_leaf_node(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
) -> DataLeafNode<InMemoryBlockStore> {
    nodestore
        .create_new_leaf_node(&vec![].into())
        .await
        .unwrap()
}

pub async fn new_inner_node(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
) -> DataInnerNode<InMemoryBlockStore> {
    let leaf1_data = full_leaf_data(1);
    let leaf2_data = half_full_leaf_data(2);
    let (leaf1, leaf2) = join!(
        nodestore.create_new_leaf_node(&leaf1_data),
        nodestore.create_new_leaf_node(&leaf2_data),
    );
    nodestore
        .create_new_inner_node(1, &[*leaf1.unwrap().block_id(), *leaf2.unwrap().block_id()])
        .await
        .unwrap()
}

pub async fn new_full_inner_node(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
) -> DataInnerNode<InMemoryBlockStore> {
    let leaves = future::join_all(
        (0..NodeLayout {
            block_size_bytes: PHYSICAL_BLOCK_SIZE_BYTES,
        }
        .max_children_per_inner_node())
            .map(|_| new_full_leaf_node(&nodestore))
            .collect::<Vec<_>>(),
    )
    .await
    .into_iter()
    .map(|node| *node.block_id())
    .collect::<Vec<_>>();
    nodestore.create_new_inner_node(1, &leaves).await.unwrap()
}

pub async fn load_node(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
    block_id: BlockId,
) -> DataNode<InMemoryBlockStore> {
    nodestore.load(block_id).await.unwrap().unwrap()
}

pub async fn load_inner_node(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
    block_id: BlockId,
) -> DataInnerNode<InMemoryBlockStore> {
    let DataNode::Inner(inner) =  nodestore.load(block_id).await.unwrap().unwrap() else {
        panic!("Expected to load an inner node but got a leaf node instead");
    };
    inner
}

pub async fn load_leaf_node(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
    block_id: BlockId,
) -> DataLeafNode<InMemoryBlockStore> {
    let DataNode::Leaf(leaf) =  nodestore.load(block_id).await.unwrap().unwrap() else {
        panic!("Expected to load a leaf node but got an inner node instead");
    };
    leaf
}

pub async fn with_nodestore(
    f: impl FnOnce(&DataNodeStore<InMemoryBlockStore>) -> BoxFuture<'_, ()>,
) {
    let mut nodestore = DataNodeStore::new(
        LockingBlockStore::new(InMemoryBlockStore::new()),
        PHYSICAL_BLOCK_SIZE_BYTES,
    )
    .await
    .unwrap();
    f(&nodestore).await;
    nodestore.async_drop().await.unwrap();
}

// pub async fn with_block_and_nodestore<'a, 'b, 'c>(
//     f: impl FnOnce(
//         &'a SharedBlockStore<InMemoryBlockStore>,
//         &'b DataNodeStore<SharedBlockStore<InMemoryBlockStore>>,
//     ) -> BoxFuture<'c, ()>,
// ) where
//     'a: 'c,
//     'b: 'c,
// {
//     let mut blockstore = SharedBlockStore::new(InMemoryBlockStore::new());
//     let mut nodestore = DataNodeStore::new(
//         LockingBlockStore::new(SharedBlockStore::clone(&blockstore)),
//         PHYSICAL_BLOCK_SIZE_BYTES,
//     )
//     .unwrap();
//     let _ = f(&blockstore, &nodestore).await;
//     nodestore.async_drop().await.unwrap();
//     blockstore.async_drop().await.unwrap();
// }

pub fn half_full_leaf_data(seed: u64) -> Data {
    let len = NodeLayout {
        block_size_bytes: PHYSICAL_BLOCK_SIZE_BYTES,
    }
    .max_bytes_per_leaf() as usize
        / 2;
    data_fixture(len, seed)
}

pub fn full_leaf_data(seed: u64) -> Data {
    let len = NodeLayout {
        block_size_bytes: PHYSICAL_BLOCK_SIZE_BYTES,
    }
    .max_bytes_per_leaf() as usize;
    data_fixture(len, seed)
}

pub fn data_fixture(len: usize, seed: u64) -> Data {
    let mut data = Data::from(vec![0u8; len]);
    let mut rng = SmallRng::seed_from_u64(seed);
    rng.fill(data.as_mut());
    data
}

pub async fn new_full_leaves(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
    num: u32,
) -> Vec<BlockId> {
    future::join_all(
        (0..num)
            .map(|_| new_full_leaf_node(nodestore))
            .collect::<Vec<_>>(),
    )
    .await
    .into_iter()
    .map(|n| *n.block_id())
    .collect::<Vec<_>>()
}

pub async fn new_inner_nodes(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
    num: u32,
) -> Vec<BlockId> {
    future::join_all(
        (0..num)
            .map(|_| new_inner_node(nodestore))
            .collect::<Vec<_>>(),
    )
    .await
    .into_iter()
    .map(|n| *n.block_id())
    .collect::<Vec<_>>()
}

pub async fn assert_full_inner_node_is_valid(
    nodestore: &DataNodeStore<InMemoryBlockStore>,
    inner_node_id: BlockId,
) {
    let inner = load_inner_node(nodestore, inner_node_id).await;
    assert_eq!(1, inner.depth().get());
    assert_eq!(
        nodestore.layout().max_children_per_inner_node(),
        inner.num_children().get(),
    );
    for child_id in inner.children() {
        load_leaf_node(nodestore, child_id).await;
    }
}
