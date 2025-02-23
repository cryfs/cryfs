use console::style;
use std::fmt::{self, Formatter};

use crate::{
    BlobReferenceWithId, MaybeBlobReferenceWithId, MaybeNodeInfoAsSeenByLookingAtNode,
    NodeAndBlobReference,
};
use cryfs_blockstore::BlockId;
use cryfs_filesystem::filesystem::fsblobstore::BlobType;

use super::ErrorTitle;

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct NodeErrorDisplayMessage<'a, RI>
where
    RI: Iterator<Item = &'a NodeAndBlobReference>,
{
    pub error_title: ErrorTitle,
    pub node_info: ErrorDisplayNodeInfo<'a, RI>,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct ErrorDisplayNodeInfo<'a, RI>
where
    RI: Iterator<Item = &'a NodeAndBlobReference>,
{
    pub node_id: BlockId,
    pub node_info: MaybeNodeInfoAsSeenByLookingAtNode,
    pub node_referenced_as: RI,
}

impl<'a, RI> NodeErrorDisplayMessage<'a, RI>
where
    RI: Iterator<Item = &'a NodeAndBlobReference>,
{
    pub fn display(self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{error_title}", error_title = self.error_title)?;
        display_error_display_node_info(f, self.node_info)?;

        Ok(())
    }
}

fn display_error_display_node_info<'a, RI>(
    f: &mut Formatter<'_>,
    obj: ErrorDisplayNodeInfo<'a, RI>,
) -> fmt::Result
where
    RI: Iterator<Item = &'a NodeAndBlobReference>,
{
    let mut has_references = false;
    for referenced_as in obj.node_referenced_as {
        has_references = true;
        write!(f, "  ---> ")?;
        display_node_and_blob_reference(f, referenced_as)?;
        write!(f, "\n")?;
    }
    if !has_references {
        write!(f, "  ---> No references to node found\n")?;
    }
    write!(
        f,
        "  {node_id_title} {node_id}\n  {node_info_title} ",
        node_id_title = style("Node Id:").bold(),
        node_id = &obj.node_id,
        node_info_title = style("Node Info:").bold(),
    )?;
    display_maybe_node_info_as_seen_by_looking_at_node(f, &obj.node_info)?;
    write!(f, "\n")?;

    Ok(())
}

fn display_maybe_node_info_as_seen_by_looking_at_node(
    f: &mut fmt::Formatter<'_>,
    obj: &MaybeNodeInfoAsSeenByLookingAtNode,
) -> fmt::Result {
    match obj {
        MaybeNodeInfoAsSeenByLookingAtNode::Missing => write!(f, "Node is missing"),
        MaybeNodeInfoAsSeenByLookingAtNode::Unreadable => write!(f, "Node is unreadable"),
        MaybeNodeInfoAsSeenByLookingAtNode::LeafNode => write!(f, "Leaf node"),
        MaybeNodeInfoAsSeenByLookingAtNode::InnerNode { depth } => {
            write!(f, "Inner node [depth={depth}]")
        }
    }
}

fn display_node_and_blob_reference(
    f: &mut fmt::Formatter<'_>,
    obj: &NodeAndBlobReference,
) -> fmt::Result {
    match obj {
        NodeAndBlobReference::RootNode { belongs_to_blob } => {
            display_blob_reference_with_id(f, belongs_to_blob)?;
            write!(
                f,
                "\n       {node_header} Root node",
                node_header = style("Node referenced as:").bold()
            )
        }
        NodeAndBlobReference::NonRootInnerNode {
            belongs_to_blob,
            depth,
            parent_id,
        } => {
            display_maybe_blob_reference_with_id(f, belongs_to_blob)?;
            write!(
                f,
                "\n       {node_header} Non-root inner node [depth={depth}, parent_node={parent_id}]",
                node_header = style("Node referenced as:").bold()
            )
        }
        NodeAndBlobReference::NonRootLeafNode {
            belongs_to_blob,
            parent_id,
        } => {
            display_maybe_blob_reference_with_id(f, belongs_to_blob)?;
            write!(
                f,
                "\n       {node_header} Non-root leaf node [parent_node={parent_id}]",
                node_header = style("Node referenced as:").bold()
            )
        }
    }
}

fn display_blob_reference_with_id(
    f: &mut fmt::Formatter<'_>,
    obj: &BlobReferenceWithId,
) -> fmt::Result {
    let blob_type = match obj.referenced_as.blob_type {
        BlobType::File => "file",
        BlobType::Dir => "dir",
        BlobType::Symlink => "symlink",
    };
    write!(
        f,
        "In {blob_type} at {path}\n       {blob_title} id={blob_id}, parent_blob={parent_id}",
        path = obj.referenced_as.path,
        blob_title = style("Blob:").bold(),
        blob_id = obj.blob_id,
        parent_id = obj.referenced_as.parent_id,
    )
}

fn display_maybe_blob_reference_with_id(
    f: &mut fmt::Formatter<'_>,
    obj: &MaybeBlobReferenceWithId,
) -> fmt::Result {
    match obj {
        MaybeBlobReferenceWithId::UnreachableFromFilesystemRoot => write!(f, "In unreachable blob"),
        MaybeBlobReferenceWithId::ReachableFromFilesystemRoot {
            blob_id,
            referenced_as,
        } => {
            let blob_type = match referenced_as.blob_type {
                BlobType::File => "file",
                BlobType::Dir => "dir",
                BlobType::Symlink => "symlink",
            };
            write!(
                f,
                "In {blob_type} at {path}\n       {blob_title} id={blob_id}, parent_blob={parent_id}",
                path = referenced_as.path,
                blob_title = style("Blob:").bold(),
                parent_id = referenced_as.parent_id,
            )
        }
    }
}
