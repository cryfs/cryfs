use std::process::ExitCode;

use cryfs_cli::Cli;

fn main() -> ExitCode {
    cryfs_cli_utils::run::<Cli>()

    // TODO Better error messages for common errors instead of just printing the error stack trace
}

// TODO Tests (e.g. integration tests)
