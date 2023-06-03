use anyhow::Result;
use cryfs_cli::Cli;

fn main() -> Result<()> {
    // TODO Is env_logger the right logging library?
    env_logger::init();

    // TODO Runtime settings
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("cryfs")
        .enable_all()
        .build()
        .unwrap();

    if let Some(cli) = Cli::new()? {
        runtime.block_on(cli.main())?;
    }

    Ok(())

    // TODO Better error messages for common errors instead of just printing the error stack trace
    // TODO The C++ version had well-defined exit codes for common error cases. Add that here as well.
}

// TODO Tests (e.g. integration tests)
