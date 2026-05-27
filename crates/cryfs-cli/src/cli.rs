use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, Result};
use clap_logflag::{LogArgs, LogDestination, LogDestinationConfig, LoggingConfig};
use cryfs_config::config::CryConfigFile;
use cryfs_config::localstate::{CheckFilesystemIdError, VaultdirMetadata};
use cryfs_runner::CreateOrLoad;
use daemonizable::{Daemonizable, Daemonizer, RpcClient, RpcServer};
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
use cryfs_config::CRYFS_VERSION;
use cryfs_config::{
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
//   - vaultdir/mountdir don't exist
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
    daemonizer: Daemonizer<CryfsApp>,
}

/// The [`Daemonizable`] application cryfs registers with `daemonizable::run`:
/// ties the mount RPC protocol types, the logging bootstrap payload, and the
/// build id together in one type. Lives in the lib rather than the `cryfs`
/// bin so [`Cli`] can name `Daemonizer<CryfsApp>` in its field type.
pub struct CryfsApp;

impl Daemonizable for CryfsApp {
    type Request = cryfs_runner::Request;
    type Response = cryfs_runner::Response;

    fn build_id() -> String {
        // Same "{name} {version}" shape the legacy framework used: name AND
        // version, because two different binaries built from the same
        // workspace commit share the identical version string and the
        // handshake must still tell them apart.
        format!("{} {}", Cli::NAME, CRYFS_VERSION)
    }

    fn run_foreground(daemonizer: Daemonizer<Self>) -> ExitCode {
        // The whole CLI pipeline (human-panic, environment, clap parse,
        // logging init, version banner + update check) lives in cli-utils;
        // construction is injected so `Cli` can carry the daemonizer.
        cryfs_cli_utils::run_with(|args, env| Cli::new(args, env, daemonizer))
    }

    fn run_daemon(rpc: RpcServer<Self::Request, Self::Response>) -> ! {
        // The daemon child bypasses cli-utils' run pipeline — which is what
        // installs the panic hooks on every other path — so re-install them
        // first: the long-lived FUSE daemon must not panic silently once
        // `detach_stdio` has pointed stderr at /dev/null.
        cryfs_cli_utils::setup_panic_handling(Cli::NAME, &CRYFS_VERSION.to_string());
        // The daemon side doesn't need any of `Cli`'s parent-side state
        // (password prompts, config loading, sanity checks). It receives
        // fully-prepared `MountArgs` — plus the logging config it installs
        // before mounting — over RPC; just drive the request loop.
        // `background_main` initializes tokio inside this fresh process image.
        cryfs_runner::background_main(rpc)
    }
}

impl Application for Cli {
    type ConcreteArgs = CryfsArgs;
    const NAME: &'static str = "cryfs";
    const VERSION: VersionInfo<'static, 'static, &'static str> = CRYFS_VERSION;

    fn default_log_config(&self) -> LoggingConfig {
        // Used for the foreground entry point and for the parent CLI in the
        // backgrounding path — both want stderr (the user is watching the
        // terminal for password prompts, mount progress, and errors).
        LoggingConfig::new(vec![LogDestinationConfig {
            destination: LogDestination::Stderr,
            level: Some(LevelFilter::Warn),
        }])
    }

    fn main(self, log_args: LogArgs) -> Result<(), CliError> {
        if self.should_daemonize() {
            // Resolve the user's `--log` flags against the daemon default
            // (syslog) here on the parent side; the daemon can't do it itself
            // (empty argv), so we ship the result in the mount request and it
            // installs it before mounting. Spawn before `run_parent` starts
            // tokio: spawning while single-threaded sidesteps the narrow macOS
            // pipe-fd-inheritance race (a non-issue on Linux, where pipe2 sets
            // CLOEXEC atomically).
            let daemon_log_config = log_args.or_default(self.default_daemon_log_config());
            let mut rpc = self
                .daemonizer
                .spawn_daemon()
                .map_cli_error(|_| CliErrorKind::UnspecifiedError)?;
            self.run_parent(&mut rpc, daemon_log_config)
        } else {
            self.run_foreground()
        }
    }
}

