use anyhow::{Context, Result};
use std::num::NonZeroU32;
use std::path::PathBuf;
use thiserror::Error;

use super::configfile::{
    Access, CreateConfigFileError, CryConfigFile, LoadConfigFileError, SaveConfigFileError,
};
use super::console::Console;
use super::creator::ConfigCreateError;
use super::password_provider::PasswordProvider;
use super::CryConfig;
use crate::localstate::{FilesystemMetadata, LocalStateDir};
use cryfs_blockstore::ClientId;
use cryfs_utils::crypto::symmetric::EncryptionKey;
use cryfs_version::{Version, VersionInfo};

use crate::config::FILESYSTEM_FORMAT_VERSION;
pub const CRYFS_VERSION: VersionInfo = crate::version::CRYFS_VERSION;
const MIN_SUPPORTED_FORMAT_VERSION: Version = konst::unwrap_ctx!(Version::parse_const("0.10"));
const MAX_SUPPORTED_FORMAT_VERSION: Version = FILESYSTEM_FORMAT_VERSION;

#[derive(Error, Debug)]
pub enum ConfigLoadError {
    #[error("Invalid data in config file: {0:?}")]
    InvalidConfig(anyhow::Error),

    #[error("This filesystem is for CryFS {actual_format_version} but you're running CryFS {cryfs_version} which needs at least file system format version {min_supported_format_version}. Please migrate the file system to a supported version first by opening it with CryFS {min_supported_format_version}")]
    TooOldFilesystemFormat {
        // TODO Store Version object instead of String
        actual_format_version: String,
        min_supported_format_version: Version<'static>,
        cryfs_version: VersionInfo<'static, 'static, 'static>,
    },

    #[error("This filesystem is for CryFS {actual_format_version} and would have to be migrated to {max_supported_format_version} to be used with CryFS {cryfs_version} but the migration was declined.")]
    TooOldFilesystemFormatDeclinedMigration {
        // TODO Store Version object instead of String
        actual_format_version: String,
        max_supported_format_version: Version<'static>,
        cryfs_version: VersionInfo<'static, 'static, 'static>,
    },

    #[error("This filesystem is in the format of CryFS {actual_format_version} but you're running CryFS {cryfs_version}, which uses file system format {max_supported_format_version}. Please update your CryFS version.")]
    TooNewFilesystemFormat {
        // TODO Store Version object instead of String
        actual_format_version: String,
        max_supported_format_version: Version<'static>,
        cryfs_version: VersionInfo<'static, 'static, 'static>,
    },

