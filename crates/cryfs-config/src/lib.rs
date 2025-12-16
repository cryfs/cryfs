#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

// TODO Figure out what the public API of this module should be

pub mod config;
pub mod localstate;
mod version;

pub use config::ALL_CIPHERS;
pub use version::CRYFS_VERSION;

cryfs_version::assert_cargo_version_equals_git_version!();
