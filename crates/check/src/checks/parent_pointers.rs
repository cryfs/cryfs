use std::collections::HashSet;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::{
    utils::reference_checker::{MarkAsSeenResult, ReferenceChecker},
    CheckError, CorruptedError, FilesystemCheck,
};

// TODO Tests
// TODO Rename to blob reference checks to contrast with unreferenced_nodes.rs?

#[derive(Clone, Eq, PartialEq, Debug)]
struct BlobInfo {
    parent_pointer: BlobId,
    children: ChildrenOfBlob,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum ChildrenOfBlob {
    File,
    Symlink,
    Dir { children: HashSet<BlobId> },
}

impl ChildrenOfBlob {
    pub fn into_iter(self) -> impl Iterator<Item = BlobId> {
        match self {
            Self::File | Self::Symlink => HashSet::new().into_iter(),
            Self::Dir { children } => children.into_iter(),
        }
    }
}

#[derive(Eq, PartialEq)]
struct ReferencedByParent(BlobId);

/// Check that blob parent pointers go back to the parent that referenced the blob
pub struct CheckParentPointers {
    // Stores the parent pointer of each blob and which parent blobs it is referenced by
    reference_checker: ReferenceChecker<BlobId, BlobInfo, ReferencedByParent>,

    errors: Vec<CorruptedError>,
}

impl CheckParentPointers {
    pub fn new(root_blob_id: BlobId) -> Self {
        let mut reference_checker = ReferenceChecker::new();
        reference_checker.mark_as_referenced(root_blob_id, ReferencedByParent(BlobId::zero()));
        Self {
            reference_checker,
            errors: vec![],
        }
    }
}

impl FilesystemCheck for CheckParentPointers {
    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) -> Result<(), CheckError> {
        let blob_info = BlobInfo {
            parent_pointer: blob.parent(),
            children: match blob {
                FsBlob::File(_) => ChildrenOfBlob::File,
                FsBlob::Symlink(_) => ChildrenOfBlob::Symlink,
                FsBlob::Directory(blob) => ChildrenOfBlob::Dir {
                    children: blob.entries().map(|entry| *entry.blob_id()).collect(),
                },
            },
        };
        let mark_as_seen_result = self
            .reference_checker
            .mark_as_seen(blob.blob_id(), blob_info.clone());
        match mark_as_seen_result {
            MarkAsSeenResult::AlreadySeenBefore { prev_seen_info } => {
                panic!("This shouldn't happen because the runner guarantees that it doesn't process the same node multiple times");
            }
            MarkAsSeenResult::NotSeenBeforeYet => {
                for child_id in blob_info.children.into_iter() {
                    self.reference_checker
                        .mark_as_referenced(child_id, ReferencedByParent(blob.blob_id()));
                }
            }
        }

        Ok(())
    }

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
        let mut errors = self.errors;
        errors.extend(
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
            }));
        errors

        // TODO Should we report any left-over items in `reference_checker`?
        //      Maybe it makes sense to remove some of that responsibility from `unreferenced_nodes` and have that one focus on within-tree references only.
    }
}
