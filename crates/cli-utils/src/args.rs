use std::sync::Arc;

use anyhow::{Result, anyhow};
use clap::{
    Args, Parser,
    builder::{Styles, styling::AnsiColor},
    error::ErrorKind,
};
use clap_logflag::LogArgs;

use crate::error::{CliError, CliErrorKind};

pub enum ArgParseError {
    Clap(clap::Error),
    Other(CliError),
}

#[derive(Parser, Debug)]
pub struct ImmediateExitFlags {
    #[arg(short = 'V', long)]
    pub version: bool,
}

#[derive(Parser, Debug)]
#[command(styles=clap_style())]
pub struct CombinedArgs<ConcreteArgs: Args> {
    #[command(flatten)]
    pub immediate_exit_flags: ImmediateExitFlags,

    #[command(flatten)]
    pub concrete_args: ConcreteArgs,

    #[command(flatten)]
    pub log: LogArgs,
}

pub enum ParseArgsResult<ConcreteArgs: Args> {
    ShowVersion,
    Normal { log: LogArgs, args: ConcreteArgs },
}

pub fn parse_args<ConcreteArgs: Args>() -> Result<ParseArgsResult<ConcreteArgs>, ArgParseError> {
    // First try to parse ImmediateExitFlags by themselves. This is necessary because if we start by parsing `CombinedArgs`,
    // it would fail if `ConcreteArgs` aren't present.
    let args = match ImmediateExitFlags::try_parse() {
        Ok(immediate_exit_flags) => {
            if immediate_exit_flags.version {
                return Ok(ParseArgsResult::ShowVersion);
            } else {
                // TODO Can this actually happen? If ImmediateExitFlags parsed, we should have `--version` since that's the only flag right now.
                CombinedArgs::<ConcreteArgs>::try_parse().map_err(ArgParseError::Clap)?
            }
        }
        Err(e) => {
            match e.kind() {
                ErrorKind::DisplayHelp => {
                    // We need to display a help message. The easiest way to do that is to parse the arguments again,
                    // but this time including `ConcreteArgs`. Clap will then exit and display the help message.
                    let Err(err) = CombinedArgs::<ConcreteArgs>::try_parse() else {
                        panic!(
                            "We expected the previous line to exit with a help message. CLI Parsing error was: {e:#?}"
                        );
                    };
                    return Err(ArgParseError::Clap(err));
                }
                ErrorKind::UnknownArgument => {
                    // Looks like some `ConcreteArgs` may have been present. In this case, we don't support the `--version` flag.
                    // So let's parse our flags and make sure that `--version` isn't present.
                    let args =
                        CombinedArgs::<ConcreteArgs>::try_parse().map_err(ArgParseError::Clap)?;
                    // We successfully parsed the arguments, so we can return them. But we don't support the `--version` flag together with other arguments.
                    if args.immediate_exit_flags.version {
                        return Err(ArgParseError::Other(CliError {
                            kind: CliErrorKind::InvalidArguments,
                            error: Arc::new(anyhow!(
                                "the argument '--version' cannot be used with other arguments"
                            )),
                        }));
                    }
                    args
                }
                ErrorKind::DisplayVersion => {
                    panic!("We have our own handling for `--version`, this shouldn't happen");
                }
                _ => {
                    // Something went wrong parsing the arguments, e.g `--version=bad` or something like that was passed in.
                    // Let's parse the arguments again, but this time so that clap exits with an error.
                    // TODO Can this actually happen?
                    let Err(err) = CombinedArgs::<ConcreteArgs>::try_parse() else {
                        panic!(
                            "We expected the previous line to exit with an error. CLI Parsing error was: {e:#?}"
                        );
                    };
                    return Err(ArgParseError::Clap(err));
                }
            }
        }
    };

    Ok(ParseArgsResult::Normal {
        log: args.log,
        args: args.concrete_args,
    })
}

const fn clap_style() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default().bold())
        .usage(AnsiColor::Cyan.on_default().bold())
        .literal(AnsiColor::Cyan.on_default())
        .placeholder(AnsiColor::BrightCyan.on_default())
}
