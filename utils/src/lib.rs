#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

pub mod async_drop;
pub mod at_exit;
pub mod binary;
pub mod containers;
pub mod crypto;
pub mod data;
mod panic;
pub mod path;
pub mod periodic_task;
pub mod stream;

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

cryfs_version::assert_cargo_version_equals_git_version!();
