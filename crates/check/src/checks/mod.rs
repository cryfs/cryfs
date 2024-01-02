use std::fmt::Debug;
use std::sync::Mutex;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::error::CorruptedError;

// TODO Check
//  ( some of these should probably be added as checks into general loading code so they run in regular cryfs as well and then cryfs-check just catches the loading error )
//  - root is a directory
//  - trees are balanced left-max-data trees
//  - depth of nodes is correct
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
/// The cryfs-check program will call the methods of this trait for blobs/nodes it encounters.
/// The order of these calls is not specified.
/// At the end, it will call `finalize` to get a list of all the errors found.
pub trait FilesystemCheck {
    /// Called for each blob that is reachable from the root of the file system via its directory structure
    fn process_reachable_blob(
        &mut self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    );

    /// Called for each node that is part of a reachable blob
    fn process_reachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    );

    /// Called for each node that is part of a reachable blob but is unreadable
    fn process_reachable_unreadable_node(&mut self, node_id: BlockId);

    /// Called for each node that is not part of a reachable blob
    fn process_unreachable_node(
        &mut self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    );

    /// Called for each node that is not part of a reachable blob and is unreadable
    fn process_unreachable_unreadable_node(&mut self, node_id: BlockId);

    /// Called to get the results and all accumulated errors
    fn finalize(self) -> Vec<CorruptedError>;
}

mod unreferenced_nodes;
use unreferenced_nodes::CheckUnreferencedNodes;

mod nodes_readable;
use nodes_readable::CheckNodesReadable;

pub struct AllChecks {
    check_unreachable_nodes: Mutex<CheckUnreferencedNodes>,
    check_nodes_readable: Mutex<CheckNodesReadable>,
    additional_errors: Mutex<Vec<CorruptedError>>,
}

impl AllChecks {
    pub fn new(root_blob_id: BlobId) -> Self {
        Self {
            check_unreachable_nodes: Mutex::new(CheckUnreferencedNodes::new(root_blob_id)),
            check_nodes_readable: Mutex::new(CheckNodesReadable::new()),
            additional_errors: Mutex::new(Vec::new()),
        }
    }

    pub fn process_reachable_blob(
        &self,
        blob: &FsBlob<BlobStoreOnBlocks<impl BlockStore + Send + Sync + Debug + 'static>>,
    ) {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_blob(blob);
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_reachable_blob(blob);
    }

    pub fn process_reachable_node(
        &self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_node(node);
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_reachable_node(node);
    }

    pub fn process_reachable_unreadable_node(&self, node_id: BlockId) {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_unreadable_node(node_id);
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_reachable_unreadable_node(node_id);
    }

    pub fn process_unreachable_node(
        &self,
        node: &DataNode<impl BlockStore + Send + Sync + Debug + 'static>,
    ) {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_unreachable_node(node);
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_unreachable_node(node);
    }

    pub fn process_unreachable_unreadable_node(&self, node_id: BlockId) {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_unreachable_unreadable_node(node_id);
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_unreachable_unreadable_node(node_id);
    }

    pub fn finalize(self) -> Vec<CorruptedError> {
        self.additional_errors
            .into_inner()
            .unwrap()
            .into_iter()
            .chain(
                self.check_unreachable_nodes
                    .into_inner()
                    .unwrap()
                    .finalize()
                    .into_iter(),
            )
            .chain(
                self.check_nodes_readable
                    .into_inner()
                    .unwrap()
                    .finalize()
                    .into_iter(),
            )
            .collect()
    }

    pub fn add_error(&self, error: CorruptedError) {
        self.additional_errors.lock().unwrap().push(error);
    }
}
