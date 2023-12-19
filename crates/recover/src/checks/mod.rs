use std::fmt::Debug;
use std::sync::Mutex;

use cryfs_blobstore::{BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::BlockStore;
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::error::CorruptedError;

// TODO Check
//  ( some of these should probably be added as checks into general loading code so they run in regular cryfs as well and then cryfs-recover just catches the loading error )
//  - root is a directory
//  - all referenced blocks are present
//  - all present blocks are referenced
//  - all blocks are readable
//  - trees are balanced left-max-data trees
//  - leaves not empty
//  - there are no cycles or self-references within a tree
//  - there are no cycles or self-references between trees (i.e. in the directory structure)
//  - all parent pointers are correct
//  - dir blobs are correct
//     - for each entry: entry type is valid
//     - for each entry: mode is correct (matches entry type)
//     - for each entry: entry type matches blob type
//     - for each entry: entry name is valid utf-8 without forbidden characters
//     - blob size is correct, i.e. contains exactly an integer number of entries. Currently, that's an assertion in DirEntryList::parse but it probably shouldn't be.
//  - utf-8 encoding of symlink targets (tbd: check if we actually allow non-utf8 symlink targets, then we don't need to check this)
//  - unused space in blocks is correctly zeroed out
//  - do we need to check integrity block store in-block-block-id matches the real block id? We might have to check it separately because I think we're using integrity block store with AllowIntegrityViolations.
//    Do we then also need to check for any other integrity violations?

/// The trait that all filesystem checks must implement.
/// The cryfs-recover program will call the methods of this trait
/// - `process_existing_node` for each existing block in the file system, whether referenced or not.
/// - `process_reachable_tree` for each blob that is (transitively) referenced from the root blob, i.e. a reachable entity in the file system.
/// The order of these calls is not guaranteed.
///
/// At the end, it will call `finalize` to get a list of all the errors found.
pub trait FilesystemCheck {
    fn process_existing_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    );
    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    );
    fn finalize(self) -> Vec<CorruptedError>;
}

mod unreferenced_blocks;
use unreferenced_blocks::CheckUnreferencedNodes;

pub struct AllChecks {
    check_unreferenced_nodes: Mutex<CheckUnreferencedNodes>,
    additional_errors: Mutex<Vec<CorruptedError>>,
}

impl AllChecks {
    pub fn new() -> Self {
        Self {
            check_unreferenced_nodes: Mutex::new(CheckUnreferencedNodes::new()),
            additional_errors: Mutex::new(Vec::new()),
        }
    }

    pub fn process_existing_node(
        &self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        self.check_unreferenced_nodes
            .lock()
            .unwrap()
            .process_existing_node(node);
    }

    pub fn process_reachable_blob(
        &self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        self.check_unreferenced_nodes
            .lock()
            .unwrap()
            .process_reachable_blob(blob);
    }

    pub fn finalize(mut self) -> Vec<CorruptedError> {
        let mut errors = self.additional_errors.into_inner().unwrap();
        errors.extend(
            self.check_unreferenced_nodes
                .into_inner()
                .unwrap()
                .finalize(),
        );
        errors
    }

    pub fn add_error(&self, error: CorruptedError) {
        self.additional_errors.lock().unwrap().push(error);
    }
}
