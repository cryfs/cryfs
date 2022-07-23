use crate::blobstore::on_blocks::data_node_store::{DataNodeStore, DataNode};
use crate::blockstore::low_level::BlockStore;
use crate::utils::async_drop::{AsyncDropGuard, AsyncDropArc};

pub struct DataTree<B: BlockStore + Send + Sync> {
    // The lock on the root node also ensures that there never are two [DataTree] instances for the same tree
    root_node: DataNode<B>,
    node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>,
}

impl <B: BlockStore + Send + Sync> DataTree<B> {
    pub(super) fn new(root_node: DataNode<B>, node_store: AsyncDropGuard<AsyncDropArc<DataNodeStore<B>>>) -> Self {
        Self {root_node, node_store}
    }
}
