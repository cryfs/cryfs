use std::collections::HashSet;
use std::fmt::Debug;

use super::FilesystemCheck;
use crate::error::CorruptedError;
use cryfs_blobstore::{BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

/// Check that each existing node is reachable
/// Note: That each reachable node exists is already checked by the way cryfs-recover traverses the filesystem.
pub struct CheckUnreachableNodes {
    errors: Vec<CorruptedError>,
}

impl CheckUnreachableNodes {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }
}

impl FilesystemCheck for CheckUnreachableNodes {
    fn process_reachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        // nothing to do
    }

    fn process_unreachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        self.errors.push(CorruptedError::NodeUnreferenced {
            node_id: *node.block_id(),
        });
    }

    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        // nothing to do
    }

    fn finalize(self) -> Vec<CorruptedError> {
        self.errors
    }
}
