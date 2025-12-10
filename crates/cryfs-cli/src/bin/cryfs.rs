// cryfs-cli only makes sense if either fuser or fuse_mt is enabled

use std::process::ExitCode;

#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
use cryfs_cli::Cli;

#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
fn main() -> ExitCode {
    cryfs_cli_utils::run::<Cli>()

    // TODO Better error messages for common errors instead of just printing the error stack trace
}

#[cfg(not(any(feature = "fuser", feature = "fuse_mt")))]
fn main() -> ExitCode {
    eprintln!(
        "Error: cryfs-cli was compiled without selecting a fuse backend. Please enable either the 'fuser' or 'fuse_mt' feature."
    );
    ExitCode::from(1)
}

// TODO Tests (e.g. integration tests)
