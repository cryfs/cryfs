// Raise the trait-solver recursion limit above rustc's default of 128: proving
// `generic_array::ArrayLength` for our typenum-parameterized ciphers overflows
// it. See crates/crypto/src/lib.rs for the full explanation.
#![recursion_limit = "512"]
#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod args;

mod cli;
pub use cli::{RecoverCli, check_filesystem};

mod checks;
mod console;
mod error;
pub use error::{
    BlobReferencedMultipleTimesError, BlobUnreadableError, CorruptedError, NodeMissingError,
    NodeReferencedMultipleTimesError, NodeUnreadableError, NodeUnreferencedError,
    WrongParentPointerError,
};
mod node_info;
pub use node_info::{
    BlobInfoAsSeenByLookingAtBlob, BlobReference, BlobReferenceWithId,
    MaybeBlobInfoAsSeenByLookingAtBlob, MaybeBlobReferenceWithId,
    MaybeNodeInfoAsSeenByLookingAtNode, NodeAndBlobReference,
    NodeAndBlobReferenceFromReachableBlob, NodeInfoAsSeenByLookingAtNode, NodeReference,
};
mod assertion;
mod runner;
mod task_queue;

cryfs_version::assert_cargo_version_equals_git_version!();
