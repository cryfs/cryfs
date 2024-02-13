use std::fmt::Debug;

use crate::error::{
    BlobInfoAsExpectedByEntryInParent, BlobInfoAsSeenByLookingAtBlob, BlobReference,
};
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::{BlobType, EntryType, FsBlob};
use cryfs_utils::peekable::PeekableExt;

use super::{
    utils::reference_checker::ReferenceChecker, CheckError, CorruptedError, FilesystemCheck,
};

// TODO Rename to blob reference checks to contrast with unreferenced_nodes.rs?

#[derive(Eq, PartialEq)]
enum SeenBlobInfo {
    Readable {
        blob_info: BlobInfoAsSeenByLookingAtBlob,
    },
    Unreadable {
        expected_blob_info: BlobInfoAsExpectedByEntryInParent,
    },
}

/// Check that blob parent pointers go back to the parent that referenced the blob
pub struct CheckParentPointers {
    // Stores the parent pointer of each blob and which parent blobs it is referenced by
    reference_checker: ReferenceChecker<BlobId, SeenBlobInfo, BlobReference>,
}

impl CheckParentPointers {
    pub fn new(root_blob_id: BlobId) -> Self {
        let mut reference_checker = ReferenceChecker::new();
        reference_checker.mark_as_referenced(
            root_blob_id,
            BlobReference {
                expected_child_info: BlobInfoAsExpectedByEntryInParent::root_dir(),
            },
        );
        Self { reference_checker }
    }
}

impl FilesystemCheck for CheckParentPointers {
    fn process_reachable_readable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
        blob_info_as_expected_by_entry_in_parent: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        let seen_blob_info = SeenBlobInfo::Readable {
            blob_info: BlobInfoAsSeenByLookingAtBlob {
                parent_pointer: blob.parent(),
                blob_type: blob.blob_type(),
            },
        };
        self.reference_checker
            .mark_as_seen(blob.blob_id(), seen_blob_info);

        match blob {
            FsBlob::File(_) | FsBlob::Symlink(_) => {
                // Files and Symlinks don't have children
            }
            FsBlob::Directory(blob) => {
                for entry in blob.entries() {
                    self.reference_checker.mark_as_referenced(
                        *entry.blob_id(),
                        BlobReference {
                            expected_child_info: BlobInfoAsExpectedByEntryInParent {
                                blob_type: entry_type_to_blob_type(entry.entry_type()),
                                parent_id: blob.blob_id(),
                                path: blob_info_as_expected_by_entry_in_parent
                                    .path
                                    .join(entry.name()),
                            },
                        },
                    );
                }
            }
        }

        Ok(())
    }

    fn process_reachable_unreadable_blob(
        &mut self,
        blob_id: BlobId,
        expected_blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        self.reference_checker.mark_as_seen(
            blob_id,
            SeenBlobInfo::Unreadable {
                expected_blob_info: expected_blob_info.clone(),
            },
        );
        Ok(())
    }

    fn process_reachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
        _blob_id: BlobId,
        _blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_reachable_unreadable_node(
        &mut self,
        _node_id: BlockId,
        _blob_id: BlobId,
        _blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
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
            .flat_map(|(blob_id, seen_blob_info, referenced_as)| {
                let mut errors = vec![];
                if referenced_as.len() > 1 {
                    errors.push(CorruptedError::BlobReferencedMultipleTimes { blob_id });
                } else if referenced_as.len() == 0 {
                    panic!("Algorithm invariant violated: blob was not referenced by any parent. The way the algorithm works, this should not happen since we only handle reachable blobs.");
                }

                match seen_blob_info {
                    Some(SeenBlobInfo::Readable{blob_info}) => {
                        let mut matching_parents = referenced_as.iter()
                            .filter(|&BlobReference {expected_child_info}| blob_info.parent_pointer == expected_child_info.parent_id)
                            .peekable();
                        if matching_parents.is_empty() {
                            errors.push(CorruptedError::WrongParentPointer {
                                blob_id,
                                blob_info,
                                // TODO How to handle the case where referenced_as Vec has duplicate entries?
                                referenced_as: referenced_as.into_iter().collect(),
                            });
                        }
                        // TODO If matching_parents does not contain one with the right blob type, generate an assertion for blob type mismatch.
                        //      This should be an error caught by another check but we should do an assertion here.
                    }
                    Some(SeenBlobInfo::Unreadable {expected_blob_info}) => {
                        errors.push(CorruptedError::Assert(Box::new(CorruptedError::BlobUnreadable { blob_id, expected_blob_info })));
                    }
                    None => {
                        errors.extend(referenced_as.into_iter().map(|BlobReference{expected_child_info}| {
                            CorruptedError::Assert(Box::new(CorruptedError::BlobMissing { blob_id, expected_blob_info: expected_child_info }))
                        }));
                    }
                }
                errors.into_iter()
            }).collect()

        // TODO Should we report any left-over items in `reference_checker`?
        //      Maybe it makes sense to remove some of that responsibility from `unreferenced_nodes` and have that one focus on within-tree references only.
    }
}

// TODO This exists in multiple places. Deduplicate.
fn entry_type_to_blob_type(entry_type: EntryType) -> BlobType {
    match entry_type {
        EntryType::File => BlobType::File,
        EntryType::Dir => BlobType::Dir,
        EntryType::Symlink => BlobType::Symlink,
    }
}
