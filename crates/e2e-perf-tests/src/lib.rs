#![cfg(test)]

mod fixture;
mod operations;
mod rstest;

cryfs_version::assert_cargo_version_equals_git_version!();
