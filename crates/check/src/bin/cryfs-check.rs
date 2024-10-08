use std::process::ExitCode;

use cryfs_check::RecoverCli;

fn main() -> ExitCode {
    cryfs_cli_utils::run::<RecoverCli>()
}

// TODO Tests (e.g. integration tests)
