use nix::{Result, unistd};
use std::path::PathBuf;

use crate::{AbsolutePathBuf, Mode};

// TODO Unify this with [cryfs_e2e_tests:::FilesystemDriver]

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
        // TODO Why do we need from_bits_retain instead of just from_bits here? It seems to fail when the ISDIR bit is set otherwise.
        #[cfg(target_os = "macos")]
        let mode = nix::sys::stat::Mode::from_bits_retain(u16::try_from(u32::from(mode)).unwrap());
        #[cfg(not(target_os = "macos"))]
        let mode = nix::sys::stat::Mode::from_bits_retain(mode.into());
        tokio::task::spawn_blocking(move || unistd::mkdir(&path, mode))
            .await
            .unwrap()
    }
}
