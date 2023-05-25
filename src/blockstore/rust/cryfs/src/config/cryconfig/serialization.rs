use serde::{Deserialize, Serialize};
use serde_with::{formats::Uppercase, hex::Hex, serde_as, DisplayFromStr};
use thiserror::Error;
use version_compare::Cmp;

use super::cryconfig::CryConfig;

#[derive(Error, Debug)]
pub enum DeserializationError {
    #[error("File system format is {read_version}, which is not supported anymore. Please use CryFS 0.10 or 0.11 to migrate it to a newer format.")]
    VersionTooOld { read_version: String },

    #[error("File system format is {read_version}, which is not supported yet. Please use a newer version of CryFS to access it.")]
    VersionTooNew { read_version: String },

    #[error("Invalid file system config file: {message}")]
    InvalidConfig { message: String },

    #[error("Failed to deserialize JSON from config file: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn serialize(config: CryConfig) -> String {
    serde_json::to_string(&SerializableCryConfig {
        cryfs: SerializableCryConfigInner {
            root_blob: config.root_blob,
            enc_key: config.enc_key,
            cipher: config.cipher,
            version: Some(config.version),
            created_with_version: Some(config.created_with_version),
            last_opened_with_version: Some(config.last_opened_with_version),
            blocksize_bytes: Some(config.blocksize_bytes),
            filesystem_id: config.filesystem_id,
            exclusive_client_id: config.exclusive_client_id,

            migrations: Some(SerializableCryConfigInnerMigrations {
                // This is a trigger to recognize old file systems that didn't have version numbers.
                // In CryFS 0.10, it is expected to be present and set to true.
                deprecated_has_version_numbers: Some(true),

                // This is a trigger to recognize old file systems that didn't have version numbers.
                // In CryFS 0.10, it is expected to be present and set to true.
                deprecated_has_parent_pointers: Some(true),
            }),
        },
    })
    .expect("Failed to serialize CryConfig")
}

pub fn deserialize(config: &str) -> Result<CryConfig, DeserializationError> {
    let config: SerializableCryConfig = serde_json::from_str(config)?;

    let version = check_format_version(&config.cryfs)?;

    let migrations =
        config
            .cryfs
            .migrations
            .ok_or_else(|| DeserializationError::InvalidConfig {
                message: format!(
        "File system version is {version} but migrations are not set. This should be impossible.",
    ),
            })?;

    if migrations.deprecated_has_version_numbers != Some(true) {
        return Err(
            DeserializationError::InvalidConfig{message: format!(
                "File system version is {version} but hasVersionNumbers is not set to true. This should be impossible.",
            )}
        );
    }

    if migrations.deprecated_has_parent_pointers != Some(true) {
        return Err(
        DeserializationError::InvalidConfig{message: format!(
            "File system version is {version} but hasVersionNumbers is not set to true. This should be impossible.",
        )});
    }

    let created_with_version = config.cryfs.created_with_version.ok_or_else(|| {
        // In CryFS <= 0.9.2, we didn't have this field
        DeserializationError::InvalidConfig{message: format!(
            "File system version is {version} but createdWithVersion is not set. This should be impossible.",
        )}
    })?;

    let last_opened_with_version = config.cryfs.last_opened_with_version.ok_or_else(|| {
        // In CryFS <= 0.9.8, we didn't have this field
        DeserializationError::InvalidConfig{message:format!(
            "File system version is {version} but lastOpenedWithVersion is not set. This should be impossible.",
        )}
    })?;

    let blocksize_bytes = config.cryfs.blocksize_bytes.ok_or_else(|| {
        // CryFS <= 0.9.2 didn't have this field
        DeserializationError::InvalidConfig{message:format!(
            "File system version is {version} but blocksizeBytes is not set. This should be impossible.",
        )}
    })?;

    Ok(CryConfig {
        root_blob: config.cryfs.root_blob,
        enc_key: config.cryfs.enc_key,
        cipher: config.cryfs.cipher,
        version,
        created_with_version,
        last_opened_with_version,
        blocksize_bytes,
        filesystem_id: config.cryfs.filesystem_id,
        exclusive_client_id: config.cryfs.exclusive_client_id,
    })
}

fn check_format_version(
    config: &SerializableCryConfigInner,
) -> Result<String, DeserializationError> {
    let version = config.version.clone().ok_or_else(|| {
        DeserializationError::VersionTooOld {
            // CryFS 0.8 didn't specify this field, so if the field doesn't exist, it's 0.8.
            read_version: "0.8".to_string(),
        }
    })?;

    let version_cmp = version_compare::compare(&version, super::FILESYSTEM_FORMAT_VERSION)
        .map_err(|()| DeserializationError::InvalidConfig {
            message: format!("Invalid file system version: {version}"),
        })?;

    match version_cmp {
        Cmp::Eq => Ok(version),
        Cmp::Gt => Err(DeserializationError::VersionTooNew {
            read_version: version,
        }),
        Cmp::Lt => Err(DeserializationError::VersionTooOld {
            read_version: version,
        }),
        Cmp::Ge | Cmp::Le | Cmp::Ne => {
            panic!("version_compare::compare returned unexpected result")
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SerializableCryConfig {
    cryfs: SerializableCryConfigInner,
}

/// This is mostly identical to [CryConfig], but it allows for backwards compatible serialization,
/// e.g. by having ways to compute fields that were added later.
#[serde_as]
#[derive(Serialize, Deserialize)]
struct SerializableCryConfigInner {
    #[serde(rename = "rootblob")]
    root_blob: String,

    #[serde(rename = "key")]
    enc_key: String,

    #[serde(rename = "cipher")]
    cipher: String,

    #[serde(rename = "version")]
    version: Option<String>,

    #[serde(rename = "createdWithVersion")]
    created_with_version: Option<String>,

    #[serde(rename = "lastOpenedWithVersion")]
    last_opened_with_version: Option<String>,

    #[serde(rename = "blocksizeBytes")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    blocksize_bytes: Option<u64>,

    #[serde(rename = "filesystemId")]
    #[serde_as(as = "Hex<Uppercase>")]
    filesystem_id: [u8; 16],

    #[serde(rename = "exclusiveClientId")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    exclusive_client_id: Option<u32>,

    migrations: Option<SerializableCryConfigInnerMigrations>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
struct SerializableCryConfigInnerMigrations {
    #[serde(rename = "hasVersionNumbers")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    deprecated_has_version_numbers: Option<bool>,

    #[serde(rename = "hasParentPointers")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    deprecated_has_parent_pointers: Option<bool>,
}

// TODO Tests, including deserialization errors and different file system version numbers.
//      Also test that we can still read the JSON format as created by C++ in different scenarios.
