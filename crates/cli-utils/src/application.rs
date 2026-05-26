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

    fn new(args: Self::ConcreteArgs, env: Environment) -> Result<Self, CliError>;

    /// The logging configuration to use if the user didn't supply any `--log` flags.
    fn default_log_config(&self) -> clap_logflag::LoggingConfig;

    /// Whether to print the version banner (and optionally check for updates)
    /// before invoking [`Application::main`]. Defaults to `true`; daemon-style
    /// entry points override this to skip the banner so it doesn't leak onto
    /// the user's TTY through the parent's inherited stderr.
    fn should_show_version(&self) -> bool {
        true
    }

    /// If true, [`run`] skips its own `init_logging!` call. The application
    /// takes responsibility for initializing logging itself — typically
    /// because it receives logging config out-of-band (e.g., a daemon child
    /// reading config from its parent over an IPC channel after `main` is
    /// entered). The raw [`LogArgs`] are still passed to `main` so the
    /// application can decide what to do with them.
    fn defer_logging_init(&self) -> bool {
        false
    }

    /// Entry point. `log_args` is the parsed `--log` flag values (possibly
    /// empty). For most apps this can be ignored — [`run`] has already
    /// initialized logging from these args + [`default_log_config`]. Apps
    /// that need to forward the config elsewhere (e.g. to a daemon child
    /// via RPC) can resolve `log_args.or_default(...)` themselves with a
    /// destination-appropriate default.
    fn main(self, log_args: LogArgs) -> Result<(), CliError>;
}

pub fn run<App: Application>() -> ExitCode {
    // TODO Print an error message, probably should be specific to the error. Maybe main should return a Result<(), Self::Error>?
    match _run::<App>() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // TODO Coloring the output would be nice
            // TODO This indentation matches cryfs-cli, but it might not match cryfs-check. We should either add indentation to cryfs-check, or make it conditional here.
            eprintln!("  Error: {}", err);
            err.kind.exit_code()
        }
    }
}

pub fn _run<App: Application>() -> Result<(), CliError> {
    show_backtrace_on_panic::<App>();

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
            let app = App::new(args, env.clone())?;
            if !app.defer_logging_init() {
                clap_logflag::init_logging!(
                    log.or_default(app.default_log_config()),
                    DEFAULT_LOG_LEVEL
                );
            }
            if app.should_show_version() {
                show_version(
                    #[cfg(feature = "check_for_updates")]
                    env,
                );
            }
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
