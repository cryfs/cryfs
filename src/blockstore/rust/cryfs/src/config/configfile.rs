use anyhow::Result;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

use super::cryconfig::{CryConfig, FILESYSTEM_FORMAT_VERSION};
use super::serialization::{deserialize, serialize};

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
}

pub fn create(path: &Path) -> Result<CryConfig, CreateConfigFileError> {
    // TODO Set values correctly
    let config = CryConfig {
        root_blob: "".to_string(),
        enc_key: "".to_string(),
        cipher: "".to_string(),
        version: FILESYSTEM_FORMAT_VERSION.to_string(),
        created_with_version: FILESYSTEM_FORMAT_VERSION.to_string(),
        last_opened_with_version: FILESYSTEM_FORMAT_VERSION.to_string(),
        blocksize_bytes: 0,
        filesystem_id: [0; 16],
        exclusive_client_id: None,
    };

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
    write_config_to_file(file, config.clone())?;

    Ok(config)
}

fn write_config_to_file(mut file: File, config: CryConfig) -> Result<(), CreateConfigFileError> {
    let serialized_config = serialize(config);

    // TODO Encrypt

    file.write_all(serialized_config.as_bytes())
        .map_err(CreateConfigFileError::IoError)?;

    Ok(())
}

// TODO Tests, including error cases
