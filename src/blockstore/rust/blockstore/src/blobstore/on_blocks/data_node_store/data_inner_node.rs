use anyhow::{ensure, Result};
use std::num::{NonZeroU32, NonZeroU8};

use super::layout::{node, FORMAT_VERSION_HEADER};
use super::DataNodeStore;
use crate::blockstore::{high_level::Block, low_level::BlockStore, BlockId, BLOCKID_LEN};
use crate::data::Data;

const MAX_DEPTH: u8 = 10;

pub struct DataInnerNode<B: BlockStore + Send + Sync> {
    block: Block<B>,
}

impl<B: BlockStore + Send + Sync> DataInnerNode<B> {
    pub fn new(block: Block<B>, node_store: &DataNodeStore<B>) -> Result<Self> {
        let view = super::node::View::new(block.data());
        let depth = view.depth().read();
        assert_ne!(
            0, depth,
            "Loaded an inner node with depth 0. This doesn't make sense, it should have been loaded as a leaf node",
        );
        ensure!(
            depth <= MAX_DEPTH,
            "Loaded an inner node with depth {} but the maximum is {}",
            depth,
            MAX_DEPTH,
        );
        let size = view.size().read();
        let max_children_per_inner_node = node_store.max_children_per_inner_node();
        ensure!(
            size >= 1,
            "Loaded an inner node that claims to store {} children but the minimum is 1.",
            size,
        );
        ensure!(
            size < max_children_per_inner_node,
            "Loaded an inner node that claims to store {} children but the maximum is {}.",
            size,
            max_children_per_inner_node,
        );
        Ok(Self { block })
    }

    pub fn depth(&self) -> NonZeroU8 {
        let view = super::node::View::new(self.block.data());
        NonZeroU8::new(view.depth().read())
            .expect("DataInnerNode class invariant violated: Has depth of zero")
    }

    pub fn num_children(&self) -> NonZeroU32 {
        let view = super::node::View::new(self.block.data().as_ref());
        NonZeroU32::new(view.size().read())
            .expect("DataInnerNode class invariant violated: Has only zero children")
    }

    pub fn children<'a>(&'a self) -> impl Iterator<Item = BlockId> + ExactSizeIterator + 'a {
        let view = super::node::View::new(self.block.data().as_ref());
        let num_children = usize::try_from(view.size().read()).unwrap();
        let children_data = view.into_data().into_slice();
        let children_ids = children_data.chunks_exact(BLOCKID_LEN);
        assert!(
            num_children <= children_ids.len(),
            "Class invariant violated: Tried to load an inner node with {} children but support at most {} per inner node",
            num_children,
            children_ids.len(),
        );
        children_ids
            .take(num_children)
            .map(|id_bytes| BlockId::from_slice(id_bytes).unwrap())
    }
}

pub fn serialize_inner_node<B: BlockStore + Send + Sync>(
    depth: u8,
    children: &[BlockId],
    node_store: &DataNodeStore<B>,
) -> Data {
    assert!(
        depth != 0,
        "Inner node cannot have a depth of 0. Is this perhaps a leaf instead?"
    );
    assert!(
        depth <= MAX_DEPTH,
        "Inner node cannot have a depth of {}, the maximum is {}",
        depth,
        MAX_DEPTH,
    );
    assert!(
        children.len() >= 1,
        "Inner node must have at least one child"
    );
    assert!(
        children.len() <= usize::try_from(node_store.max_children_per_inner_node()).unwrap(),
        "Inner nodes can only store {} children but tried to store {}",
        node_store.max_children_per_inner_node(),
        children.len(),
    );

    let mut view = node::View::new(Data::from(vec![
        0;
        node_store
            .block_size_bytes
            .try_into()
            .unwrap()
    ]));
    view.format_version_header_mut()
        .write(FORMAT_VERSION_HEADER);
    view.unused_must_be_zero_mut().write(0);
    view.depth_mut().write(depth);
    view.size_mut()
        .write(u32::try_from(children.len()).unwrap());
    _serialize_children(view.data_mut(), children);
    view.into_storage()
}

fn _serialize_children(dest: &mut [u8], children: &[BlockId]) {
    assert_eq!(dest.len(), children.len() * BLOCKID_LEN, "Serializing {} children requires {} bytes but tried to serialize into a buffer with {} bytes.", children.len(), children.len() * BLOCKID_LEN, dest.len());
    for (index, child) in children.iter().enumerate() {
        // TODO Some way to avoid this copy by not using &[BlockId] or Vec<BlockId> but our own collection type that already has it aligned correctly?
        dest[(BLOCKID_LEN * index)..(BLOCKID_LEN * (index + 1))].copy_from_slice(child.data());
    }
}