impl Cli {
    /// Constructed through the closure [`CryfsApp::run_foreground`] passes to
    /// `run_with` — not a trait constructor, because the `daemonizer`
    /// capability can only be minted by `daemonizable::run`.
    fn new(
        args: CryfsArgs,
        env: Environment,
        daemonizer: Daemonizer<CryfsApp>,
    ) -> Result<Self, CliError> {
        let is_noninteractive = env.is_noninteractive;
        // TODO Make sure we have tests for the local_state_dir location
        let local_state_dir = cryfs_config::localstate::LocalStateDir::new(env.local_state_dir);

        Ok(Self {
            is_noninteractive,
            args,
            local_state_dir,
            daemonizer,
        })
    }

    fn default_daemon_log_config(&self) -> LoggingConfig {
        // Syslog because the daemon's inherited stderr is `dup2(/dev/null)`'d
        // after a successful mount (via `daemonizable::detach_stdio`) — a
        // stderr-based default would go silent for the bulk of the daemon's
        // lifetime. If the user explicitly passes `--log stderr` we honor it
        // (`main` resolves `LogArgs::or_default(daemon_default)` on the
        // parent side and ships the result in the mount request); but
        // we don't pick stderr as the default.
        LoggingConfig::new(vec![LogDestinationConfig {
            destination: LogDestination::Syslog,
            level: Some(LevelFilter::Warn),
        }])
    }

    fn should_daemonize(&self) -> bool {
        // Daemonize only when we have mount args AND the user didn't request
        // foreground. Short-running invocations like `--show-ciphers` (no
        // mount args) stay foreground.
        self.args
            .mount
            .as_ref()
            .is_some_and(|args| !args.foreground)
    }

    fn run_foreground(self) -> Result<(), CliError> {
        // TODO Once we support Windows, we need to check that we're running on a supported windows version. C++ CryFS only supported Windows 7 or later.

        if self.args.show_ciphers {
            self.show_ciphers();
            return Ok(());
        }

        // Note: tokio-console requires running with `RUSTFLAGS="--cfg tokio_unstable" cargo build`, see https://github.com/tokio-rs/console
        #[cfg(feature = "tokio_console")]
        console_subscriber::init();

        let runtime = cryfs_runner::init_tokio();
        runtime.block_on(self.run_foreground_async())
    }

    fn run_parent(
        self,
        rpc: &mut RpcClient<cryfs_runner::Request, cryfs_runner::Response>,
        log_config: LoggingConfig,
    ) -> Result<(), CliError> {
        // `should_daemonize` only returned `true` because we had mount args,
        // so `show_ciphers` etc never reach here.
        #[cfg(feature = "tokio_console")]
        console_subscriber::init();

        let runtime = cryfs_runner::init_tokio();
        runtime.block_on(self.run_parent_async(rpc, log_config))
    }
    async fn run_foreground_async(self) -> Result<(), CliError> {
        // TODO Making cryfs-cli init code async could speed it up, e.g. do update checks while creating vaultdirs or loading the config.
        self.sanity_checks().await?;
        let mountdir = self.mount_args().mountdir.clone();
        let mount_args = self.build_mount_args(ConsoleProgressBarManager)?;
        let on_successfully_mounted = || {
            Self::print_mount_success(&mountdir, /* foreground */ true);
            Ok(())
        };
        cryfs_runner::mount_filesystem(mount_args, on_successfully_mounted).await?;
        // In foreground mode, we only return after unmount
        // TODO Output formatting, e.g. colorization (and search the codebase for other println statements that might be missing it)
        println!("  CryFS has been unmounted.");
        Ok(())
    }

    async fn run_parent_async(
        self,
        rpc: &mut RpcClient<cryfs_runner::Request, cryfs_runner::Response>,
        log_config: LoggingConfig,
    ) -> Result<(), CliError> {
        self.sanity_checks().await?;
        let mountdir = self.mount_args().mountdir.clone();
        let mount_args = self.build_mount_args(ConsoleProgressBarManager)?;
        cryfs_runner::parent_mount_filesystem(rpc, mount_args, log_config)?;
        Self::print_mount_success(&mountdir, /* foreground */ false);
        Ok(())
    }

