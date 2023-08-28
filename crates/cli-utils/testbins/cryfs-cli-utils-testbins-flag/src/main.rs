use cryfs_cli_utils::{
    reexports_for_tests::{
        anyhow::Result,
        async_trait::async_trait,
        clap::{self, Args},
        cryfs_version::{Version, VersionInfo},
    },
    run, Application, Environment,
};

#[derive(Args, Debug)]
struct MyArgs {
    /// Flag Documentation
    #[arg(short = 'f', long = "flag")]
    flag: bool,
}

struct Cli {
    args: MyArgs,
}

#[async_trait]
impl Application for Cli {
    type ConcreteArgs = MyArgs;
    const NAME: &'static str = "my-cli-name";
    const VERSION: VersionInfo<'static, 'static, 'static> = VersionInfo::new(
        Version {
            major: 1,
            minor: 2,
            patch: 3,
            prerelease: None,
        },
        None,
    );

    fn new(args: MyArgs, env: Environment) -> Result<Self> {
        Ok(Self { args })
    }

    async fn main(&self) -> Result<()> {
        println!("cryfs-cli-utils-testbins-flag:main:{:?}", self.args.flag,);
        Ok(())
    }
}

fn main() -> Result<()> {
    run::<Cli>()
}
