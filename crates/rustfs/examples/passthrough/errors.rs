use cryfs_rustfs::{FsError, FsResult};

pub trait IoResultExt<T> {
    fn map_error(self) -> FsResult<T>;
}
impl<T> IoResultExt<T> for std::io::Result<T> {
    fn map_error(self) -> FsResult<T> {
        self.map_err(|err| match err.raw_os_error() {
            Some(error_code) => FsError::Custom { error_code },
            None => FsError::UnknownError,
        })
    }
}

pub trait NixResultExt<T> {
    fn map_error(self) -> FsResult<T>;
}
impl<T> NixResultExt<T> for nix::Result<T> {
    fn map_error(self) -> FsResult<T> {
        self.map_err(|errno| {
            let error = std::io::Error::from(errno);
            match error.raw_os_error() {
                Some(error_code) => FsError::Custom { error_code },
                None => FsError::UnknownError,
            }
        })
    }
}
