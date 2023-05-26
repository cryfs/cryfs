use anyhow::{Context, Result};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

use cryfs_utils::crypto::kdf::scrypt::{Scrypt, ScryptSettings};

use super::cryconfig::{CryConfig, FILESYSTEM_FORMAT_VERSION};
use super::password_provider::PasswordProvider;

#[derive(Error, Debug)]
pub enum CreateConfigFileError {
    #[error("Config file already exists: {path}\nError: {error}")]
    AlreadyExists {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("A directory on the path to the config file doesn't exist: {path}\nError: {error}")]
    DirectoryComponentDoesntExist {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("Permission to create the config file denied: {error}")]
    PermissionDenied { error: std::io::Error },

    #[error("IO Error trying to create config file: {0}")]
    IoError(std::io::Error),

    #[error("Error serializing the config file: {0}")]
    SerializationError(anyhow::Error),
}

// pub fn load_or_create(
//     path: &Path,
//     password_provider: impl PasswordProvider,
// )

pub fn create_new(
    path: &Path,
    config: CryConfig,
    password: &str,
    kdf_settings: &ScryptSettings,
) -> Result<CryConfig, CreateConfigFileError> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| match error.kind() {
            ErrorKind::AlreadyExists => CreateConfigFileError::AlreadyExists {
                path: path.to_path_buf(),
                error,
            },
            ErrorKind::NotFound => CreateConfigFileError::DirectoryComponentDoesntExist {
                path: path.to_path_buf(),
                error,
            },
            ErrorKind::PermissionDenied => CreateConfigFileError::PermissionDenied { error },
            ErrorKind::InvalidInput => panic!("Invalid set of open options"),
            _ => CreateConfigFileError::IoError(error),
        })?;
    write_config_to_file(file, config.clone(), password, kdf_settings)?;

    Ok(config)
}

fn write_config_to_file(
    mut file: File,
    config: CryConfig,
    password: &str,
    kdf_settings: &ScryptSettings,
) -> Result<(), CreateConfigFileError> {
    super::encryption::encrypt::<Scrypt>(config, password, kdf_settings, &mut file)
        .context("Trying to write config to file")
        .map_err(CreateConfigFileError::SerializationError)?;

    Ok(())
}

// TODO Tests, including error cases
