use std::collections::HashMap;
use std::fmt::Debug;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::{CorruptedError, FilesystemCheck};

// TODO Tests
// TODO Rename to blob reference checks to contrast with unreferenced_nodes.rs?

/// Check that blob parent pointers go back to the parent that referenced the blob
pub struct CheckParentPointers {
    // TODO Use [super:utils::reference_checker]

    // Stores parents for blobs we've seen but haven't seen their parents yet.
    // This is the parent pointer of the blob itself.
    parents_of_seen_and_unreferenced: HashMap<BlobId, BlobId>,

    // Stores parents for blobs we haven't seen yet but have seen their parents.
    // This is the actual blob id of a parent blob referencing the blob.
    parents_of_unseen_and_referenced: HashMap<BlobId, BlobId>,

    // Stores parents for blobs we've seen and we have seen their parents.
    parents_of_seen_and_referenced: HashMap<BlobId, BlobId>,

    errors: Vec<CorruptedError>,
}

impl CheckParentPointers {
    pub fn new() -> Self {
        Self {
            parents_of_seen_and_unreferenced: HashMap::new(),
            parents_of_unseen_and_referenced: HashMap::new(),
            parents_of_seen_and_referenced: HashMap::new(),
            errors: vec![],
        }
    }

    fn mark_as_seen(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        let blob_id = blob.blob_id();
        if let Some(referenced_by_parent) = self.parents_of_unseen_and_referenced.remove(&blob_id) {
            // We've previously seen the parent of this blob and are now seeing the blob itself.
            assert!(!self.parents_of_seen_and_unreferenced.contains_key(&blob_id), "Algorithm invariant violated: A node was in both `seen_and_unreferenced` and in `unseen_and_referenced`.");
            let parent_pointer = blob.parent();
            if self
                .parents_of_seen_and_referenced
                .insert(blob_id, parent_pointer)
                .is_some()
            {
                panic!("Algorithm invariant violated: A blob was in both `seen_and_referenced` and `unseen_and_referenced`: {blob_id:?}");
            }
            if parent_pointer != referenced_by_parent {
                self.errors.push(CorruptedError::WrongParentPointer {
                    blob_id,
                    referenced_by_parent,
                    parent_pointer,
                });
            }
        } else if self.parents_of_seen_and_referenced.contains_key(&blob_id) {
            panic!("Algorithm invariant violated: We've seen {blob_id:?} twice");
        } else {
            // We haven't seen the parent of this blob yet.
            if self
                .parents_of_seen_and_unreferenced
                .insert(blob_id, blob.parent())
                .is_some()
            {
                panic!("Algorithm invariant violated: We've seen {blob_id:?} twice");
            }
        }
    }

    fn mark_as_referenced(&mut self, parent_id: BlobId, child_id: BlobId) {
        if let Some(parent_pointer) = self.parents_of_seen_and_unreferenced.remove(&child_id) {
            // We already saw this blob previously and now we saw its parent.
            assert!(!self.parents_of_unseen_and_referenced.contains_key(&child_id), "Algorithm invariant violated: A blob was in both `unseen_and_referenced` and in `seen_and_unreferenced`.");
            if self
                .parents_of_seen_and_referenced
                .insert(child_id, parent_pointer)
                .is_some()
            {
                panic!("Algorithm invariant violated: A blob was in both `seen_and_unreferenced` and in `seen_and_referenced`.");
            }
        } else if self.parents_of_seen_and_referenced.contains_key(&child_id) {
            // We've already seen this blob and the reference to it. This is now the second reference to it.
            self.errors
                .push(CorruptedError::BlobReferencedMultipleTimes { blob_id: child_id });
        } else {
            // We haven't seen this blob yet. Remember it.
            if self
                .parents_of_unseen_and_referenced
                .insert(child_id, parent_id)
                .is_some()
            {
                // TODO Specifically test scenarios for BlobReferencedMultipleTimes, both this and the case above
                self.errors
                    .push(CorruptedError::BlobReferencedMultipleTimes { blob_id: child_id });
            }
        }
    }
}

impl FilesystemCheck for CheckParentPointers {
    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        self.mark_as_seen(blob);
        match blob {
            FsBlob::File(_) | FsBlob::Symlink(_) => {
                // Files and symlinks don't reference other blobs
            }
            FsBlob::Directory(blob) => {
                for child in blob.entries() {
                    self.mark_as_referenced(blob.blob_id(), *child.blob_id());
                }
            }
        }
    }

    fn process_reachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        // do nothing
    }

    fn process_reachable_unreadable_node(&mut self, _node_id: BlockId) {
        // do nothing
    }

    fn process_unreachable_node(
        &mut self,
        _node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        // do nothing
    }

    fn process_unreachable_unreadable_node(&mut self, _node_id: BlockId) {
        // do nothing
    }

    fn finalize(self) -> Vec<CorruptedError> {
        self.errors

        // TODO Should we report any left-over items in `parents_of_seen_and_unreferenced` or `parents_of_unseen_and_referenced`?
        //      Maybe it makes sense to remove some of that responsibility from `unreferenced_nodes` and have that one focus on within-tree references only.
    }
}
