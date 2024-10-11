use anyhow::Result;
use daemonize::Daemonize;

use super::console::InteractiveConsole;
use super::runner::FilesystemRunner;
use crate::args::{CryfsArgs, MountArgs};
use cryfs_blockstore::{
    AllowIntegrityViolations, IntegrityConfig, MissingBlockIsIntegrityViolation, OnDiskBlockStore,
};
use cryfs_cli_utils::password_provider::{
    InteractivePasswordProvider, NoninteractivePasswordProvider,
};
use cryfs_cli_utils::{
    print_config, setup_blockstore_stack, Application, CliError, CliErrorKind, CliResultExt,
    CliResultExtFn, Environment,
};
use cryfs_cryfs::CRYFS_VERSION;
use cryfs_cryfs::{
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
    fn new(args: CryfsArgs, env: Environment) -> Result<Self, CliError> {
        let is_noninteractive = env.is_noninteractive;

        // TODO Make sure we have tests for the local_state_dir location
        let local_state_dir = cryfs_cryfs::localstate::LocalStateDir::new(env.local_state_dir);

        Ok(Self {
            is_noninteractive,
            args,
            local_state_dir,
        })
    }

    fn main(self) -> Result<(), CliError> {
        if self.args.show_ciphers {
            self.show_ciphers();
            return Ok(());
        }

        self.sanity_checks()?;
        self.run_filesystem(ConsoleProgressBarManager)?;

        Ok(())
    }
}

impl Cli {
    fn sanity_checks(&self) -> Result<(), CliError> {
        let mount_args = self.mount_args();
        super::sanity_checks::check_mountdir_doesnt_contain_basedir(mount_args)
            .map_cli_error(CliErrorKind::BaseDirInsideMountDir)?;
        super::sanity_checks::check_dir_accessible(
            &mount_args.basedir,
            "vault",
            mount_args.create_missing_basedir,
            |path| self.console().ask_create_basedir(path),
        )
        .map_cli_error(CliErrorKind::InaccessibleBaseDir)?;
        // TODO C++ had special handling of Windows drive letters here. We should probably re-add that
        super::sanity_checks::check_dir_accessible(
            &mount_args.mountdir,
            "mountpoint",
            mount_args.create_missing_mountpoint,
            |path| self.console().ask_create_mountdir(path),
        )
        .map_cli_error(CliErrorKind::InaccessibleMountDir)?;
        Ok(())
    }

    fn run_filesystem(&self, progress_bars: impl ProgressBarManager) -> Result<(), CliError> {
        // TODO Making cryfs-cli init code async could speed it up, e.g. do update checks while creating basedirs or loading the config.
        //      However, we cannot use tokio before we daemonize (https://github.com/tokio-rs/tokio/issues/4301) and we would like to daemonize
        //      as late as possible so we can show error messages to the user if something goes wrong. But fork+exec is fine, just fork by itself breaks tokio.
        //      Maybe we need to split out a cryfs-mount executable and call that from cryfs-cli? But then how do the executables find each other?
        //      Or use the same binary with different arguments? Maybe `cryfs` just calls `cryfs -f`? Or maybe use a different async executor before daemonizing?

        let mount_args = self.mount_args();
        // TODO C++ code has lots more logic here, migrate that.

        let config = self.load_or_create_config(progress_bars)?;
        print_config(&config);

        // We need to daemonize **before** initializing tokio because tokio doesn't support fork, see https://github.com/tokio-rs/tokio/issues/4301
        // TODO Can we delay daemonization until after we know the filesystem was mounted successfully?
        self.maybe_daemonize(&mount_args)
            .map_cli_error(CliErrorKind::UnspecifiedError)?;

        // TODO Runtime settings
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name(Self::NAME)
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(setup_blockstore_stack(
            OnDiskBlockStore::new(mount_args.basedir.to_owned()),
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
                mountdir: &mount_args.mountdir,
                config: &config,
            },
        ))??;

        Ok(())
    }

    fn show_ciphers(&self) {
        for cipher in cryfs_cryfs::config::ALL_CIPHERS {
            println!("{}", cipher);
        }
    }

    fn maybe_daemonize(&self, mount_args: &MountArgs) -> Result<()> {
        if mount_args.foreground {
            println!("Mounting in foreground mode. CryFS will not exit until the filesystem is unmounted.");
        } else {
            println!("Mounting in background mode. CryFS will continue to run in the background.");
            let umask = unsafe { libc::umask(0) }; // get current umask value because `daemonize` force overwrites it but we don't really want it to change, so we give it the old value
            Daemonize::new().umask(umask).start()?;
            println!("We're in background now");
        }
        Ok(())
    }

    // TODO Test the console flows for opening an existing/creating a new file system
    fn load_or_create_config(
        &self,
        progress_bars: impl ProgressBarManager,
    ) -> Result<ConfigLoadResult, CliError> {
        // TODO Allow changing config file using args as C++ did
        let mount_args = self.mount_args();
        let config_file_location = mount_args.basedir.join("cryfs.config");
        cryfs_cryfs::config::load_or_create(
            config_file_location.to_owned(),
            self.password_provider(),
            self.console(),
            &CommandLineFlags {
                missing_block_is_integrity_violation: mount_args
                    .missing_block_is_integrity_violation,
                expected_cipher: mount_args.cipher.clone(),
            },
            &self.local_state_dir,
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
        })
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

    fn console(&self) -> &'static dyn Console {
        // TODO Implement NoninteractiveConsole
        &InteractiveConsole
    }
}

// TODO Tests
