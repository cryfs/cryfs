use anyhow::{Context, Result};
use std::path::PathBuf;

use crate::config::FilesystemId;

/// This struct helps find the right locations in the local file system to store local state
#[derive(Clone, Debug)]
pub struct LocalStateDir {
    app_dir: PathBuf,
}

impl LocalStateDir {
    pub fn new(app_dir: PathBuf) -> Self {
        Self { app_dir }
    }

    /// Location for the local state and integrity data of a specific filesystem
    pub fn for_filesystem_id(&self, filesystem_id: &FilesystemId) -> Result<PathBuf> {
        let filesystems_dir = self.app_dir.join("filesystems");
        let this_filesystem_dir = filesystems_dir.join(filesystem_id.to_hex());
        std::fs::create_dir_all(&this_filesystem_dir)
            .context("Tried to create directories for the filesystem local state")?;
        Ok(this_filesystem_dir)
    }

    /// Location for a file that stores the list of all basedirs
    /// and their filesystem ids so we can recognize if a filesystem
    /// gets replaced with a different filesystem by an adversary
    pub fn for_basedir_metadata(&self) -> Result<PathBuf> {
        std::fs::create_dir_all(&self.app_dir)
            .context("Tried to create directories for the local cryfs state")?;
        let basedirs_file = self.app_dir.join("basedirs");

        Ok(basedirs_file)
    }
}

// TODO Tests
