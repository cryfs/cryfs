#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]
#![allow(rustdoc::private_intra_doc_links)] // TODO Remove this, we probably don't want private links in the documentation.

mod entry;
mod guard;
mod inserting;
mod loading_or_loaded;
mod store;

pub use guard::LoadedEntryGuard;
pub use inserting::Inserting;
pub use loading_or_loaded::LoadingOrLoaded;
pub use store::{ConcurrentStore, RequestImmediateDropResult};

cryfs_version::assert_cargo_version_equals_git_version!();
