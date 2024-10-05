use anyhow::Result;
use std::path::Path;

use cryfs_blockstore::{
    AllowIntegrityViolations, BlockStore, IntegrityConfig, MissingBlockIsIntegrityViolation,
    OnDiskBlockStore, OptimizedBlockStoreWriter, ReadOnlyBlockStore,
};
use cryfs_cli_utils::{
    password_provider::InteractivePasswordProvider, print_config, setup_blockstore_stack,
    Application, Environment,
};
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult, PasswordProvider},
    localstate::LocalStateDir,
    CRYFS_VERSION,
};
use cryfs_utils::{
    async_drop::AsyncDropGuard,
    progress::{ConsoleProgressBarManager, ProgressBarManager},
};
use cryfs_version::VersionInfo;

use super::{console::RecoverConsole, error::CorruptedError, runner::RecoverRunner};
use crate::args::CryfsRecoverArgs;

// TODO Make sure we don't write to local state or integrity data either. Read-only blockstore isn't enough.

pub struct RecoverCli {
    args: CryfsRecoverArgs,
    local_state_dir: LocalStateDir,
}

impl Application for RecoverCli {
    type ConcreteArgs = CryfsRecoverArgs;
    const NAME: &'static str = "cryfs-check";
    const VERSION: VersionInfo<'static, 'static, 'static> = CRYFS_VERSION;

    fn new(args: CryfsRecoverArgs, env: Environment) -> Result<Self> {
        // TODO Make sure we have tests for the local_state_dir location
        let local_state_dir = LocalStateDir::new(env.local_state_dir);
        Ok(Self {
            args,
            local_state_dir,
        })
    }

    fn main(self) -> Result<()> {
        // TODO Runtime settings
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name(Self::NAME)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(self.async_main())
    }
}

impl RecoverCli {
    async fn async_main(self) -> Result<()> {
        println!(
            "Checking filesystem at {}",
            self.args
                .basedir
                .to_str()
                .expect("Invalid utf-8 in filesystem path")
        );

        let config_file_path = self.args.basedir.join("cryfs.config");
        let blockstore = OnDiskBlockStore::new(self.args.basedir);

        // TODO Allow NonInteractivePasswordProvider like cryfs-cli does?
        let password_provider = InteractivePasswordProvider;

        let errors = check_filesystem(
            blockstore,
            &config_file_path,
            &self.local_state_dir,
            &password_provider,
            ConsoleProgressBarManager,
        )
        .await?;

        for error in &errors {
            println!("{error}\n");
        }
        println!("Found {} errors", errors.len());

        Ok(())
    }
}

pub async fn check_filesystem(
    blockstore: AsyncDropGuard<impl BlockStore + OptimizedBlockStoreWriter + Sync + Send>,
    config_file_path: &Path,
    local_state_dir: &LocalStateDir,
    password_provider: &impl PasswordProvider,
    progress_bar_manager: impl ProgressBarManager,
) -> Result<Vec<CorruptedError>> {
    let blockstore = ReadOnlyBlockStore::new(blockstore);

    let config = load_config(
        config_file_path,
        local_state_dir,
        password_provider,
        progress_bar_manager,
    )?;
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
        RecoverRunner {
            config: &config,
            progress_bar_manager,
        },
    )
    .await?
}

fn load_config(
    config_file_path: &Path,
    local_state_dir: &LocalStateDir,
    password_provider: &impl PasswordProvider,
    progress_bars: impl ProgressBarManager,
) -> Result<ConfigLoadResult, ConfigLoadError> {
    // TODO Allow changing config file using args as C++ did
    cryfs_cryfs::config::load_readonly(
        config_file_path.to_owned(),
        password_provider,
        &RecoverConsole,
        &CommandLineFlags {
            missing_block_is_integrity_violation: Some(false),
            expected_cipher: None,
        },
        local_state_dir,
        progress_bars,
    )
}