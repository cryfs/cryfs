use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap_logflag::{LogDestination, LogDestinationConfig, LoggingConfig};
use cryfs_filesystem::config::CryConfigFile;
use cryfs_filesystem::localstate::{BasedirMetadata, CheckFilesystemIdError};
use cryfs_runner::{CreateOrLoad, Mounter};
use log::LevelFilter;

use super::console::InteractiveConsole;
use crate::args::{AtimeOption, CryfsArgs, FuseOption, MountArgs};
use cryfs_blockstore::AllowIntegrityViolations;
use cryfs_cli_utils::password_provider::{
    InteractivePasswordProvider, NoninteractivePasswordProvider,
};
use cryfs_cli_utils::{
    Application, CliError, CliErrorKind, CliResultExt, CliResultExtFn, Environment, print_config,
};
use cryfs_filesystem::CRYFS_VERSION;
use cryfs_filesystem::{
    config::{
        CommandLineFlags, ConfigCreateError, ConfigLoadError, ConfigLoadResult, Console,
        CreateConfigFileError, LoadConfigFileError, PasswordProvider, SaveConfigFileError,
    },
    localstate::LocalStateDir,
};
use cryfs_utils::progress::{ConsoleProgressBarManager, ProgressBarManager};
use cryfs_version::VersionInfo;

// TODO Check (and add tests for) error messages make sense, e.g. when
//   - wrong password
//   - basedir/mountdir don't exist
//   - ...

// TODO Cryfs currently panics in fuse when mountdir is not empty or already mounted. We should either check that beforehand, or even better, display fuse errors without a panic.

// TODO Leftover TODOs from C++ code. Do they apply to our rust implementation?
//    - Delete a large file in parallel possible? Takes a long time right now...
//    - Improve parallelity.
//    - Replace panics with other error handling when it is not a programming error but an environment influence (e.g. a block is missing)
//    - Can we improve performance by setting compiler parameter -maes for scrypt?

pub struct Cli {
    args: CryfsArgs,
    is_noninteractive: bool,
    local_state_dir: LocalStateDir,
}

impl Application for Cli {
    type ConcreteArgs = CryfsArgs;
    const NAME: &'static str = "cryfs";
    const VERSION: VersionInfo<'static, 'static, &'static str> = CRYFS_VERSION;

    // Returns None if the program should exit immediately with a success error code
    fn new(args: CryfsArgs, env: Environment) -> Result<Self, CliError> {
        let is_noninteractive = env.is_noninteractive;
        // TODO Make sure we have tests for the local_state_dir location
        let local_state_dir = cryfs_filesystem::localstate::LocalStateDir::new(env.local_state_dir);

        Ok(Self {
            is_noninteractive,
            args,
            local_state_dir,
        })
    }

    fn default_log_config(&self) -> LoggingConfig {
        let in_foreground = self
            .args
            .mount
            .as_ref()
            .map(|args| args.foreground)
            .unwrap_or(
                // No mount args, so we're running a short running command that stays in foreground
                true,
            );
        if in_foreground {
            // Mounting in foreground, let's log to stderr
            LoggingConfig::new(vec![LogDestinationConfig {
                destination: LogDestination::Stderr,
                level: Some(LevelFilter::Warn),
            }])
        } else {
            // Mounting in background, let's log to syslog
            LoggingConfig::new(vec![LogDestinationConfig {
                destination: LogDestination::Syslog,
                level: Some(LevelFilter::Warn),
            }])
        }
    }

    fn main(self) -> Result<(), CliError> {
        // TODO Once we support Windows, we need to check that we're running on a supported windows version. C++ CryFS only supported Windows 7 or later.

        if self.args.show_ciphers {
            self.show_ciphers();
            return Ok(());
        }

        let mounter = if self.mount_args().foreground {
            Mounter::run_in_foreground().map_cli_error(CliErrorKind::UnspecifiedError)?
        } else {
            // We need to daemonize **before** initializing tokio because tokio doesn't support fork, see https://github.com/tokio-rs/tokio/issues/4301
            Mounter::run_in_background().map_cli_error(CliErrorKind::UnspecifiedError)?
        };

        // Note: tokio-console requires running with `RUSTFLAGS="--cfg tokio_unstable" cargo build`, see https://github.com/tokio-rs/console
        #[cfg(feature = "tokio_console")]
        console_subscriber::init();

        // Initialize the tokio runtime in the parent process.
        // If we're mounting in background, then the child process creates its own separate runtime.
        // If we're mounting in foreground, the runtime created here will be used to mount the file system.
        let runtime = cryfs_runner::init_tokio();
        runtime.block_on(self.async_main(mounter))
    }
}

impl Cli {
    async fn async_main(self, mounter: Mounter) -> Result<(), CliError> {
        // TODO Making cryfs-cli init code async could speed it up, e.g. do update checks while creating basedirs or loading the config.
        self.sanity_checks().await?;
        self.run_filesystem(mounter, ConsoleProgressBarManager)
            .await?;

        Ok(())
    }