    fn build_mount_args(
        &self,
        progress_bars: impl ProgressBarManager,
    ) -> Result<cryfs_runner::MountArgs, CliError> {
        let mount_args = self.mount_args();
        let config =
            self.load_or_create_config(mount_args.allow_replaced_filesystem, progress_bars)?;
        print_config(&config);

        let (atime_options, fuse_permission_options) =
            FuseOption::partition(&mount_args.fuse_option);

        let atime_behavior = AtimeOption::to_atime_behavior(&atime_options)
            .map_cli_error(CliErrorKind::InvalidArguments)?;

        Ok(cryfs_runner::MountArgs {
            // Resolve to absolute paths before handing them to the (possibly
            // daemonized) runner: the daemon chdir's to `/`, so a cwd-relative
            // vault/mount path would otherwise resolve against the wrong
            // directory. `sanity_checks` above has already ensured both exist.
            vaultdir: std::fs::canonicalize(&mount_args.vaultdir)
                .with_context(|| {
                    format!(
                        "Failed to resolve vault directory {}",
                        mount_args.vaultdir.display()
                    )
                })
                .map_cli_error(CliErrorKind::UnspecifiedError)?,
            mountdir: std::fs::canonicalize(&mount_args.mountdir)
                .with_context(|| {
                    format!(
                        "Failed to resolve mount directory {}",
                        mount_args.mountdir.display()
                    )
                })
                .map_cli_error(CliErrorKind::UnspecifiedError)?,
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
        })
    }

    fn print_mount_success(mountdir: &Path, foreground: bool) {
        // TODO Output formatting, e.g. colorization
        println!(
            "  CryFS has been successfully mounted to {}",
            mountdir.display()
        );
        if foreground {
            println!(
                // TODO Add necessary escape sequences to the mountdir path, e.g. " -> \"
                "  You can unmount it by pressing Ctrl+C or by running `cryfs-unmount \"{}\"`.",
                mountdir.display(),
            );
        } else {
            println!(
                // TODO Add necessary escape sequences to the mountdir path, e.g. " -> \"
                "  You can unmount it by running `cryfs-unmount \"{}\"`.",
                mountdir.display(),
            );
        }
        println!("  To see more information, run `cryfs --help`.");
    }

    async fn sanity_checks(&self) -> Result<(), CliError> {
        let mount_args = self.mount_args();
        super::sanity_checks::check_mountdir_doesnt_contain_vaultdir(mount_args)
            .map_cli_error(CliErrorKind::VaultDirInsideMountDir)?;
        super::sanity_checks::check_dir_accessible(
            &mount_args.vaultdir,
            "vault",
            mount_args.create_missing_vaultdir,
            |path| self.console().ask_create_vaultdir(path),
        )
        .await
        .map_cli_error(CliErrorKind::InaccessibleVaultDir)?;
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

    fn show_ciphers(&self) {
        for cipher in cryfs_config::config::ALL_CIPHERS {
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
        let config = cryfs_config::config::load_or_create(
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
            .unwrap_or_else(|| mount_args.vaultdir.join("cryfs.config"))
    }

    fn check_config_integrity(
        &self,
        config: &CryConfigFile,
        allow_replaced_filesystem: bool,
    ) -> Result<(), CliError> {
        let mount_args = self.mount_args();
        let mut vaultdir_metadata = VaultdirMetadata::load(&self.local_state_dir)
            .context("Failed to load local state")
            .map_cli_error(CliErrorKind::UnspecifiedError)?;
        let check_result = vaultdir_metadata.filesystem_id_for_vaultdir_is_correct(
            &mount_args.vaultdir,
            &config.config().filesystem_id,
        );
        if let Err(check_result) = check_result {
            let CheckFilesystemIdError::FilesystemIdIncorrect {
                vaultdir,
                expected_id,
                actual_id,
            } = &check_result;
            log::warn!(
                "Filesystem id for vault directory {} has changed: expected {:?}, got {:?}",
                vaultdir.display(),
                expected_id,
                actual_id,
            );
            if !allow_replaced_filesystem
                && !self
                    .console()
                    .ask_allow_replaced_filesystem()
                    .map_cli_error(CliErrorKind::UnspecifiedError)?
            {
                return Err(check_result).map_cli_error(|_| CliErrorKind::FilesystemIdChanged);
            }
        }
        // Update local state (or create it if it didn't exist yet)
        vaultdir_metadata
            .update_filesystem_id_for_vaultdir(
                &mount_args.vaultdir,
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
