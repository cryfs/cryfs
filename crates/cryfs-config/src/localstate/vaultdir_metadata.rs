use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::config::FilesystemId;
use crate::localstate::local_state_dir::LocalStateDir;

// TODO Right now, VaultdirMetadata is based on the vaultdir location and is checked in cryfs-cli.
//      Instead, we should probably make it based on the config file location and check it in cryfs-filesystem.

/// Store the list of all vaultdirs and their filesystem ids
/// so we can recognize if a filesystem gets replaced with
/// a different filesystem by an adversary
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultdirMetadata {
    #[serde(flatten)]
    vaultdirs: HashMap<PathBuf, VaultdirMetadataEntry>,
}

impl VaultdirMetadata {
    pub fn load(local_state_dir: &LocalStateDir) -> Result<Self> {
        let vaultdirs_file = local_state_dir.for_vaultdir_metadata()?;
        let result = if vaultdirs_file.exists() {
            let file = std::fs::File::open(&vaultdirs_file)?;
            serde_json::from_reader(BufReader::new(file))?
        } else {
            Self::default()
        };
        Ok(result)
    }

    pub fn filesystem_id_for_vaultdir_is_correct(
        &self,
        vaultdir: &Path,
        expected_filesystem_id: &FilesystemId,
    ) -> Result<(), CheckFilesystemIdError> {
        match self.vaultdirs.get(vaultdir) {
            None => {
                // Vaultdir not known yet, everything is fine
                Ok(())
            }
            Some(entry) => {
                if entry.filesystem_id == *expected_filesystem_id {
                    Ok(())
                } else {
                    Err(CheckFilesystemIdError::FilesystemIdIncorrect {
                        vaultdir: vaultdir.to_path_buf(),
                        expected_id: *expected_filesystem_id,
                        actual_id: entry.filesystem_id,
                    })
                }
            }
        }
    }

    pub fn update_filesystem_id_for_vaultdir(
        &mut self,
        vaultdir: &Path,
        filesystem_id: FilesystemId,
        local_state_dir: &LocalStateDir,
    ) -> Result<()> {
        let new_entry = VaultdirMetadataEntry { filesystem_id };
        match self.vaultdirs.entry(vaultdir.to_path_buf()) {
            Entry::Occupied(mut entry) => {
                if *entry.get() == new_entry {
                    // Filesystem id is already correct, nothing to do
                    Ok(())
                } else {
                    // Filesystem id is incorrect, update it
                    entry.insert(new_entry);
                    self.save(local_state_dir)
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(new_entry);
                self.save(local_state_dir)
            }
        }
    }

    fn save(&self, local_state_dir: &LocalStateDir) -> Result<()> {
        let vaultdirs_file = local_state_dir.for_vaultdir_metadata()?;
        let file = std::fs::File::create(&vaultdirs_file)?;
        serde_json::to_writer_pretty(BufWriter::new(file), self)?;
        Ok(())
    }
}

impl Default for VaultdirMetadata {
    fn default() -> Self {
        Self {
            vaultdirs: HashMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CheckFilesystemIdError {
    #[error(
        "Filesystem id for vault directory {vaultdir} is incorrect. Expected {expected_id:?} but got {actual_id:?}. This likely means that the filesystem that was previously at this location was replaced with a different filesystem. CryFS prevents this to avoid malicious actors from replacing a file system without you noticing."
    )]
    FilesystemIdIncorrect {
        vaultdir: PathBuf,
        expected_id: FilesystemId,
        actual_id: FilesystemId,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct VaultdirMetadataEntry {
    #[serde(rename = "filesystemId", with = "serialize_filesystem_id")]
    filesystem_id: FilesystemId,
}

mod serialize_filesystem_id {
    use serde::{Deserialize, Deserializer, Serializer};

    use crate::config::FilesystemId;

    pub fn serialize<S>(filesystem_id: &FilesystemId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&filesystem_id.to_hex())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<FilesystemId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex = String::deserialize(deserializer)?;
        FilesystemId::from_hex(&hex).map_err(serde::de::Error::custom)
    }
}

// TODO Tests