    async fn sanity_checks(&self) -> Result<(), CliError> {
        let mount_args = self.mount_args();
        super::sanity_checks::check_mountdir_doesnt_contain_basedir(mount_args)
            .map_cli_error(CliErrorKind::BaseDirInsideMountDir)?;
        super::sanity_checks::check_dir_accessible(
            &mount_args.basedir,
            "vault",
            mount_args.create_missing_basedir,
            |path| self.console().ask_create_basedir(path),
        )
        .await
        .map_cli_error(CliErrorKind::InaccessibleBaseDir)?;
        // TODO C++ had special handling of Windows drive letters here. We should probably re-add that
        super::sanity_checks::check_dir_accessible(
            &mount_args.mountdir,
            "mountpoint",
            mount_args.create_missing_mountpoint,
            |path| self.console().ask_create_mountdir(path),
        )
        .await
        .map_cli_error(CliErrorKind::InaccessibleMountDir)?;
        Ok(())
    }

    async fn run_filesystem(
        &self,
        mut mounter: Mounter,
        progress_bars: impl ProgressBarManager,
    ) -> Result<(), CliError> {
        let mount_args = self.mount_args();

        let config =
            self.load_or_create_config(mount_args.allow_replaced_filesystem, progress_bars)?;
        print_config(&config);

        let on_successfully_mounted = || {
            // TODO Output formatting, e.g. colorization
            println!(
                "  CryFS has been successfully mounted to {}",
                mount_args.mountdir.display()
            );
            if mount_args.foreground {
                println!(
                    // TODO Add necessary escape sequences to the mountdir path, e.g. " -> \"
                    "  You can unmount it by pressing Ctrl+C or by running `cryfs-unmount \"{}\"`.",
                    mount_args.mountdir.display(),
                );
            } else {
                println!(
                    // TODO Add necessary escape sequences to the mountdir path, e.g. " -> \"
                    "  You can unmount it by running `cryfs-unmount \"{}\"`.",
                    mount_args.mountdir.display(),
                );
            }
            println!("  To see more information, run `cryfs --help`.");
        };

        let (atime_options, fuse_permission_options) =
            FuseOption::partition(&mount_args.fuse_option);

        let atime_behavior = AtimeOption::to_atime_behavior(&atime_options)
            .map_cli_error(CliErrorKind::InvalidArguments)?;

        mounter
            .mount_filesystem(
                cryfs_runner::MountArgs {
                    basedir: mount_args.basedir.clone(),
                    mountdir: mount_args.mountdir.clone(),
                    allow_integrity_violations: if mount_args.allow_integrity_violations {
                        AllowIntegrityViolations::AllowViolations
                    } else {
                        AllowIntegrityViolations::DontAllowViolations
                    },
                    create_or_load: if config.first_time_access {
                        CreateOrLoad::CreateNewFilesystem
                    } else {
                        CreateOrLoad::LoadExistingFilesystem
                    },
                    config: config.config.into_config(),
                    my_client_id: config.my_client_id,
                    local_state_dir: self.local_state_dir.clone(),
                    unmount_idle: mount_args.unmount_idle.map(Into::into),
                    fuse_options: fuse_permission_options.iter().map(Into::into).collect(),
                    atime_behavior,
                },
                on_successfully_mounted,
            )
            .await?;

        if mount_args.foreground {
            // In foreground mode, we only return after unmount
            // TODO Output formatting, e.g. colorization (and search the codebase for other println statements that might be missing it)
            println!("  CryFS has been unmounted.");
        }

        Ok(())
    }

    fn show_ciphers(&self) {
        for cipher in cryfs_filesystem::config::ALL_CIPHERS {
            println!("{}", cipher);
        }
    }

