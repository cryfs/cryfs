use nix::{sys::stat::Mode, unistd, Result};
use std::path::{Path, PathBuf};

pub struct FilesystemDriver {
    path: PathBuf,
}

impl FilesystemDriver {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn mkdir<'a>(&self, path: &Path, mode: Mode) -> Result<()> {
        unistd::mkdir(&self.path.join(path), mode)
    }
}
