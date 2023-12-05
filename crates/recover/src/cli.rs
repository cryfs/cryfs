use anyhow::Result;
use async_trait::async_trait;
use cryfs_blockstore::{
    AllowIntegrityViolations, IntegrityConfig, MissingBlockIsIntegrityViolation,
};
use cryfs_cli_utils::{
    password_provider::InteractivePasswordProvider, print_config, setup_blockstore, Application,
    Environment,
};
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult},
    localstate::LocalStateDir,
    CRYFS_VERSION,
};
use cryfs_version::VersionInfo;

use super::{console::RecoverConsole, runner::RecoverRunner};
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

    async fn main(self) -> Result<()> {
        let config = self.load_config()?;
        print_config(&config);

        log::info!(
            "Calculating stats for filesystem at {}",
            self.args
                .basedir
                .to_str()
                .expect("Invalid utf-8 in filesystem path")
        );
        setup_blockstore(
            self.args.basedir,
            &config,
            &self.local_state_dir,
            // TODO Setup IntegrityConfig correctly
            IntegrityConfig {
                allow_integrity_violations: AllowIntegrityViolations::AllowViolations,
                missing_block_is_integrity_violation:
                    // TODO Since we say AllowViolations above, should this be IsAViolation so we log it?
                    MissingBlockIsIntegrityViolation::IsNotAViolation,
                on_integrity_violation: Box::new(|err| {
                    // TODO What to do here? Maybe we should at least log it
                }),
            },
            RecoverRunner { config: &config },
        )
        .await??;

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
