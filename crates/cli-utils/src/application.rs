use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

use cryfs_version::VersionInfo;

use crate::args::parse_args;
use crate::env::Environment;

#[async_trait]
pub trait Application: Sized {
    type ConcreteArgs: Args;

    const NAME: &'static str;
    const VERSION: VersionInfo<'static, 'static, 'static>;

    fn new(args: Self::ConcreteArgs, env: Environment) -> Result<Self>;
    async fn main(&self) -> Result<()>;
}

pub fn run<App: Application>() -> Result<()> {
    // TODO Is env_logger the right logging library?
    env_logger::init();

    let env = Environment::new();

    if let Some(args) = parse_args::<App::ConcreteArgs>(&env, App::NAME, App::VERSION)? {
        // TODO Runtime settings
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name(App::NAME)
            .enable_all()
            .build()
            .unwrap();

        let app = App::new(args, env)?;
        runtime.block_on(app.main())?;
    }

    Ok(())
}
