use thiserror::Error;

mod node_unreadable;
pub use node_unreadable::NodeUnreadableError;

mod node_unreferenced;
pub use node_unreferenced::NodeUnreferencedError;

mod node_missing;
pub use node_missing::NodeMissingError;

mod node_referenced_multiple_times;
pub use node_referenced_multiple_times::NodeReferencedMultipleTimesError;

mod blob_referenced_multiple_times;
pub use blob_referenced_multiple_times::BlobReferencedMultipleTimesError;

mod blob_unreadable;
pub use blob_unreadable::BlobUnreadableError;

mod wrong_parent_pointer;
pub use wrong_parent_pointer::WrongParentPointerError;

/// A [CorruptedError] is an error we found in the file system when analyzing it
#[derive(Debug, Error, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub enum CorruptedError {
    #[error(transparent)]
    NodeUnreadable(#[from] NodeUnreadableError),

    #[error(transparent)]
    NodeMissing(#[from] NodeMissingError),

    #[error(transparent)]
    NodeUnreferenced(#[from] NodeUnreferencedError),

    #[error(transparent)]
    NodeReferencedMultipleTimes(#[from] NodeReferencedMultipleTimesError),

    // TODO Should we unify NodeReferencedMultipleTimes with BlobReferencedMultipleTimes?
    #[error(transparent)]
    BlobReferencedMultipleTimes(#[from] BlobReferencedMultipleTimesError),

    #[error(transparent)]
    BlobUnreadable(#[from] BlobUnreadableError),

    #[error(transparent)]
    WrongParentPointer(#[from] WrongParentPointerError),
}

/// A CheckError is an error found in the analysis itself. This doesn't necessarily mean that the file system is corrupted
#[derive(Error, Debug)]
pub enum CheckError {
    #[error("The filesystem was modified while the check was running. Please make sure the file system is not mounted or modified for the duration of the check.\n Details: {msg}")]
    FilesystemModified { msg: String },
}
