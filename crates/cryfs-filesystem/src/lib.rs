#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]
#![allow(rustdoc::private_intra_doc_links)] // TODO Remove this, we probably don't want private links in the documentation.

// TODO Figure out what the public API of this module should be
pub mod filesystem;

// TODO Throughout the whole codebase, check for short functions that should be `#[inline]`
