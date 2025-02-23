use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::BlockStore;

use super::{
    BlobToProcess, CheckError, FilesystemCheck, NodeAndBlobReferenceFromReachableBlob,
    NodeToProcess, check_result::CheckResult,
};
use crate::{error::BlobUnreadableError, node_info::BlobReference};

/// Check that all blobs are readable
/// Note: This may not check that all nodes of a blob are readable, it only catches blobs that fail to load entirely
pub struct CheckBlobsReadable {
    unreadable_blobs: BTreeMap<BlobId, BTreeSet<BlobReference>>,
}

impl CheckBlobsReadable {
    pub fn new() -> Self {
        Self {
            unreadable_blobs: BTreeMap::new(),
        }
    }
}

impl FilesystemCheck for CheckBlobsReadable {
    fn process_reachable_blob<'a, 'b>(
        &mut self,
        blob: BlobToProcess<'a, 'b, impl BlockStore + Send + Sync + Debug + 'static>,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError> {
        match blob {
            BlobToProcess::Readable(_blob) => {
                // nothing to do
            }
            BlobToProcess::Unreadable(blob_id) => {
                self.unreadable_blobs
                    .entry(blob_id)
                    .or_insert_with(BTreeSet::new)
                    .insert(referenced_as.clone());
            }
        }

        Ok(())
    }

    fn process_reachable_blob_again<'a, 'b>(
        &mut self,
        blob: BlobToProcess<'a, 'b, impl BlockStore + Send + Sync + Debug + 'static>,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError> {
        self.process_reachable_blob(blob, referenced_as)
    }

    fn process_reachable_node<'a>(
        &mut self,
        _node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
        _referenced_as: &NodeAndBlobReferenceFromReachableBlob,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_unreachable_node<'a>(
        &mut self,
        _node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn finalize(self) -> CheckResult {
        let mut errors = CheckResult::new();

        for (blob_id, referenced_as) in self.unreadable_blobs {
            errors.add_error(BlobUnreadableError::new(blob_id, referenced_as));
        }

        errors
    }
}
