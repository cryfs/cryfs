use thiserror::Error;

// TODO Is there a better way for error reporting, e.g. having custom error types for each interface function and mapping them to system error codes in the fuse_mt backend adapter?
#[derive(Error, Clone, Copy, Debug)]
pub enum FsError {
    // TODO We should probably get rid of Custom and instead use more specific error types, or at least minimize its use
    #[error("Error code: {error_code}")]
    Custom { error_code: libc::c_int },

    #[error("Unknown Error")]
    UnknownError,

    #[error("The file descriptor {fh} does not represent an open file")]
    InvalidFileDescriptor { fh: u64 },
}

impl FsError {
    pub fn system_error_code(self) -> libc::c_int {
        match self {
            FsError::Custom { error_code } => error_code,
            FsError::InvalidFileDescriptor { .. } => libc::EBADF,
            FsError::UnknownError => libc::EIO,
        }
    }
}

pub type FsResult<T> = Result<T, FsError>;
