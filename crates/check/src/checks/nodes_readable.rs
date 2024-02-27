use std::fmt::Debug;

use cryfs_blockstore::BlockStore;

use super::{BlobToProcess, CheckError, CorruptedError, FilesystemCheck, NodeToProcess};
use crate::node_info::{BlobReference, NodeAndBlobReferenceFromReachableBlob};

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
        _referenced_as: &BlobReference,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_reachable_node<'a>(
        &mut self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
        expected_node_info: &NodeAndBlobReferenceFromReachableBlob,
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
