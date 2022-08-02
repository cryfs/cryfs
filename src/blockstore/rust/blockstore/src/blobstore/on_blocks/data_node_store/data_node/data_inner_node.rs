use anyhow::{anyhow, ensure, Result};
use binary_layout::Field;
use std::num::{NonZeroU32, NonZeroU8};

use super::super::layout::{node, NodeLayout, FORMAT_VERSION_HEADER};
use super::DataNode;
use crate::blockstore::{high_level::Block, low_level::BlockStore, BlockId, BLOCKID_LEN};
use crate::data::Data;

const MAX_DEPTH: u8 = 10;

pub struct DataInnerNode<B: BlockStore + Send + Sync> {
    block: Block<B>,
}

impl<B: BlockStore + Send + Sync> DataInnerNode<B> {
    pub fn new(block: Block<B>, layout: &NodeLayout) -> Result<Self> {
        let view = node::View::new(block.data());
        let depth = view.depth().read();
        assert_ne!(
            0, depth,
            "Loaded an inner node with depth 0. This doesn't make sense, it should have been loaded as a leaf node",
        );
        // Min block size: enough for header and for inner nodes to have at least two children and form a tree.
        let min_block_size = usize::try_from(node::data::OFFSET + 2 * BLOCKID_LEN).unwrap();
        assert!(block.data().len() >= min_block_size, "Block doesn't have enough space for header and two children. This should have been checked before calling DataInnerNode::new");
        ensure!(
            depth <= MAX_DEPTH,
            "Loaded an inner node with depth {} but the maximum is {}",
            depth,
            MAX_DEPTH,
        );
        let size = view.size().read();
        let max_children_per_inner_node = layout.max_children_per_inner_node();
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
        let view = node::View::new(self.block.data());
        NonZeroU8::new(view.depth().read())
            .expect("DataInnerNode class invariant violated: Has depth of zero")
    }

    pub fn block_id(&self) -> &BlockId {
        self.block.block_id()
    }

    pub(super) fn raw_blockdata(&self) -> &Data {
        self.block.data()
    }

    pub(super) fn into_block(self) -> Block<B> {
        self.block
    }

    pub fn num_children(&self) -> NonZeroU32 {
        let view = node::View::new(self.block.data().as_ref());
        NonZeroU32::new(view.size().read())
            .expect("DataInnerNode class invariant violated: Has only zero children")
    }

    pub fn children<'a>(&'a self) -> impl Iterator<Item = BlockId> + ExactSizeIterator + 'a {
        let view = node::View::new(self.block.data().as_ref());
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

    fn _children_mut_raw<'a>(
        &'a mut self,
    ) -> impl Iterator<Item = &'a mut [u8]> + ExactSizeIterator + 'a {
        let view = node::View::new(self.block.data_mut().as_mut());
        let children_data: &mut [u8] = view.into_data().into_slice();
        children_data.chunks_exact_mut(BLOCKID_LEN)
    }

    pub fn add_child(&mut self, child: &DataNode<B>) -> Result<()> {
        let depth = self.depth().get();
        let view = node::View::new(self.block.data_mut());
        let prev_num_children = view.size().read();
        ensure!(
            child.depth() == depth - 1,
            "Tried to add a child of depth {} to an inner node of depth {}",
            child.depth(),
            self.depth()
        );
        let new_child_entry: &mut [u8] = self
            ._children_mut_raw()
            .skip(usize::try_from(prev_num_children).unwrap())
            .next()
            .ok_or_else(|| anyhow!("Adding more children than we can store"))?;
        new_child_entry.copy_from_slice(child.block_id().data());
        let mut view = node::View::new(self.block.data_mut());
        view.size_mut().write(prev_num_children + 1);
        Ok(())
    }

    pub fn shrink_num_children(&mut self, new_num_children: NonZeroU32) -> Result<()> {
        let mut view = node::View::new(self.block.data_mut().as_mut());
        let old_num_children = view.size().read();
        ensure!(
            new_num_children.get() <= old_num_children,
            "Called DataInnerNode::shrink_num_children({}) for a node with {} children",
            new_num_children,
            view.size().read()
        );
        let free_begin = usize::try_from(new_num_children.get()).unwrap() * BLOCKID_LEN;
        let free_end = usize::try_from(old_num_children).unwrap() * BLOCKID_LEN;
        view.data_mut()[free_begin..free_end].fill(0);
        view.size_mut().write(new_num_children.get());
        Ok(())
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.block.flush().await
    }

    pub fn upcast(self) -> DataNode<B> {
        DataNode::Inner(self)
    }
}

pub fn serialize_inner_node(depth: u8, children: &[BlockId], layout: &NodeLayout) -> Data {
    let mut data = Data::from(vec![0; layout.block_size_bytes.try_into().unwrap()]);
    initialize_inner_node(depth, children, layout, &mut data);
    data
}

pub fn initialize_inner_node(
    depth: u8,
    children: &[BlockId],
    layout: &NodeLayout,
    dest: &mut Data,
) {
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
        children.len() <= usize::try_from(layout.max_children_per_inner_node()).unwrap(),
        "Inner nodes can only store {} children but tried to store {}",
        layout.max_children_per_inner_node(),
        children.len(),
    );

    let mut view = node::View::new(dest);
    view.format_version_header_mut()
        .write(FORMAT_VERSION_HEADER);
    view.unused_mut().write(0);
    view.depth_mut().write(depth);
    view.size_mut()
        .write(u32::try_from(children.len()).unwrap());
    _serialize_children(view.data_mut(), children);
}

fn _serialize_children(dest: &mut [u8], children: &[BlockId]) {
    assert_eq!(dest.len(), children.len() * BLOCKID_LEN, "Serializing {} children requires {} bytes but tried to serialize into a buffer with {} bytes.", children.len(), children.len() * BLOCKID_LEN, dest.len());
    for (index, child) in children.iter().enumerate() {
        // TODO Some way to avoid this copy by not using &[BlockId] or Vec<BlockId> but our own collection type that already has it aligned correctly?
        dest[(BLOCKID_LEN * index)..(BLOCKID_LEN * (index + 1))].copy_from_slice(child.data());
    }
}
