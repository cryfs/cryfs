use std::process::ExitCode;

use anyhow::Result;
use clap::Args;

use cryfs_version::VersionInfo;
use log::LevelFilter;

use super::version::show_version;
#[cfg(feature = "check_for_updates")]
use super::version::ReqwestHttpClient;
use crate::args::{parse_args, ParseArgsResult};
use crate::env::Environment;
use crate::error::CliError;

const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

pub trait Application: Sized {
    type ConcreteArgs: Args;

    const NAME: &'static str;
    const VERSION: VersionInfo<'static, 'static, 'static>;

    fn new(args: Self::ConcreteArgs, env: Environment) -> Result<Self, CliError>;

    /// The logging configuration to use if the user didn't supply any `--log` flags.
    fn default_log_config(&self) -> clap_logflag::LoggingConfig;

    fn main(self) -> Result<(), CliError>;
}

pub fn run<App: Application>() -> ExitCode {
    // TODO Print an error message, probably should be specific to the error. Maybe main should return a Result<(), Self::Error>?
    match _run::<App>() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // TODO Coloring the output would be nice
            eprintln!("Error: {}", err);
            err.kind.exit_code()
        }
    }
}

pub fn _run<App: Application>() -> Result<(), CliError> {
    show_backtrace_on_panic::<App>();

    let env = Environment::read_env()?;

    let show_version = |env| {
        show_version(
            &env,
            App::NAME,
            #[cfg(feature = "check_for_updates")]
            ReqwestHttpClient,
            App::VERSION,
        )
    };

    match parse_args::<App::ConcreteArgs>() {
        Ok(ParseArgsResult::ShowVersion) => {
            // TODO We probably should initialize logging here before showing the version,
            // so that any http requests we do for checking for updates have a working logging backend.
            show_version(env);
        }
        Ok(ParseArgsResult::Normal { log, args }) => {
            let app = App::new(args, env.clone())?;
            clap_logflag::init_logging!(
                log.or_default(app.default_log_config()),
                DEFAULT_LOG_LEVEL
            );
            show_version(env);
            app.main()?;
        }
        Err(err) => {
            show_version(env);
            if let Some(error) = err.error.downcast_ref::<clap::Error>() {
                // clap error types can display colored output if exiting this way, otherwise they wouldn't
                // TODO Is there a better way to handle this? We're ignoring the CliErrorKind here which is weird. Should we maybe return an enum Error type that can be either CliError or clap::Error?
                error.exit();
            } else {
                return Err(err);
            }
        }
    }

    Ok(())
}

fn show_backtrace_on_panic<App: Application>() {
    match ::std::env::var("RUST_BACKTRACE") {
        Ok(_) => {
            // The `RUST_BACKTRACE` environment variable is set, change nothing and just use the default behavior of that variable.
        }
        Err(_) => {
            // The `RUST_BACKTRACE` environment variable is not set, define our own default behavior
            if cfg!(debug_assertions) {
                // In debug builds, always show a backtrace on panic, irrespective of the `RUST_BACKTRACE` environment variable
                std::panic::set_hook(Box::new(|panic_info| {
                    let backtrace = std::backtrace::Backtrace::force_capture();
                    eprintln!("{panic_info}");
                    eprintln!("\nBacktrace:\n{backtrace}");
                }));
            } else {
                // In release builds, show a human readable error message and generate a dump file for the user to upload with the issue report
                human_panic::setup_panic!(
                    human_panic::Metadata::new(App::NAME, App::VERSION.to_string())
                        .authors(env!("CARGO_PKG_AUTHORS").replace(":", ", "))
                        .homepage(env!("CARGO_PKG_HOMEPAGE"))
                        .support("Open a ticket at https://github.com/cryfs/cryfs/issues and include the report file.")
                );
                // TODO https://github.com/rust-cli/human-panic/issues/155
            }
        }
    }
}
