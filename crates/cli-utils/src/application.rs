use std::process::ExitCode;

use anyhow::Result;
use clap::Args;
use clap_logflag::LogArgs;

use cryfs_version::VersionInfo;
use log::LevelFilter;

#[cfg(feature = "check_for_updates")]
use super::version::ReqwestHttpClient;
use super::version::show_version;
use crate::args::{ArgParseError, ParseArgsResult, parse_args};
use crate::env::Environment;
use crate::error::CliError;

/// Default log level used by `init_logging` when neither the user's `--log`
/// flag nor the application's [`Application::default_log_config`] specifies
/// one. Re-exported from this crate so downstream daemon entry points that do
/// their own deferred `init_logging` use the same level as the default path.
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

pub trait Application: Sized {
    type ConcreteArgs: Args;

    const NAME: &'static str;
    const VERSION: VersionInfo<'static, 'static, &'static str>;

    /// The logging configuration to use if the user didn't supply any `--log` flags.
    fn default_log_config(&self) -> clap_logflag::LoggingConfig;

    /// Entry point. `log_args` is the parsed `--log` flag values (possibly
    /// empty). For most apps this can be ignored — [`run`] has already
    /// initialized logging from these args + [`Self::default_log_config`]. Apps
    /// that need to forward the config elsewhere (e.g. to a daemon child
    /// via RPC) can resolve `log_args.or_default(...)` themselves with a
    /// destination-appropriate default.
    fn main(self, log_args: LogArgs) -> Result<(), CliError>;
}

/// The subset of [`Application`]s that can be constructed from parsed args
/// and environment alone. [`run`] requires it; an application whose
/// constructor needs additional inputs (e.g. a capability token that only
/// its outer framework can mint) implements just [`Application`] and is
/// started via [`run_with`] with the construction injected.
pub trait ConstructibleApplication: Application {
    fn new(args: Self::ConcreteArgs, env: Environment) -> Result<Self, CliError>;
}

pub fn run<App: ConstructibleApplication>() -> ExitCode {
    run_with(App::new)
}

/// Like [`run`], but with the application's construction injected instead of
/// taken from [`ConstructibleApplication::new`]. The constructor runs at the
/// same point of the startup pipeline `new` would: after arg parsing and
/// environment loading, before logging init and the version banner.
pub fn run_with<App: Application>(
    construct: impl FnOnce(App::ConcreteArgs, Environment) -> Result<App, CliError>,
) -> ExitCode {
    // TODO Print an error message, probably should be specific to the error. Maybe main should return a Result<(), Self::Error>?
    match _run(construct) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // TODO Coloring the output would be nice
            // TODO This indentation matches cryfs-cli, but it might not match cryfs-check. We should either add indentation to cryfs-check, or make it conditional here.
            eprintln!("  Error: {}", err);
            err.kind.exit_code()
        }
    }
}

pub fn _run<App: Application>(
    construct: impl FnOnce(App::ConcreteArgs, Environment) -> Result<App, CliError>,
) -> Result<(), CliError> {
    setup_panic_handling(App::NAME, &App::VERSION.to_string());

    let env = Environment::read_env()?;

    let show_version = |#[cfg(feature = "check_for_updates")] env| {
        show_version(
            #[cfg(feature = "check_for_updates")]
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
            // Same for the cases below that don't initialize logging yet.
            show_version(
                #[cfg(feature = "check_for_updates")]
                env,
            );
            Ok(())
        }
        Ok(ParseArgsResult::Normal { log, args }) => {
            let app = construct(args, env.clone())?;
            clap_logflag::init_logging!(
                log.or_default(app.default_log_config()),
                DEFAULT_LOG_LEVEL
            );
            show_version(
                #[cfg(feature = "check_for_updates")]
                env,
            );
            app.main(log)
        }
        Err(ArgParseError::Clap(err)) => {
            show_version(
                #[cfg(feature = "check_for_updates")]
                env,
            );
            // clap error types can display colored output if exiting this way, otherwise they wouldn't
            err.exit();
        }
        Err(ArgParseError::Other(err)) => {
            show_version(
                #[cfg(feature = "check_for_updates")]
                env,
            );
            Err(err)
        }
    }
}

/// Install the panic hooks [`run`]/[`run_with`] give every application: a
/// forced backtrace in debug builds, a human-readable message + report file
/// (human-panic) in release builds; a user-set `RUST_BACKTRACE` wins over
/// both. Public so process entry points that bypass the [`run`] pipeline —
/// e.g. a re-exec'd daemon child dispatching straight into its daemon loop —
/// can install the same hooks first thing.
pub fn setup_panic_handling(name: &str, version: &str) {
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
                    human_panic::Metadata::new(name.to_string(), version.to_string())
                        .authors(env!("CARGO_PKG_AUTHORS").replace(":", ", "))
                        .homepage(env!("CARGO_PKG_HOMEPAGE"))
                        .support("Open a ticket at https://github.com/cryfs/cryfs/issues and include the report file.")
                );
                // TODO https://github.com/rust-cli/human-panic/issues/155
            }
        }
    }
}
