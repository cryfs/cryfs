use std::fmt::Debug;

use crate::error::{BlobInfoAsExpectedByEntryInParent, NodeInfoAsExpectedByEntryInParent};
use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockStore;

use super::{BlobToProcess, CheckError, CorruptedError, FilesystemCheck, NodeToProcess};

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
    fn process_reachable_blob<'a, 'b>(
        &mut self,
        _blob: BlobToProcess<'a, 'b, impl BlockStore + Send + Sync + Debug + 'static>,
        _expected_blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_reachable_node<'a>(
        &mut self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
        expected_node_info: &NodeInfoAsExpectedByEntryInParent,
        _blob_id: BlobId,
        _blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        match node {
            NodeToProcess::Readable(_node) => {
                // do nothing
            }
            NodeToProcess::Unreadable(node_id) => {
                self.errors.push(CorruptedError::NodeUnreadable {
                    node_id: *node_id,
                    expected_node_info: Some(expected_node_info.clone()),
                });
            }
        }
        Ok(())
    }

    fn process_unreachable_node<'a>(
        &mut self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        match node {
            NodeToProcess::Readable(_node) => {
                // do nothing
            }
            NodeToProcess::Unreadable(node_id) => {
                self.errors.push(CorruptedError::NodeUnreadable {
                    node_id: *node_id,
                    expected_node_info: None,
                });
            }
        }
        Ok(())
    }

    fn finalize(self) -> Vec<CorruptedError> {
        self.errors
    }
}
