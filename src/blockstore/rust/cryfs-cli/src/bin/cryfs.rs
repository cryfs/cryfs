use anyhow::Result;
use clap::Parser;
use cryfs_cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::new();
    cli.main()
    // TODO Better error messages for common errors instead of just printing the error stack trace
    // TODO The C++ version had well-defined exit codes for common error cases. Add that here as well.
}
