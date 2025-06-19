use cryfs_utils::async_drop::AsyncDrop;
use derivative::Derivative;
use std::fmt::Debug;
use std::sync::Mutex;

use cryfs_blobstore::{BlobId, BlobStoreOnBlocks, DataNode};
use cryfs_blockstore::{BlockId, BlockStore};
use cryfs_filesystem::filesystem::fsblobstore::FsBlob;

use super::assertion::Assertion;
use super::error::{CheckError, CorruptedError};
use crate::node_info::{BlobReference, NodeAndBlobReferenceFromReachableBlob};

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
pub enum BlobToProcess<'a, B>
where
    B: BlockStore<Block: Send + Sync>
        + AsyncDrop<Error = anyhow::Error>
        + Send
        + Sync
        + Debug
        + 'static,
{
    Readable(&'a FsBlob<BlobStoreOnBlocks<B>>),
    Unreadable(BlobId),
}

#[derive(Debug)]
pub enum NodeToProcess<B>
where
    B: BlockStore<Block: Send + Sync>
        + AsyncDrop<Error = anyhow::Error>
        + Send
        + Sync
        + Debug
        + 'static,
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
    fn process_reachable_blob<'a>(
        &mut self,
        blob: BlobToProcess<
            'a,
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError>;

    /// Like [Self::process_reachable_blob], but this is called whenever a blob is referenced **for the second or later time**,
    /// i.e. there are multiple references to it in the file system.
    fn process_reachable_blob_again<'a>(
        &mut self,
        blob: BlobToProcess<
            'a,
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError>;

    /// Called for each node that is part of a reachable blob
    fn process_reachable_node(
        &mut self,
        node: &NodeToProcess<
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
        expected_node_info: &NodeAndBlobReferenceFromReachableBlob,
    ) -> Result<(), CheckError>;

    /// Called for each node that is not part of a reachable blob
    fn process_unreachable_node<'a>(
        &mut self,
        node: &NodeToProcess<
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
    ) -> Result<(), CheckError>;

    /// Called to get the results and all accumulated errors
    fn finalize(self) -> CheckResult;
}

mod utils;

mod unreferenced_nodes;
use unreferenced_nodes::CheckUnreferencedNodes;

mod parent_pointers;
use parent_pointers::CheckParentPointers;

mod blobs_readable;
use blobs_readable::CheckBlobsReadable;

mod check_result;
use check_result::CheckResult;

pub struct AllChecks {
    check_unreachable_nodes: Mutex<CheckUnreferencedNodes>,
    check_parent_pointers: Mutex<CheckParentPointers>,
    check_blobs_readable: Mutex<CheckBlobsReadable>,
    additional_errors: Mutex<CheckResult>,
}

impl AllChecks {
    pub fn new(root_blob_id: BlobId) -> Self {
        Self {
            check_unreachable_nodes: Mutex::new(CheckUnreferencedNodes::new(root_blob_id)),
            check_parent_pointers: Mutex::new(CheckParentPointers::new(root_blob_id)),
            check_blobs_readable: Mutex::new(CheckBlobsReadable::new()),
            additional_errors: Mutex::new(CheckResult::new()),
        }
    }

    pub fn process_reachable_blob<'a>(
        &self,
        blob: BlobToProcess<
            'a,
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError> {
        // TODO Here and in other methods, avoid having to list all the members and risking to forget one. Maybe a macro?
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_blob(blob, referenced_as)?;
        self.check_parent_pointers
            .lock()
            .unwrap()
            .process_reachable_blob(blob, referenced_as)?;
        self.check_blobs_readable
            .lock()
            .unwrap()
            .process_reachable_blob(blob, referenced_as)?;
        Ok(())
    }

    pub fn process_reachable_blob_again<'a>(
        &self,
        blob: BlobToProcess<
            'a,
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
        referenced_as: &BlobReference,
    ) -> Result<(), CheckError> {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_blob_again(blob, referenced_as)?;
        self.check_parent_pointers
            .lock()
            .unwrap()
            .process_reachable_blob_again(blob, referenced_as)?;
        self.check_blobs_readable
            .lock()
            .unwrap()
            .process_reachable_blob_again(blob, referenced_as)?;
        Ok(())
    }

    pub fn process_reachable_node<'a>(
        &self,
        node: &NodeToProcess<
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
        referenced_as: &NodeAndBlobReferenceFromReachableBlob,
    ) -> Result<(), CheckError> {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_reachable_node(node, referenced_as)?;
        self.check_parent_pointers
            .lock()
            .unwrap()
            .process_reachable_node(node, referenced_as)?;
        self.check_blobs_readable
            .lock()
            .unwrap()
            .process_reachable_node(node, referenced_as)?;
        Ok(())
    }

    pub fn process_unreachable_node<'a>(
        &self,
        node: &NodeToProcess<
            impl BlockStore<Block: Send + Sync>
            + AsyncDrop<Error = anyhow::Error>
            + Send
            + Sync
            + Debug
            + 'static,
        >,
    ) -> Result<(), CheckError> {
        self.check_unreachable_nodes
            .lock()
            .unwrap()
            .process_unreachable_node(node)?;
        self.check_parent_pointers
            .lock()
            .unwrap()
            .process_unreachable_node(node)?;
        self.check_blobs_readable
            .lock()
            .unwrap()
            .process_unreachable_node(node)?;
        Ok(())
    }

    pub fn finalize(self) -> Vec<CorruptedError> {
        let mut reported_errors = self.additional_errors.into_inner().unwrap();

        reported_errors.add_all(
            self.check_unreachable_nodes
                .into_inner()
                .unwrap()
                .finalize(),
        );
        reported_errors.add_all(self.check_parent_pointers.into_inner().unwrap().finalize());
        reported_errors.add_all(self.check_blobs_readable.into_inner().unwrap().finalize());

        reported_errors.finalize()
    }

    pub fn add_error(&self, error: impl Into<CorruptedError>) {
        self.additional_errors.lock().unwrap().add_error(error);
    }

    pub fn add_assertion(&self, assertion: Assertion) {
        self.additional_errors
            .lock()
            .unwrap()
            .add_assertion(assertion);
    }
}
