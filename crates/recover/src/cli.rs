use anyhow::Result;
use async_trait::async_trait;
use cryfs_cli_utils::{Application, Environment};
use cryfs_cryfs::CRYFS_VERSION;
use cryfs_version::VersionInfo;

use crate::args::CryfsRecoverArgs;

pub struct RecoverCli {
    args: CryfsRecoverArgs,
}

#[async_trait]
impl Application for RecoverCli {
    type ConcreteArgs = CryfsRecoverArgs;
    const NAME: &'static str = "cryfs-recover";
    const VERSION: VersionInfo<'static, 'static, 'static> = CRYFS_VERSION;

    fn new(args: CryfsRecoverArgs, env: Environment) -> Result<Self> {
        Ok(Self { args })
    }

    async fn main(&self) -> Result<()> {
        Ok(())
    }
}
