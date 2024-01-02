use std::fmt::Debug;

use cryfs_blobstore::{BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::{CorruptedError, FilesystemCheck};

/// Check that each node is readable
pub struct CheckNodesReadable {
    errors: Vec<CorruptedError>,
}

impl CheckNodesReadable {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }
}

impl FilesystemCheck for CheckNodesReadable {
    fn process_reachable_blob(
        &mut self,
        _blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        // do nothing
    }

    fn process_reachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        // do nothing
    }

    fn process_reachable_unreadable_node(&mut self, node_id: BlockId) {
        self.errors.push(CorruptedError::NodeUnreadable { node_id });
    }

    fn process_unreachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        // do nothing
    }

    fn process_unreachable_unreadable_node(&mut self, node_id: BlockId) {
        self.errors.push(CorruptedError::NodeUnreadable { node_id });
    }

    fn finalize(self) -> Vec<CorruptedError> {
        self.errors
    }
}
