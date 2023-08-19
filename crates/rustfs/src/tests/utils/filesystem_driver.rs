use nix::{unistd, Result};
use std::path::PathBuf;

use crate::{AbsolutePathBuf, Mode};

pub struct FilesystemDriver {
    path: AbsolutePathBuf,
}

impl FilesystemDriver {
    pub fn new(path: AbsolutePathBuf) -> Self {
        Self { path }
    }

    fn _path<'a>(&self, path: &str) -> PathBuf {
        self.path
            .clone()
            .push_all(path.try_into().unwrap())
            .as_ref()
            .to_owned()
    }

    pub async fn mkdir<'a>(&self, path: &str, mode: Mode) -> Result<()> {
        let path = self._path(path);
        let mode = nix::sys::stat::Mode::from_bits(mode.into()).unwrap();
        tokio::task::spawn_blocking(move || unistd::mkdir(&path, mode))
            .await
            .unwrap()
    }
}
