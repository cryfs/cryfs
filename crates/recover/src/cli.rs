use anyhow::Result;
use async_trait::async_trait;
use cryfs_blockstore::{
    AllowIntegrityViolations, BlockStore, IntegrityConfig, MissingBlockIsIntegrityViolation,
    OnDiskBlockStore, OptimizedBlockStoreWriter, ReadOnlyBlockStore,
};
use cryfs_cli_utils::{
    password_provider::InteractivePasswordProvider, print_config, setup_blockstore_stack,
    Application, Environment,
};
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult},
    localstate::LocalStateDir,
    CRYFS_VERSION,
};
use cryfs_utils::async_drop::AsyncDropGuard;
use cryfs_version::VersionInfo;
use std::path::Path;

use super::{console::RecoverConsole, error::CorruptedError, runner::RecoverRunner};
use crate::args::CryfsRecoverArgs;

pub struct RecoverCli {
    args: CryfsRecoverArgs,
    local_state_dir: LocalStateDir,
}

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
        println!(
            "Checking filesystem at {}",
            self.args
                .basedir
                .to_str()
                .expect("Invalid utf-8 in filesystem path")
        );

        let config_file_path = self.args.basedir.join("cryfs.config");
        let blockstore = OnDiskBlockStore::new(self.args.basedir);

        let errors = check_filesystem(blockstore, &config_file_path, &self.local_state_dir).await?;

        for error in &errors {
            println!("- {error}");
        }
        println!("Found {} errors", errors.len());

        Ok(())
    }
}

pub async fn check_filesystem(
    blockstore: AsyncDropGuard<impl BlockStore + OptimizedBlockStoreWriter + Sync + Send>,
    config_file_path: &Path,
    local_state_dir: &LocalStateDir,
) -> Result<Vec<CorruptedError>> {
    let blockstore = ReadOnlyBlockStore::new(blockstore);

    let config = load_config(config_file_path, local_state_dir)?;
    print_config(&config);

    // TODO It currently seems to spend some seconds before getting from here in to `RecoveryRunner`. Probably to load local state or something like that. Let's add a spinner.
    //      Or parallelize that with scrypt.

    setup_blockstore_stack(
        blockstore,
        &config,
        local_state_dir,
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
    .await?
}

fn load_config(
    config_file_path: &Path,
    local_state_dir: &LocalStateDir,
) -> Result<ConfigLoadResult, ConfigLoadError> {
    // TODO Allow changing config file using args as C++ did
    cryfs_cryfs::config::load_readonly(
        config_file_path.to_owned(),
        // TODO Allow NonInteractivePasswordProvider like cryfs-cli does?
        &InteractivePasswordProvider,
        &RecoverConsole,
        &CommandLineFlags {
            missing_block_is_integrity_violation: Some(false),
            expected_cipher: None,
        },
        local_state_dir,
    )
}
