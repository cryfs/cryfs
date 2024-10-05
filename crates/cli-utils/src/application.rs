use anyhow::Result;
use clap::Args;

use cryfs_version::VersionInfo;

use super::version::show_version;
#[cfg(feature = "check_for_updates")]
use super::version::ReqwestHttpClient;
use crate::args::parse_args;
use crate::env::Environment;

pub trait Application: Sized {
    type ConcreteArgs: Args;

    const NAME: &'static str;
    const VERSION: VersionInfo<'static, 'static, 'static>;

    fn new(args: Self::ConcreteArgs, env: Environment) -> Result<Self>;

    fn main(self) -> Result<()>;
}

pub fn run<App: Application>() -> Result<()> {
    // TODO Is env_logger the right logging library?
    env_logger::init();

    show_backtrace_on_panic::<App>();

    let env = Environment::read_env()?;

    show_version(
        &env,
        App::NAME,
        #[cfg(feature = "check_for_updates")]
        ReqwestHttpClient,
        App::VERSION,
    );

    if let Some(args) = parse_args::<App::ConcreteArgs>(&env, App::NAME, App::VERSION)? {
        let app = App::new(args, env)?;
        app.main()?;
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
