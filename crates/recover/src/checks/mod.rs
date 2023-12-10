use std::fmt::Debug;
use std::sync::Mutex;

use cryfs_blobstore::{BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::BlockStore;
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::error::CorruptedError;

// TODO Check
//  - root is a directory
//  - all referenced blocks are present
//  - all present blocks are referenced
//  - all blocks are readable
//  - trees are balanced left-max-data trees
//  - all parent pointers are correct
//  - there are no cycles or self-references

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
}

impl AllChecks {
    pub fn new() -> Self {
        Self {
            check_unreferenced_nodes: Mutex::new(CheckUnreferencedNodes::new()),
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

    pub fn finalize(self) -> Vec<CorruptedError> {
        self.check_unreferenced_nodes
            .into_inner()
            .unwrap()
            .finalize()
    }
}