    // TODO Test the console flows for opening an existing/creating a new file system
    fn load_or_create_config(
        &self,
        allow_replaced_filesystem: bool,
        progress_bars: impl ProgressBarManager,
    ) -> Result<ConfigLoadResult, CliError> {
        let mount_args = self.mount_args();
        let config_file_location = self.config_file_location();
        let config = cryfs_filesystem::config::load_or_create(
            config_file_location.to_owned(),
            self.password_provider(),
            &self.console(),
            &CommandLineFlags {
                missing_block_is_integrity_violation: mount_args
                    .missing_block_is_integrity_violation,
                expected_cipher: mount_args.cipher.clone(),
                blocksize: mount_args.blocksize,
            },
            &self.local_state_dir,
            mount_args.allow_filesystem_upgrade,
            mount_args.allow_replaced_filesystem,
            progress_bars,
        )
        .map_cli_error(|error| match error {
            ConfigLoadError::TooOldFilesystemFormat { .. }
            | ConfigLoadError::TooOldFilesystemFormatDeclinedMigration { .. } => {
                CliErrorKind::TooOldFilesystemFormat
            }

            ConfigLoadError::TooNewFilesystemFormat { .. } => CliErrorKind::TooNewFilesystemFormat,

            ConfigLoadError::InvalidConfig(_)
            | ConfigLoadError::LoadFileError(LoadConfigFileError::ConfigFileNotFound { .. })
            | ConfigLoadError::LoadFileError(LoadConfigFileError::PermissionDenied { .. })
            | ConfigLoadError::LoadFileError(LoadConfigFileError::IoError(_)) => {
                CliErrorKind::InvalidFilesystem
            }

            ConfigLoadError::LoadFileError(LoadConfigFileError::DeserializationError(_)) => {
                CliErrorKind::WrongPasswordOrCorruptedConfigFile
            }

            ConfigLoadError::WrongCipher { .. } => CliErrorKind::WrongCipher,

            ConfigLoadError::WrongBlocksize { .. } => CliErrorKind::WrongBlocksize,

            ConfigLoadError::FilesystemDoesNotTreatMissingBlocksAsIntegrityViolations
            | ConfigLoadError::FilesystemTreatsMissingBlocksAsIntegrityViolations => {
                CliErrorKind::FilesystemHasDifferentIntegritySetup
            }

            ConfigLoadError::FilesystemInSingleClientMode => CliErrorKind::SingleClientFileSystem,

            ConfigLoadError::LocalStateError(_) => CliErrorKind::InaccessibleLocalStateDir,

            ConfigLoadError::SaveFileError(
                SaveConfigFileError::DirectoryComponentDoesntExist { .. },
            )
            | ConfigLoadError::SaveFileError(SaveConfigFileError::PermissionDenied { .. })
            | ConfigLoadError::SaveFileError(SaveConfigFileError::IoError(_))
            | ConfigLoadError::SaveFileError(SaveConfigFileError::SerializationError(_))
            | ConfigLoadError::SaveFileError(SaveConfigFileError::ScryptError(_))
            | ConfigLoadError::ConfigCreateError(ConfigCreateError::CipherNotSupported {
                ..
            })
            | ConfigLoadError::ConfigCreateError(ConfigCreateError::LocalStateError(_))
            | ConfigLoadError::ConfigCreateError(ConfigCreateError::InteractionError(_))
            | ConfigLoadError::CreateFileError(CreateConfigFileError::AlreadyExists { .. })
            | ConfigLoadError::CreateFileError(
                CreateConfigFileError::DirectoryComponentDoesntExist { .. },
            )
            | ConfigLoadError::CreateFileError(CreateConfigFileError::PermissionDenied {
                ..
            })
            | ConfigLoadError::CreateFileError(CreateConfigFileError::IoError(_))
            | ConfigLoadError::CreateFileError(CreateConfigFileError::SerializationError(_))
            | ConfigLoadError::CreateFileError(CreateConfigFileError::ScryptError(_))
            | ConfigLoadError::InteractionError(_) => CliErrorKind::UnspecifiedError,
        })?;
        self.check_config_integrity(&config.config, allow_replaced_filesystem)?;
        Ok(config)
    }

    fn config_file_location(&self) -> PathBuf {
        let mount_args = self.mount_args();
        mount_args
            .config
            .clone()
            .unwrap_or_else(|| mount_args.basedir.join("cryfs.config"))
    }

    fn check_config_integrity(
        &self,
        config: &CryConfigFile,
        allow_replaced_filesystem: bool,
    ) -> Result<(), CliError> {
        let mount_args = self.mount_args();
        let mut basedir_metadata = BasedirMetadata::load(&self.local_state_dir)
            .context("Failed to load local state")
            .map_cli_error(CliErrorKind::UnspecifiedError)?;
        let check_result = basedir_metadata.filesystem_id_for_basedir_is_correct(
            &mount_args.basedir,
            &config.config().filesystem_id,
        );
        if let Err(check_result) = check_result {
            let CheckFilesystemIdError::FilesystemIdIncorrect {
                basedir,
                expected_id,
                actual_id,
            } = &check_result;
            log::warn!(
                "Filesystem id for basedir {} has changed: expected {:?}, got {:?}",
                basedir.display(),
                expected_id,
                actual_id,
            );
            if !allow_replaced_filesystem {
                if !self
                    .console()
                    .ask_allow_replaced_filesystem()
                    .map_cli_error(CliErrorKind::UnspecifiedError)?
                {
                    return Err(check_result).map_cli_error(|_| CliErrorKind::FilesystemIdChanged);
                }
            }
        }
        // Update local state (or create it if it didn't exist yet)
        basedir_metadata
            .update_filesystem_id_for_basedir(
                &mount_args.basedir,
                config.config().filesystem_id,
                &self.local_state_dir,
            )
            .map_cli_error(CliErrorKind::UnspecifiedError)?;

        Ok(())
    }

    fn mount_args(&self) -> &MountArgs {
        self.args.mount.as_ref().expect("Mount args not set")
    }

    fn password_provider(&self) -> &'static dyn PasswordProvider {
        if self.is_noninteractive {
            // TODO Make sure we have tests for noninteractive mode
            &NoninteractivePasswordProvider
        } else {
            &InteractivePasswordProvider
        }
    }

    fn console(&self) -> impl Console {
        // TODO Implement NoninteractiveConsole
        InteractiveConsole::new()
    }
}

// TODO Tests