    #[error("Error loading config file: {0:?}")]
    LoadFileError(#[from] LoadConfigFileError),

    #[error("Error saving config file modifications: {0:?}")]
    SaveFileError(#[from] SaveConfigFileError),

    #[error("Error creating config: {0:?}")]
    ConfigCreateError(#[from] ConfigCreateError),

    #[error("Error creating config file: {0:?}")]
    CreateFileError(#[from] CreateConfigFileError),

    #[error("Wrong cipher: Expected {expected_cipher} but found {actual_cipher}")]
    WrongCipher {
        expected_cipher: String,
        actual_cipher: String,
    },

    #[error("Error checking the local state of the file system: {0:?}")]
    LocalStateError(anyhow::Error),

    #[error("You specified on the command line to treat missing blocks as integrity violations, but the file system is not setup to do that.")]
    FilesystemDoesNotTreatMissingBlocksAsIntegrityViolations,

    #[error("You specified on the command line to not treat missing blocks as integrity violations, but the file system is setup to do that.")]
    FilesystemTreatsMissingBlocksAsIntegrityViolations,

    #[error("File system is in single-client mode and can only be used from the client that created it.")]
    FilesystemInSingleClientMode,

    #[error("Error interacting with the user: {0:?}")]
    InteractionError(anyhow::Error),
}

pub struct ConfigLoadResult {
    // loading a config file updates the config file, e.g. the "lastOpenedWith" field, but this member keeps the original config
    pub old_config: CryConfig,

    pub config: CryConfigFile,

    pub my_client_id: ClientId,

    // True if this config file / file system was just created and wasn't opened from an existing config file
    pub first_time_access: bool,
}

#[derive(Clone, Copy)]
pub struct CommandLineFlags<'a> {
    pub missing_block_is_integrity_violation: Option<bool>,
    pub expected_cipher: Option<&'a str>,
}

pub fn load_or_create(
    filename: PathBuf,
    password: &(impl PasswordProvider + ?Sized),
    console: &(impl Console + ?Sized),
    command_line_flags: &CommandLineFlags,
    local_state_dir: &LocalStateDir,
) -> Result<ConfigLoadResult, ConfigLoadError> {
    if filename.exists() {
        // TODO Protect password similar to how we protect EncryptionKey
        let password = password
            .password_for_existing_filesystem()
            .map_err(ConfigLoadError::InteractionError)?;
        _load(
            filename,
            &password,
            console,
            command_line_flags,
            local_state_dir,
            Access::ReadWrite,
        )
    } else {
        // TODO Protect password similar to how we protect EncryptionKey
        let password = password
            .password_for_new_filesystem()
            .map_err(ConfigLoadError::InteractionError)?;
        _create(
            filename,
            &password,
            console,
            command_line_flags,
            local_state_dir,
        )
    }
}

pub fn load_readonly(
    filename: PathBuf,
    password: &(impl PasswordProvider + ?Sized),
    console: &(impl Console + ?Sized),
    command_line_flags: &CommandLineFlags,
    local_state_dir: &LocalStateDir,
) -> Result<ConfigLoadResult, ConfigLoadError> {
    let password = password
        .password_for_existing_filesystem()
        .map_err(ConfigLoadError::InteractionError)?;
    _load(
        filename,
        &password,
        console,
        command_line_flags,
        local_state_dir,
        Access::ReadOnly,
    )
}

fn _create(
    filename: PathBuf,
    password: &str,
    console: &(impl Console + ?Sized),
    command_line_flags: &CommandLineFlags,
    local_state_dir: &LocalStateDir,
) -> Result<ConfigLoadResult, ConfigLoadError> {
    let config = super::creator::create(console, command_line_flags, local_state_dir)?;
    let file = CryConfigFile::create_new(
        filename,
        config.config.clone(),
        password,
        &console
            .ask_scrypt_settings_for_new_filesystem()
            .map_err(ConfigLoadError::InteractionError)?,
    )?;
    Ok(ConfigLoadResult {
        old_config: config.config,
        config: file,
        my_client_id: config.my_client_id,
        first_time_access: true,
    })
}

fn _load(
    filename: PathBuf,
    password: &str,
    console: &(impl Console + ?Sized),
    command_line_flags: &CommandLineFlags,
    local_state_dir: &LocalStateDir,
    access: Access,
) -> Result<ConfigLoadResult, ConfigLoadError> {
    let mut configfile: CryConfigFile = CryConfigFile::load(filename, password, access)?;
    let old_config = configfile.config().clone();
    _check_version(configfile.config(), console)?;
    _update_version_in_config(&mut configfile);
    _check_cipher(configfile.config(), command_line_flags.expected_cipher)?;
    let local_state = FilesystemMetadata::load_or_generate(
        local_state_dir,
        &configfile.config().filesystem_id,
        &EncryptionKey::from_hex(&configfile.config().enc_key)
            .context("Tried to read encryption key from config")
            .map_err(ConfigLoadError::InvalidConfig)?,
        console,
    )
    .map_err(ConfigLoadError::LocalStateError)?;
    let my_client_id = *local_state.my_client_id();
    _check_missing_blocks_are_integrity_violations(
        &mut configfile,
        my_client_id,
        command_line_flags,
        console,
    )?;
    configfile.save_if_modified_and_has_readwrite_access()?;
    Ok(ConfigLoadResult {
        old_config,
        config: configfile,
        my_client_id,
        first_time_access: false,
    })
}

fn _check_version(
    config: &CryConfig,
    console: &(impl Console + ?Sized),
) -> Result<(), ConfigLoadError> {
    let actual_format_version = Version::parse(&config.format_version).map_err(|_| {
        ConfigLoadError::InvalidConfig(anyhow::anyhow!(
            "Could not parse format version number {} from config file",
            config.format_version
        ))
    })?;
    // TODO Make these asserts const_assert
    assert!(CRYFS_VERSION.version() >= MAX_SUPPORTED_FORMAT_VERSION);
    assert!(MAX_SUPPORTED_FORMAT_VERSION >= MIN_SUPPORTED_FORMAT_VERSION);
    assert!(CRYFS_VERSION.version() >= MIN_SUPPORTED_FORMAT_VERSION);

    if actual_format_version < MIN_SUPPORTED_FORMAT_VERSION {
        return Err(ConfigLoadError::TooOldFilesystemFormat {
            actual_format_version: actual_format_version.to_string(),
            min_supported_format_version: MIN_SUPPORTED_FORMAT_VERSION,
            cryfs_version: CRYFS_VERSION,
        });
    }
    if actual_format_version > MAX_SUPPORTED_FORMAT_VERSION {
        return Err(ConfigLoadError::TooNewFilesystemFormat {
            actual_format_version: actual_format_version.to_string(),
            max_supported_format_version: MAX_SUPPORTED_FORMAT_VERSION,
            cryfs_version: CRYFS_VERSION,
        });
    }
    if actual_format_version < MAX_SUPPORTED_FORMAT_VERSION {
        let allow_migration = console
            .ask_migrate_filesystem(
                &actual_format_version,
                &MAX_SUPPORTED_FORMAT_VERSION,
                &CRYFS_VERSION,
            )
            .map_err(ConfigLoadError::InteractionError)?;
        if !allow_migration {
            return Err(ConfigLoadError::TooOldFilesystemFormatDeclinedMigration {
                actual_format_version: actual_format_version.to_string(),
                max_supported_format_version: MAX_SUPPORTED_FORMAT_VERSION,
                cryfs_version: CRYFS_VERSION,
            });
        }
    }
    Ok(())
}

fn _update_version_in_config(config: &mut CryConfigFile) {
    // TODO No unwrap
    if Version::parse(&config.config().format_version).unwrap() != FILESYSTEM_FORMAT_VERSION {
        config.config_mut().format_version = FILESYSTEM_FORMAT_VERSION.to_string();
    }
    if config.config().last_opened_with_version != CRYFS_VERSION.to_string() {
        config.config_mut().last_opened_with_version = CRYFS_VERSION.to_string();
    }
}

fn _check_cipher(config: &CryConfig, expected_cipher: Option<&str>) -> Result<(), ConfigLoadError> {
    if let Some(expected_cipher) = expected_cipher {
        if config.cipher != expected_cipher {
            return Err(ConfigLoadError::WrongCipher {
                actual_cipher: config.cipher.clone(),
                expected_cipher: expected_cipher.to_string(),
            });
        }
    }
    Ok(())
}

fn _check_missing_blocks_are_integrity_violations(
    config: &mut CryConfigFile,
    my_client_id: ClientId,
    command_line_flags: &CommandLineFlags,
    console: &(impl Console + ?Sized),
) -> Result<(), ConfigLoadError> {
    if command_line_flags.missing_block_is_integrity_violation == Some(true)
        && config.config().exclusive_client_id.is_none()
    {
        return Err(ConfigLoadError::FilesystemDoesNotTreatMissingBlocksAsIntegrityViolations);
    }
    if command_line_flags.missing_block_is_integrity_violation == Some(false)
        && config.config().exclusive_client_id.is_some()
    {
        return Err(ConfigLoadError::FilesystemTreatsMissingBlocksAsIntegrityViolations);
    }
    if let Some(exclusive_client_id) = config.config().exclusive_client_id {
        if (ClientId {
            id: NonZeroU32::try_from(exclusive_client_id)
                .map_err(|err| ConfigLoadError::InvalidConfig(err.into()))?,
        }) != my_client_id
        {
            let disable_single_client_mode = console
                .ask_disable_single_client_mode()
                .map_err(ConfigLoadError::InteractionError)?;
            if !disable_single_client_mode {
                return Err(ConfigLoadError::FilesystemInSingleClientMode);
            }
            config.config_mut().exclusive_client_id = None;
        }
    }
    Ok(())
}

// TODO Tests
