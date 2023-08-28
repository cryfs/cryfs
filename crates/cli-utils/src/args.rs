use anyhow::Result;
use clap::{error::ErrorKind, Args, Parser};

use super::env::Environment;
use cryfs_version::VersionInfo;

// TODO Flag for log verbosity, https://crates.io/crates/clap-verbosity-flag

#[derive(Parser, Debug)]
pub struct ImmediateExitFlags {
    #[arg(short = 'V', long, global = true, group = "immediate-exit")]
    pub version: bool,
}

#[derive(Parser, Debug)]
pub struct CombinedArgs<ConcreteArgs: Args> {
    #[command(flatten)]
    pub immediate_exit_flags: ImmediateExitFlags,

    #[command(flatten)]
    pub concrete_args: ConcreteArgs,
}

pub fn parse_args<ConcreteArgs: Args>(
    env: &Environment,
    name: &str,
    version_info: VersionInfo,
) -> Result<Option<ConcreteArgs>> {
    super::version::show_version(env, name, version_info);

    // First try to parse ImmediateExitFlags by themselves. This is necessary because if we start by parsing `CombinedArgs`,
    // it would fail if `ConcreteArgs` aren't present.
    match ImmediateExitFlags::try_parse() {
        Ok(immediate_exit_flags) => {
            if immediate_exit_flags.version {
                // We've already printed the version number above, no need to print it again
                return Ok(None);
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp => {
                // do nothing, clap will print the help when parsing `CombinedArgs` below.
            }
            ErrorKind::UnknownArgument => {
                // this is ok, it just means some arguments from `ConcreteArgs` may have been present.
            }
            ErrorKind::DisplayVersion => {
                panic!("We have our own handling for `--version`, this shouldn't happen");
            }
            _ => {
                // Something went wrong parsing the arguments, e.g `--version=bad` or something like that was passed in.
                // Let's parse the arguments again, but this time so that clap exits with an error.
                CombinedArgs::<ConcreteArgs>::parse();
                panic!("We expected the previous line to exit with an error. CLI Parsing error was: {e:#?}");
            }
        },
    }

    let args = CombinedArgs::<ConcreteArgs>::parse();

    Ok(Some(args.concrete_args))
}
