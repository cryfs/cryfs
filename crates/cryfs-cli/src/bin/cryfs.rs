use anyhow::Result;
use cryfs_cli::Cli;

fn main() -> Result<()> {
    cryfs_cli_utils::run::<Cli>()

    // TODO Better error messages for common errors instead of just printing the error stack trace
    // TODO The C++ version had well-defined exit codes for common error cases. Add that here as well.
}

// TODO Tests (e.g. integration tests)
