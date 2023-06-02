use anyhow::Result;
use cryfs_cli::Cli;

fn main() -> Result<()> {
    // TODO Is env_logger the right logging library?
    env_logger::init();

    if let Some(cli) = Cli::new()? {
        cli.main()?;
    }

    Ok(())

    // TODO Better error messages for common errors instead of just printing the error stack trace
    // TODO The C++ version had well-defined exit codes for common error cases. Add that here as well.
}

// TODO Tests (e.g. integration tests)
