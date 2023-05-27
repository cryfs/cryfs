use anyhow::{bail, Context, Result};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::{BufReader, BufWriter, ErrorKind, SeekFrom, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

use cryfs_utils::crypto::{
    kdf::scrypt::{Scrypt, ScryptParams, ScryptSettings},
    symmetric::EncryptionKey,
};

use super::cryconfig::{CryConfig, FILESYSTEM_FORMAT_VERSION};
use super::encryption::ConfigEncryptionKey;
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

    #[error("Error generating scrypt parameters: {0}")]
    ScryptError(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum SaveConfigFileError {
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

    #[error("Error generating scrypt parameters: {0}")]
    ScryptError(anyhow::Error),
}

#[derive(Error, Debug)]
pub enum LoadConfigFileError {
    #[error("Config file not found at {path} : {error}")]
    ConfigFileNotFound {
        path: PathBuf,
        error: std::io::Error,
    },

    #[error("Permission to create the config file denied: {error}")]
    PermissionDenied { error: std::io::Error },

    #[error("IO Error trying to create config file: {0}")]
    IoError(std::io::Error),

    #[error("Error deserializing the config file: {0}")]
    DeserializationError(anyhow::Error),
}

pub enum Access {
    /// Never write to the config file, just read it.
    /// Note that this is only sound if the file system itself
    /// is also loaded read-only, or at least with migrations disabled.
    /// Otherwise, the file system might get migrated but the config
    /// file will still say it's the old version.
    ReadOnly,

    /// Load the config file and update it if necessary,
    /// e.g. write the "last opened with" entry into it
    /// and potentially upgrade the version number.
    ReadWrite,
}

pub struct CryConfigFile {
    path: PathBuf,
    config: CryConfig,
    access: Access,
    kdf_parameters: ScryptParams,
    config_encryption_key: ConfigEncryptionKey,
    modified: bool,
}

impl CryConfigFile {
    pub fn create_new(
        path: PathBuf,
        config: CryConfig,
        password: &str,
        kdf_settings: &ScryptSettings,
    ) -> Result<CryConfigFile, CreateConfigFileError> {
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| match error.kind() {
                ErrorKind::AlreadyExists => CreateConfigFileError::AlreadyExists {
                    path: path.clone(),
                    error,
                },
                ErrorKind::NotFound => CreateConfigFileError::DirectoryComponentDoesntExist {
                    path: path.clone(),
                    error,
                },
                ErrorKind::PermissionDenied => CreateConfigFileError::PermissionDenied { error },
                ErrorKind::InvalidInput => panic!("Invalid set of open options"),
                // TODO Other possible errors?
                _ => CreateConfigFileError::IoError(error),
            })?;
        let kdf_parameters = ScryptParams::generate(kdf_settings)
            .context("Trying to generate scrypt parameters")
            .map_err(CreateConfigFileError::ScryptError)?;
        let config_encryption_key =
            ConfigEncryptionKey::derive::<Scrypt>(&kdf_parameters, password);

        let result = Self {
            path,
            config,
            config_encryption_key,
            kdf_parameters,
            access: Access::ReadWrite,
            modified: false,
        };
        result
            ._write(file)
            .context("Trying to write config to new file")
            .map_err(CreateConfigFileError::SerializationError)?;
        // TODO Have super::encryption::encrypt/decrypt return more structured errors and then correctly map them here, e.g. have an error return for "DecryptionFailed".
        Ok(result)
    }

    pub fn load(
        path: PathBuf,
        password: &str,
        access: Access,
    ) -> Result<CryConfigFile, LoadConfigFileError> {
        let file =
            OpenOptions::new()
                .read(true)
                .open(&path)
                .map_err(|error| match error.kind() {
                    ErrorKind::NotFound => LoadConfigFileError::ConfigFileNotFound {
                        path: path.clone(),
                        error,
                    },
                    ErrorKind::PermissionDenied => LoadConfigFileError::PermissionDenied { error },
                    ErrorKind::InvalidInput => panic!("Invalid set of open options"),
                    // TODO Other possible errors?
                    _ => LoadConfigFileError::IoError(error),
                })?;
        let (config_encryption_key, kdf_parameters, config) =
            super::encryption::decrypt::<Scrypt>(&mut BufReader::new(file), password)
                .map_err(LoadConfigFileError::DeserializationError)?;
        Ok(Self {
            path,
            config,
            config_encryption_key,
            kdf_parameters,
            access,
            modified: false,
        })
    }

    pub fn save(&mut self) -> Result<(), SaveConfigFileError> {
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .map_err(|error| match error.kind() {
                ErrorKind::NotFound => SaveConfigFileError::DirectoryComponentDoesntExist {
                    path: self.path.clone(),
                    error,
                },
                ErrorKind::PermissionDenied => SaveConfigFileError::PermissionDenied { error },
                ErrorKind::InvalidInput => panic!("Invalid set of open options"),
                // TODO Other possible errors?
                _ => SaveConfigFileError::IoError(error),
            })?;
        self._write(file)
            .context("Trying to write config to file")
            .map_err(SaveConfigFileError::SerializationError)?;
        self.modified = false;

        Ok(())
    }

    pub fn save_if_modified_and_has_readwrite_access(&mut self) -> Result<(), SaveConfigFileError> {
        if self.modified {
            match self.access {
                Access::ReadOnly => Ok(()),
                Access::ReadWrite => self.save(),
            }
        } else {
            Ok(())
        }
    }

    fn _write(&self, file: File) -> Result<()> {
        match self.access {
            Access::ReadOnly => {
                bail!("Trying to write to a config file while in read-only mode. Aborting write.")
            }
            Access::ReadWrite => (),
        }
        super::encryption::encrypt::<Scrypt>(
            self.config.clone(),
            self.kdf_parameters.clone(),
            &self.config_encryption_key,
            &mut BufWriter::new(file),
        )?;
        Ok(())
    }

    pub fn config(&self) -> &CryConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut CryConfig {
        self.modified = true;
        &mut self.config
    }

    pub fn into_config(self) -> CryConfig {
        self.config
    }
}

// TODO Tests, including error cases
