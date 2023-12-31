use anyhow::Result;
use std::path::Path;

use super::console::InteractiveConsole;
use super::runner::FilesystemRunner;
use crate::args::CryfsArgs;
use cryfs_blockstore::{
    AllowIntegrityViolations, IntegrityConfig, MissingBlockIsIntegrityViolation, OnDiskBlockStore,
};
use cryfs_cli_utils::password_provider::{
    InteractivePasswordProvider, NoninteractivePasswordProvider,
};
use cryfs_cli_utils::{print_config, setup_blockstore_stack, Application, Environment};
use cryfs_cryfs::CRYFS_VERSION;
use cryfs_cryfs::{
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult, Console, PasswordProvider},
    localstate::LocalStateDir,
};
use cryfs_version::VersionInfo;

// TODO Check (and add tests for) error messages make sense, e.g. when
//   - wrong password
//   - basedir/mountdir don't exist
//   - ...

pub struct Cli {
    args: CryfsArgs,
    is_noninteractive: bool,
    local_state_dir: LocalStateDir,
}

impl Application for Cli {
    type ConcreteArgs = CryfsArgs;
    const NAME: &'static str = "cryfs";
    const VERSION: VersionInfo<'static, 'static, 'static> = CRYFS_VERSION;

    // Returns None if the program should exit immediately with a success error code
    fn new(args: CryfsArgs, env: Environment) -> Result<Self> {
        let is_noninteractive = env.is_noninteractive();

        // TODO Make sure we have tests for the local_state_dir location
        let local_state_dir = cryfs_cryfs::localstate::LocalStateDir::new(env.local_state_dir()?);

        Ok(Self {
            is_noninteractive,
            args,
            local_state_dir,
        })
    }

    async fn main(self) -> Result<()> {
        if self.args.show_ciphers {
            for cipher in cryfs_cryfs::config::ALL_CIPHERS {
                println!("{}", cipher);
            }
            return Ok(());
        }

        self.run_filesystem().await?;

        println!(
            "Basedir: {:?}\nMountdir: {:?}",
            self.args.mount.as_ref().unwrap().basedir,
            self.args.mount.as_ref().unwrap().mountdir,
        );
        Ok(())
    }
}

impl Cli {
    async fn run_filesystem(&self) -> Result<()> {
        // TODO C++ code has lots more logic here, migrate that.
        let basedir = self.basedir().to_owned();
        if !basedir.exists() {
            std::fs::create_dir(&basedir)?;
        }
        let mountdir = self.mountdir();
        if !mountdir.exists() {
            std::fs::create_dir(mountdir)?;
        }

        let config = self.load_or_create_config()?;
        print_config(&config);

        setup_blockstore_stack(
            OnDiskBlockStore::new(basedir),
            &config,
            &self.local_state_dir,
            // TODO Setup IntegrityConfig correctly
            IntegrityConfig {
                allow_integrity_violations: AllowIntegrityViolations::DontAllowViolations,
                missing_block_is_integrity_violation:
                    MissingBlockIsIntegrityViolation::IsNotAViolation,
                on_integrity_violation: Box::new(|err| {
                    // TODO
                }),
            },
            FilesystemRunner {
                mountdir,
                config: &config,
            },
        )
        .await??;

        Ok(())
    }

    // TODO Test the console flows for opening an existing/creating a new file system
    fn load_or_create_config(&self) -> Result<ConfigLoadResult, ConfigLoadError> {
        // TODO Allow changing config file using args as C++ did
        let config_file_location = self.basedir().join("cryfs.config");
        cryfs_cryfs::config::load_or_create(
            config_file_location.to_owned(),
            self.password_provider(),
            self.console(),
            // TODO Set CommandLineFlags correctly
            &CommandLineFlags {
                missing_block_is_integrity_violation: None,
                expected_cipher: None,
            },
            &self.local_state_dir,
        )
    }

    fn basedir(&self) -> &Path {
        &self.args.mount.as_ref().expect("Basedir not set").basedir
    }

    fn mountdir(&self) -> &Path {
        &self.args.mount.as_ref().expect("Basedir not set").mountdir
    }

    fn password_provider(&self) -> &'static dyn PasswordProvider {
        if self.is_noninteractive {
            // TODO Make sure we have tests for noninteractive mode
            &NoninteractivePasswordProvider
        } else {
            &InteractivePasswordProvider
        }
    }

    fn console(&self) -> &'static dyn Console {
        // TODO Implement NoninteractiveConsole
        &InteractiveConsole
    }
}

// TODO Tests
