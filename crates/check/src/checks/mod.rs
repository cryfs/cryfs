use derivative::Derivative;
use std::fmt::Debug;
use std::sync::Mutex;

use crate::error::{BlobInfoAsExpectedByEntryInParent, NodeReferenceFromReachableBlob};
use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;

use super::error::{CheckError, CorruptedError};

// TODO Check
//  ( some of these should probably be added as checks into general loading code so they run in regular cryfs as well and then cryfs-check just catches the loading error )
//  - root is a directory
//  - trees are balanced left-max-data trees
//  - depth of nodes is correct
//  - leaves not empty, all leaves but rightmost one must be full, rightmost one non-empty
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

#[derive(Debug, Derivative)]
#[derivative(Clone(bound = ""), Copy(bound = ""))]
pub enum BlobToProcess<'a, 'b, B>
where
    B: BlockStore + Send + Sync + Debug + 'static,
{
    Readable(&'a FsBlob<'b, BlobStoreOnBlocks<B>>),
    Unreadable(BlobId),
}

#[derive(Debug)]
pub enum NodeToProcess<B>
where
    B: BlockStore + Send + Sync + Debug + 'static,
{
    Readable(DataNode<B>),
    Unreadable(BlockId),
}

/// The trait that all filesystem checks must implement.
/// The cryfs-check program will call the methods of this trait for blobs/nodes it encounters.
/// The order of these calls is not specified but it guarantees that it calls [Self::process_reachable_blob]
/// only once per blob, and exactly one of the `_node` functions exactly once for each node.
/// At the end, it will call `finalize` to get a list of all the errors found.
pub trait FilesystemCheck {
    /// Called for each blob that is reachable from the root of the file system via its directory structure.
    fn process_reachable_blob<'a, 'b>(
        &mut self,
        blob: BlobToProcess<'a, 'b, impl BlockStore + Send + Sync + Debug + 'static>,
        expected_blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError>;

    /// Called for each node that is part of a reachable blob
    fn process_reachable_node(
        &mut self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
        expected_node_info: &NodeReferenceFromReachableBlob,
    ) -> Result<(), CheckError>;

    /// Called for each node that is not part of a reachable blob
    fn process_unreachable_node<'a>(
        &mut self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError>;

    /// Called to get the results and all accumulated errors
    fn finalize(self) -> Vec<CorruptedError>;
}

mod utils;

mod unreferenced_nodes;
use unreferenced_nodes::CheckUnreferencedNodes;

mod nodes_readable;
use nodes_readable::CheckNodesReadable;

mod parent_pointers;
use parent_pointers::CheckParentPointers;

pub struct AllChecks {
    check_unreachable_nodes: Mutex<CheckUnreferencedNodes>,
    check_nodes_readable: Mutex<CheckNodesReadable>,
    check_parent_pointers: Mutex<CheckParentPointers>,
    additional_errors: Mutex<Vec<CorruptedError>>,
}

impl AllChecks {
    pub fn new(root_blob_id: BlobId) -> Self {
        Self {
            check_unreachable_nodes: Mutex::new(CheckUnreferencedNodes::new(root_blob_id)),
            check_nodes_readable: Mutex::new(CheckNodesReadable::new()),
            check_parent_pointers: Mutex::new(CheckParentPointers::new(root_blob_id)),
            additional_errors: Mutex::new(Vec::new()),
        }
    }

    pub fn process_reachable_blob<'a, 'b>(
        &self,
        blob: BlobToProcess<'a, 'b, impl BlockStore + Send + Sync + Debug + 'static>,
        expected_blob_info: &BlobInfoAsExpectedByEntryInParent,
    ) -> Result<(), CheckError> {
        // TODO Here and in other methods, avoid having to list all the members and risking to forget one. Maybe a macro?
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_blob(blob, expected_blob_info)?;
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_reachable_blob(blob, expected_blob_info)?;
        self.check_parent_pointers
            .lock()
            .unwrap()
            .process_reachable_blob(blob, expected_blob_info)?;
        Ok(())
    }

    pub fn process_reachable_node<'a>(
        &self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
        expected_node_info: &NodeReferenceFromReachableBlob,
    ) -> Result<(), CheckError> {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_node(node, expected_node_info)?;
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_reachable_node(node, expected_node_info)?;
        self.check_parent_pointers
            .lock()
            .unwrap()
            .process_reachable_node(node, expected_node_info)?;
        Ok(())
    }

    pub fn process_unreachable_node<'a>(
        &self,
        node: &NodeToProcess<impl BlockStore + Send + Sync + Debug + 'static>,
    ) -> Result<(), CheckError> {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_unreachable_node(node)?;
        self.check_nodes_readable
            .lock()
            .unwrap()
            .process_unreachable_node(node)?;
        self.check_parent_pointers
            .lock()
            .unwrap()
            .process_unreachable_node(node)?;
        Ok(())
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
            .chain(
                self.check_parent_pointers
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
