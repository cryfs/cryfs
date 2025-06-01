//! This crate contains end-to-end tests for cryfs, testing the number of blobs and blocks that are read or written by each file system operation.
//! This is meant to ensure that we don't accidentally run more operations than necessary and don't regress performance over time.

mod filesystem_driver;
mod fixture;
pub mod operations;
pub mod rstest;
mod test_driver;

// TODO Write README
// TODO Improve behavior when "benchmarking" feature is on/off, e.g. fix unused code warnings.

cryfs_version::assert_cargo_version_equals_git_version!();
