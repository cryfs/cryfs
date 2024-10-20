use anyhow::Result;
use cryfs_runner::{CreateOrLoad, Mounter};

use super::console::InteractiveConsole;
use crate::args::{CryfsArgs, MountArgs};
use cryfs_blockstore::AllowIntegrityViolations;
use cryfs_cli_utils::password_provider::{
    InteractivePasswordProvider, NoninteractivePasswordProvider,
};
use cryfs_cli_utils::{
    print_config, Application, CliError, CliErrorKind, CliResultExt, CliResultExtFn, Environment,
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
        let local_state_dir = cryfs_filesystem::localstate::LocalStateDir::new(env.local_state_dir);

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

        let mounter = if self.mount_args().foreground {
            Mounter::run_in_foreground().map_cli_error(CliErrorKind::UnspecifiedError)?
        } else {
            // We need to daemonize **before** initializing tokio because tokio doesn't support fork, see https://github.com/tokio-rs/tokio/issues/4301
            Mounter::run_in_background().map_cli_error(CliErrorKind::UnspecifiedError)?
        };

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
        // TODO C++ code has lots more logic here, migrate that.

        let config = self.load_or_create_config(progress_bars)?;
        print_config(&config);

        let on_successfully_mounted = || {
            println!(
                "CryFS has been successfully mounted to {}",
                mount_args.mountdir.display()
            );
            if mount_args.foreground {
                println!(
                    // TODO Add necessary escape sequences to the mountdir path, e.g. " -> \"
                    "You can unmount it by pressing Ctrl+C or by running `cryfs-unmount \"{}\"`.",
                    mount_args.mountdir.display(),
                );
            } else {
                println!(
                    // TODO Add necessary escape sequences to the mountdir path, e.g. " -> \"
                    "You can unmount it by running `cryfs-unmount \"{}\"`.",
                    mount_args.mountdir.display(),
                );
            }
            println!("To see more information, run `cryfs --help`.");
        };

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
                },
                on_successfully_mounted,
            )
            .await?;

        if mount_args.foreground {
            // In foreground mode, we only return after unmount
            println!("CryFS has been unmounted.");
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
        progress_bars: impl ProgressBarManager,
    ) -> Result<ConfigLoadResult, CliError> {
        // TODO Allow changing config file using args as C++ did
        let mount_args = self.mount_args();
        let config_file_location = mount_args.basedir.join("cryfs.config");
        cryfs_filesystem::config::load_or_create(
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
