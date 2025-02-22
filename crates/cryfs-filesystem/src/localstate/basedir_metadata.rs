use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::config::FilesystemId;
use crate::localstate::local_state_dir::LocalStateDir;

// TODO Right now, BasedirMetadata is based on the basedir location and is checked in cryfs-cli.
//      Instead, we should probably make it based on the config file location and check it in cryfs-filesystem.

/// Store the list of all basedirs and their filesystem ids
/// so we can recognize if a filesystem gets replaced with
/// a different filesystem by an adversary
#[derive(Debug, Serialize, Deserialize)]
pub struct BasedirMetadata {
    #[serde(flatten)]
    basedirs: HashMap<PathBuf, BasedirMetadataEntry>,
}

impl BasedirMetadata {
    pub fn load(local_state_dir: &LocalStateDir) -> Result<Self> {
        let basedirs_file = local_state_dir.for_basedir_metadata()?;
        let result = if basedirs_file.exists() {
            let file = std::fs::File::open(&basedirs_file)?;
            serde_json::from_reader(BufReader::new(file))?
        } else {
            Self::default()
        };
        Ok(result)
    }

    pub fn filesystem_id_for_basedir_is_correct(
        &self,
        basedir: &Path,
        expected_filesystem_id: &FilesystemId,
    ) -> Result<(), CheckFilesystemIdError> {
        match self.basedirs.get(basedir) {
            None => {
                // Basedir not known yet, everything is fine
                Ok(())
            }
            Some(entry) => {
                if entry.filesystem_id == *expected_filesystem_id {
                    Ok(())
                } else {
                    Err(CheckFilesystemIdError::FilesystemIdIncorrect {
                        basedir: basedir.to_path_buf(),
                        expected_id: *expected_filesystem_id,
                        actual_id: entry.filesystem_id,
                    })
                }
            }
        }
    }

    pub fn update_filesystem_id_for_basedir(
        &mut self,
        basedir: &Path,
        filesystem_id: FilesystemId,
        local_state_dir: &LocalStateDir,
    ) -> Result<()> {
        let new_entry = BasedirMetadataEntry { filesystem_id };
        match self.basedirs.entry(basedir.to_path_buf()) {
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
        let basedirs_file = local_state_dir.for_basedir_metadata()?;
        let file = std::fs::File::create(&basedirs_file)?;
        serde_json::to_writer_pretty(BufWriter::new(file), self)?;
        Ok(())
    }
}

impl Default for BasedirMetadata {
    fn default() -> Self {
        Self {
            basedirs: HashMap::new(),
        }
    }
}

#[derive(Debug, Error)]
pub enum CheckFilesystemIdError {
    #[error(
        "Filesystem id for basedir {basedir} is incorrect. Expected {expected_id:?} but got {actual_id:?}. This likely means that the filesystem that was previously at this location was replaced with a different filesystem. CryFS prevents this to avoid malicious actors from replacing a file system without you noticing."
    )]
    FilesystemIdIncorrect {
        basedir: PathBuf,
        expected_id: FilesystemId,
        actual_id: FilesystemId,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct BasedirMetadataEntry {
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
