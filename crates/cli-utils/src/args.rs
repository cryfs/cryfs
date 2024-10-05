use anyhow::Result;
use clap::{error::ErrorKind, Args, Parser};

use super::env::Environment;
use super::version::show_version;
#[cfg(feature = "check_for_updates")]
use super::version::ReqwestHttpClient;
use cryfs_version::VersionInfo;

// TODO Flag for log verbosity, https://crates.io/crates/clap-verbosity-flag

#[derive(Parser, Debug)]
pub struct ImmediateExitFlags {
    #[arg(short = 'V', long)]
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
    // TODO Is this the right place to call `show_version` from or should it be at a higher level?
    show_version(
        env,
        name,
        #[cfg(feature = "check_for_updates")]
        ReqwestHttpClient,
        version_info,
    );

    // First try to parse ImmediateExitFlags by themselves. This is necessary because if we start by parsing `CombinedArgs`,
    // it would fail if `ConcreteArgs` aren't present.
    let args = match ImmediateExitFlags::try_parse() {
        Ok(immediate_exit_flags) => {
            if immediate_exit_flags.version {
                // We've already printed the version number above, no need to print it again
                return Ok(None);
            } else {
                CombinedArgs::<ConcreteArgs>::parse()
            }
        }
        Err(e) => {
            match e.kind() {
                ErrorKind::DisplayHelp => {
                    // We need to display a help message. The easiest way to do that is to parse the arguments again,
                    // but this time including `ConcreteArgs`. Clap will then exit and display the help message.
                    CombinedArgs::<ConcreteArgs>::parse();
                    panic!("We expected the previous line to exit with a help message. CLI Parsing error was: {e:#?}");
                }
                ErrorKind::UnknownArgument => {
                    // Looks like some `ConcreteArgs` may have been present. In this case, we don't support the `--version` flag.
                    // So let's parse our flags and make sure that `--version` isn't present.
                    let args = CombinedArgs::<ConcreteArgs>::parse();
                    if args.immediate_exit_flags.version {
                        eprintln!(
                            "error: the argument '--version' cannot be used with other arguments"
                        );
                        std::process::exit(1);
                    }
                    args
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
            }
        }
    };

    Ok(Some(args.concrete_args))
}
