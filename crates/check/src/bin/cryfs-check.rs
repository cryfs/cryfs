use anyhow::Result;
use cryfs_check::RecoverCli;

fn main() -> Result<()> {
    cryfs_cli_utils::run::<RecoverCli>()
}

// TODO Tests (e.g. integration tests)
