#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod args;

mod cli;
pub use cli::{check_filesystem, RecoverCli};

mod checks;
mod console;
mod error;
pub use error::CorruptedError;
mod node_info;
pub use node_info::{
    BlobInfoAsSeenByLookingAtBlob, BlobReference, BlobReferenceWithId, NodeAndBlobReference,
    NodeAndBlobReferenceFromReachableBlob, NodeInfoAsSeenByLookingAtNode, NodeReference,
};
mod assertion;
mod runner;
mod task_queue;

cryfs_version::assert_cargo_version_equals_git_version!();
