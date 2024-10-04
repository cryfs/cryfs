use anyhow::Result;
use clap::Args;

use cryfs_version::VersionInfo;

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

    let env = Environment::read_env()?;

    if let Some(args) = parse_args::<App::ConcreteArgs>(&env, App::NAME, App::VERSION)? {
        let app = App::new(args, env)?;
        app.main()?;
    }

    Ok(())
}
