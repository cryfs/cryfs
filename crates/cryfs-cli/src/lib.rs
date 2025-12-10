#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

// cryfs-cli only makes sense if either fuser or fuse_mt is enabled
#![cfg(any(feature = "fuser", feature = "fuse_mt"))]

mod args;

mod cli;
pub use cli::Cli;

mod console;
mod sanity_checks;

cryfs_version::assert_cargo_version_equals_git_version!();

// TODO Add tests to make sure cryfs-cli correctly mounts, both in foreground and background, and correctly exits either variant when unmounted.

// TODO Throughout the codebase, remove the `as` keyword and replace with `from`/`try_from`
// TODO Throughout the codebase, check where it makes sense to replace usize/u64 with byte_unit::Byte

// TODO Unhelpful error message when things go wrong (e.g. basedir exists but doesn't have a cryfs.config). Improve them.
// TODO When running in foreground, ctrl+c will not display anything if the directory is open in another terminal tab. It will only unmount once that directory is closed. This is good but we should display a message explaining that so that a user doesn't get confused and thinks nothing is happening.
