use anyhow::Result;
use async_trait::async_trait;
use cryfs_cli_utils::password_provider::InteractivePasswordProvider;
use cryfs_cli_utils::{print_config, Application, Environment};
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult},
    localstate::LocalStateDir,
    CRYFS_VERSION,
};
use cryfs_version::VersionInfo;

use super::console::RecoverConsole;
use crate::args::CryfsRecoverArgs;

pub struct RecoverCli {
    args: CryfsRecoverArgs,
    local_state_dir: LocalStateDir,
}

#[async_trait]
impl Application for RecoverCli {
    type ConcreteArgs = CryfsRecoverArgs;
    const NAME: &'static str = "cryfs-recover";
    const VERSION: VersionInfo<'static, 'static, 'static> = CRYFS_VERSION;

    fn new(args: CryfsRecoverArgs, env: Environment) -> Result<Self> {
        // TODO Make sure we have tests for the local_state_dir location
        let local_state_dir = LocalStateDir::new(env.local_state_dir()?);
        Ok(Self {
            args,
            local_state_dir,
        })
    }

    async fn main(&self) -> Result<()> {
        let config = self.load_config()?;
        print_config(&config);

        Ok(())
    }
}

impl RecoverCli {
    fn load_config(&self) -> Result<ConfigLoadResult, ConfigLoadError> {
        // TODO Allow changing config file using args as C++ did
        let config_file_location = self.args.basedir.join("cryfs.config");
        cryfs_cryfs::config::load_readonly(
            config_file_location.to_owned(),
            // TODO Allow NonInteractivePasswordProvider like cryfs-cli does?
            &InteractivePasswordProvider,
            &RecoverConsole,
            &CommandLineFlags {
                missing_block_is_integrity_violation: Some(false),
                expected_cipher: None,
            },
            &self.local_state_dir,
        )
    }
}
