use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::{CheckError, CorruptedError, FilesystemCheck};

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
    fn process_reachable_readable_blob(
        &mut self,
        _blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_reachable_unreadable_blob(&mut self, _blob_id: BlobId) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_reachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_reachable_unreadable_node(&mut self, node_id: BlockId) -> Result<(), CheckError> {
        self.errors.push(CorruptedError::NodeUnreadable { node_id });
        Ok(())
    }

    fn process_unreachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_unreachable_unreadable_node(&mut self, node_id: BlockId) -> Result<(), CheckError> {
        self.errors.push(CorruptedError::NodeUnreadable { node_id });
        Ok(())
    }

    fn finalize(self) -> Vec<CorruptedError> {
        self.errors
    }
}
