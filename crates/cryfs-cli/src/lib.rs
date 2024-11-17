#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod args;

mod cli;
pub use cli::Cli;

mod console;
mod sanity_checks;

cryfs_version::assert_cargo_version_equals_git_version!();

// TODO Add tests to make sure cryfs-cli correctly mounts, both in foreground and background, and correctly exits either variant when unmounted.

// TODO Throughout the codebase, remove the `as` keyword and replace with `from`/`try_from`
// TODO Throughout the codebase, check where it makes sense to replace usize/u64 with byte_unit::Byte
