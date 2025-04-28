use itertools::Itertools;
use std::fmt::Debug;

use cryfs_blobstore::BlobId;
use cryfs_blockstore::{BlockStore, LLBlockStore};
use cryfs_filesystem::filesystem::fsblobstore::{BlobType, EntryType, FsBlob};
use cryfs_utils::{async_drop::AsyncDrop, peekable::PeekableExt};

use super::{
    BlobToProcess, CheckError, FilesystemCheck, NodeAndBlobReferenceFromReachableBlob,
    NodeToProcess, check_result::CheckResult, utils::reference_checker::ReferenceChecker,
};
use crate::{
    assertion::Assertion,
    error::{
        BlobReferencedMultipleTimesError, BlobUnreadableError, CorruptedError, NodeMissingError,
        WrongParentPointerError,
    },
    node_info::{
        BlobInfoAsSeenByLookingAtBlob, BlobReference, MaybeBlobInfoAsSeenByLookingAtBlob,
        MaybeBlobReferenceWithId,
    },
};

// TODO Rename to blob reference checks to contrast with unreferenced_nodes.rs?

#[derive(Eq, PartialEq)]
enum SeenBlobInfo {
    Readable {
        blob_type: BlobType,
        parent_pointer: BlobId,
    },
    Unreadable {
        referenced_as: BlobReference,
    },
}

impl SeenBlobInfo {
    pub fn to_blob_info_as_seen_by_looking_at_blob(&self) -> BlobInfoAsSeenByLookingAtBlob {
        match self {
            Self::Unreadable { .. } => BlobInfoAsSeenByLookingAtBlob::Unreadable,
            Self::Readable {
                blob_type,
                parent_pointer,
            } => BlobInfoAsSeenByLookingAtBlob::Readable {
                blob_type: *blob_type,
                parent_pointer: *parent_pointer,
            },
        }
    }
}

/// Check that blob parent pointers go back to the parent that referenced the blob
pub struct CheckParentPointers {
    // Stores the parent pointer of each blob and which parent blobs it is referenced by
    reference_checker: ReferenceChecker<BlobId, SeenBlobInfo, BlobReference>,
}

impl CheckParentPointers {
    pub fn new(root_blob_id: BlobId) -> Self {
        let mut reference_checker = ReferenceChecker::new();
        reference_checker.mark_as_referenced(root_blob_id, BlobReference::root_dir());
        Self { reference_checker }
    }
}

