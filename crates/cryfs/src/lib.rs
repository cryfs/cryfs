#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

// TODO Figure out what the public API of this module should be
pub mod config;
pub mod filesystem;
pub mod localstate;
pub mod utils;
mod version;
pub use config::ALL_CIPHERS;
pub use version::CRYFS_VERSION;

// TODO Throughout the whole codebase, check for short functions that should be `#[inline]`
