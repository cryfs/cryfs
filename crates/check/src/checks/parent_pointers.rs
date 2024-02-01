use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::{
    utils::reference_checker::ReferenceChecker, CheckError, CorruptedError, FilesystemCheck,
};

// TODO Tests
// TODO Rename to blob reference checks to contrast with unreferenced_nodes.rs?

#[derive(Eq, PartialEq)]
struct BlobInfo {
    parent_pointer: BlobId,
}

#[derive(Eq, PartialEq)]
struct ReferencedByParent(BlobId);

/// Check that blob parent pointers go back to the parent that referenced the blob
pub struct CheckParentPointers {
    // Stores the parent pointer of each blob and which parent blobs it is referenced by
    reference_checker: ReferenceChecker<BlobId, BlobInfo, ReferencedByParent>,
}

impl CheckParentPointers {
    pub fn new(root_blob_id: BlobId) -> Self {
        let mut reference_checker = ReferenceChecker::new();
        reference_checker.mark_as_referenced(root_blob_id, ReferencedByParent(BlobId::zero()));
        Self { reference_checker }
    }
}

impl FilesystemCheck for CheckParentPointers {
    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) -> Result<(), CheckError> {
        let blob_info = BlobInfo {
            parent_pointer: blob.parent(),
        };
        self.reference_checker
            .mark_as_seen(blob.blob_id(), blob_info);

        match blob {
            FsBlob::File(_) | FsBlob::Symlink(_) => {
                // Files and Symlinks don't have children
            }
            FsBlob::Directory(blob) => {
                for entry in blob.entries() {
                    self.reference_checker
                        .mark_as_referenced(*entry.blob_id(), ReferencedByParent(blob.blob_id()));
                }
            }
        }

        Ok(())
    }

    // TODO fn process_reachable_blob_again() for when a blob is referenced multiple times, because one of those should report a ParentPointer error
    //      also make sure that the blob_referenced_multiple_times tests correctly report these.

    fn process_reachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_reachable_unreadable_node(&mut self, _node_id: BlockId) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_unreachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_unreachable_unreadable_node(&mut self, _node_id: BlockId) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn finalize(self) -> Vec<CorruptedError> {
        self.reference_checker
            .finalize()
            .flat_map(|(blob_id, blob_info, referenced_by)| {
                let mut errors = vec![];
                if referenced_by.len() > 1 {
                    errors.push(CorruptedError::BlobReferencedMultipleTimes { blob_id });
                } else if referenced_by.len() == 0 {
                    panic!("Algorithm invariant violated: blob was not referenced by any parent. The way the algorithm works, this should not happen since we only handle reachable blobs.");
                }

                if let Some(BlobInfo{parent_pointer, ..}) = blob_info {
                    if !referenced_by.contains(&ReferencedByParent(parent_pointer)) {
                        errors.push(CorruptedError::WrongParentPointer {
                            blob_id,
                            parent_pointer,
                            referenced_by: referenced_by
                                .iter()
                                .map(|ReferencedByParent(blob_id)| *blob_id)
                                .collect(),
                        });
                    }
                } else {
                    // TODO Should we be responsible for reporting BlobMissing and `unreferenced_nodes` instead throw an Assert?
                    // errors.push(CorruptedError::Assert(Box::new(CorruptedError::BlobMissing { blob_id })));
                }
                errors.into_iter()
            }).collect()

        // TODO Should we report any left-over items in `reference_checker`?
        //      Maybe it makes sense to remove some of that responsibility from `unreferenced_nodes` and have that one focus on within-tree references only.
    }
}