impl FilesystemCheck for CheckParentPointers {
    fn process_reachable_blob<'a, 'b>(
        &mut self,
        blob: BlobToProcess<
            'a,
            'b,
            impl BlockStore<Block: Send + Sync> + AsyncDrop + Send + Sync + Debug + 'static,
        >,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError> {
        match blob {
            BlobToProcess::Readable(blob) => {
                let seen_blob_info = SeenBlobInfo::Readable {
                    parent_pointer: blob.parent(),
                    blob_type: blob.blob_type(),
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
                                    blob_type: entry_type_to_blob_type(entry.entry_type()),
                                    parent_id: blob.blob_id(),
                                    path: referenced_as.path.join(entry.name()),
                                },
                            );
                        }
                    }
                }
            }
            BlobToProcess::Unreadable(blob_id) => {
                self.reference_checker.mark_as_seen(
                    blob_id,
                    SeenBlobInfo::Unreadable {
                        referenced_as: referenced_as.clone(),
                    },
                );
            }
        }

        Ok(())
    }

    fn process_reachable_blob_again<'a, 'b>(
        &mut self,
        blob: BlobToProcess<
            'a,
            'b,
            impl BlockStore<Block: Send + Sync> + AsyncDrop + Send + Sync + Debug + 'static,
        >,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError> {
        // TODO What should we do here?
        Ok(())
    }

    fn process_reachable_node<'a>(
        &mut self,
        _node: &NodeToProcess<
            impl BlockStore<Block: Send + Sync> + AsyncDrop + Send + Sync + Debug + 'static,
        >,
        _referenced_as: &NodeAndBlobReferenceFromReachableBlob,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn process_unreachable_node<'a>(
        &mut self,
        _node: &NodeToProcess<
            impl BlockStore<Block: Send + Sync> + AsyncDrop + Send + Sync + Debug + 'static,
        >,
    ) -> Result<(), CheckError> {
        // do nothing
        Ok(())
    }

    fn finalize(self) -> CheckResult {
        let mut errors = CheckResult::new();

        for (blob_id, seen_blob_info, referenced_as) in self.reference_checker.finalize() {
            if referenced_as.len() > 1 {
                let blob_info = seen_blob_info
                    .as_ref()
                    .map(|blob_info| blob_info.to_blob_info_as_seen_by_looking_at_blob().into())
                    .unwrap_or(MaybeBlobInfoAsSeenByLookingAtBlob::Missing);
                errors.add_error(BlobReferencedMultipleTimesError::new(
                    blob_id,
                    blob_info,
                    // TODO How to handle the case where referenced_as Vec has duplicate entries?
                    referenced_as.iter().cloned().collect(),
                ));
            } else if referenced_as.len() == 0 {
                panic!(
                    "Algorithm invariant violated: blob was not referenced by any parent. The way the algorithm works, this should not happen since we only handle reachable blobs."
                );
            }

            match seen_blob_info {
                Some(SeenBlobInfo::Readable {
                    parent_pointer,
                    blob_type,
                }) => {
                    let mut matching_parents = referenced_as
                        .iter()
                        .filter(|blob_reference| parent_pointer == blob_reference.parent_id)
                        .peekable();
                    if matching_parents.is_empty() {
                        errors.add_error(WrongParentPointerError {
                            blob_id,
                            parent_pointer,
                            blob_type,
                            // TODO How to handle the case where referenced_as Vec has duplicate entries?
                            referenced_as: referenced_as.into_iter().collect(),
                        });
                    }
                    // TODO If matching_parents does not contain one with the right blob type, generate an assertion for blob type mismatch.
                    //      This should be an error caught by another check but we should do an assertion here.
                }
                Some(SeenBlobInfo::Unreadable { referenced_as }) => {
                    errors.add_assertion(Assertion::error_matching_predicate_was_reported(
                        move |error| match error {
                            CorruptedError::BlobUnreadable(BlobUnreadableError {
                                blob_id: reported_blob_id,
                                referenced_as: reported_referenced_as,
                            }) => {
                                *reported_blob_id == blob_id
                                    && reported_referenced_as.contains(&referenced_as)
                            }
                            _ => false,
                        },
                    ));
                }
                None => {
                    // Assert that there is a NodeMissing reported that contains all referenced_as we know about here, but it can contain other referenced_as as well, e.g. from an inner node within another blob
                    errors.add_assertion(Assertion::error_matching_predicate_was_reported(
                        move |error| match error {
                            CorruptedError::NodeMissing(NodeMissingError {
                                node_id: reported_node_id,
                                referenced_as: reported_referenced_as,
                            }) => {
                                *reported_node_id == *blob_id.to_root_block_id()
                                    && referenced_as.iter().all(|referenced_as| {
                                        reported_referenced_as
                                            .into_iter()
                                            .filter_map(|reported_reference| {
                                                match reported_reference.clone().blob_info() {
                                                    MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot => None,
                                                    MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {  referenced_as, .. } => Some(referenced_as),
                                                }
                                            })
                                            .contains(referenced_as)
                                    })
                            }
                            _ => false,
                        },
                    ));
                }
            }
        }

        // TODO Should we report any left-over items in `reference_checker`?
        //      Maybe it makes sense to remove some of that responsibility from `unreferenced_nodes` and have that one focus on within-tree references only.

        errors
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
