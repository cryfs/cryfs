use std::path::PathBuf;

pub struct FilesystemDriver {
    path: PathBuf,
}

impl FilesystemDriver {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}
