use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, TryFromInto};
use std::io::{BufReader, BufWriter};
use std::path::Path;
use thiserror::Error;

use cryfs_blockstore::ClientId;
use cryfs_utils::crypto::{
    hash::{hash, Digest, Hash, Salt},
    symmetric::EncryptionKey,
};

use super::LocalStateDir;
use crate::config::{Console, FilesystemId};

#[derive(Error, Debug)]
pub enum FilesystemMetadataError {
    #[error("The filesystem encryption key differs from the last time we loaded this filesystem. Did an attacker replace the file system?")]
    EncryptionKeyChanged,
}

/// Store metadata about file systems we know, e.g. our own client id
/// and a hash of the encryption key so we can recognize if the file system
/// was replaced by an adversary with a file system with a different
/// encryption key.
#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct FilesystemMetadata {
    #[serde(rename = "myClientId", with = "serialize_client_id")]
    my_client_id: ClientId,

    #[serde_as(as = "TryFromInto<SerializedHash>")]
    #[serde(rename = "encryptionKey")]
    encryption_key: Hash,
}

impl FilesystemMetadata {
    pub fn load_or_generate(
        local_state_dir: &LocalStateDir,
        filesystem_id: &FilesystemId,
        encryption_key: &EncryptionKey,
        console: &(impl Console + ?Sized),
        mut allow_replaced_file_system: bool,
        // TODO Return FilesystemMetadataError instead of anyhow::Error
    ) -> Result<Self> {
        let metadata_file_path = local_state_dir
            .for_filesystem_id(filesystem_id)
            .context("Tried to determine location for local filesystem metadata")?
            .join("metadata");
        match Self::_load(&metadata_file_path).context("Tried to load local filesystem metadata")? {
            Some(mut metadata) => {
                if hash(encryption_key.as_bytes(), metadata.encryption_key.salt)
                    != metadata.encryption_key
                {
                    if !allow_replaced_file_system {
                        allow_replaced_file_system = console.ask_allow_changed_encryption_key()?;
                    }
                    if !allow_replaced_file_system {
                        return Err(FilesystemMetadataError::EncryptionKeyChanged.into());
                    }
                    metadata.encryption_key =
                        hash(encryption_key.as_bytes(), Salt::generate_random());
                    metadata
                        ._save(&metadata_file_path)
                        .context("Tried to save updated local filesystem metadata")?;
                }
                Ok(metadata)
            }
            None => Self::_generate(&metadata_file_path, encryption_key)
                .context("Tried to create local filesystem metadata"),
        }
    }

    fn _load(metadata_file_path: &Path) -> Result<Option<Self>> {
        if !metadata_file_path.exists() {
            // State file doesn't exist
            return Ok(None);
        }
        let file = std::fs::File::open(&metadata_file_path)?;
        Ok(Some(
            serde_json::from_reader(BufReader::new(file))
                .context("Trying to deserialize filesystem metadata")?,
        ))
    }

    fn _generate(metadata_file_path: &Path, encryption_key: &EncryptionKey) -> Result<Self> {
        let my_client_id = ClientId::generate_random();
        let encryption_key_hash = hash(encryption_key.as_bytes(), Salt::generate_random());
        let metadata = Self {
            my_client_id,
            encryption_key: encryption_key_hash,
        };
        metadata
            ._save(&metadata_file_path)
            .context("Trying to save filesystem metadata")?;
        Ok(metadata)
    }

    fn _save(&self, metadata_file_path: &Path) -> Result<()> {
        let file = std::fs::File::create(&metadata_file_path)?;
        serde_json::to_writer_pretty(BufWriter::new(file), self)?;
        Ok(())
    }

    pub fn my_client_id(&self) -> &ClientId {
        &self.my_client_id
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SerializedHash {
    #[serde(rename = "hash")]
    digest: String,

    #[serde(rename = "salt")]
    salt: String,
}

impl From<Hash> for SerializedHash {
    fn from(hash: Hash) -> Self {
        Self {
            digest: hash.digest.to_hex(),
            salt: hash.salt.to_hex(),
        }
    }
}

impl TryFrom<SerializedHash> for Hash {
    type Error = anyhow::Error;

    fn try_from(hashed_key: SerializedHash) -> Result<Self> {
        Ok(Self {
            digest: Digest::from_hex(&hashed_key.digest)?,
            salt: Salt::from_hex(&hashed_key.salt)?,
        })
    }
}

mod serialize_client_id {
    use cryfs_blockstore::ClientId;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(client_id: &ClientId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&client_id.id.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<ClientId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        Ok(ClientId {
            id: id.parse().map_err(serde::de::Error::custom)?,
        })
    }
}

// TODO Tests
