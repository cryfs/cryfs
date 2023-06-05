use thiserror::Error;

// TODO Add fh parameters for descriptor errors and path parameters to others

// TODO Is there a better way for error reporting, e.g. having custom error types for each interface function and mapping them to system error codes in the fuse_mt backend adapter?

#[derive(Error, Clone, Debug)]
pub enum FsError {
    // TODO We should probably get rid of Custom and instead use more specific error types, or at least minimize its use
    #[error("Error code: {error_code}")]
    Custom { error_code: libc::c_int },

    #[error("Not implemented")]
    NotImplemented,

    // TODO Remove UnknownError and do better reporting for those cases
    #[error("Unknown Error")]
    UnknownError,

    #[error("There is an error in the file system data. Maybe it is corrupted. {message}")]
    CorruptedFilesystem { message: String },

    #[error("The file descriptor {fh} does not represent an open file")]
    InvalidFileDescriptor { fh: u64 },

    #[error("The file descriptor represents a file that is open for writing, but the file is not open for reading")]
    ReadOnWriteOnlyFileDescriptor,

    #[error("The file descriptor represents a file that is open for reading, but the file is not open for writing")]
    WriteOnReadOnlyFileDescriptor,

    #[error("Tried to create a file system node that already exists")]
    NodeAlreadyExists,

    #[error("Tried to access a file system node that does not exist")]
    NodeDoesNotExist,

    #[error("The file system node is not a directory")]
    NodeIsNotADirectory,

    #[error("The file system node is a directory")]
    NodeIsADirectory,

    #[error("The file system node is not a symlink")]
    NodeIsNotASymlink,

    #[error("The path is invalid")]
    InvalidPath,

    #[error("The operation is invalid")]
    InvalidOperation,
}

impl FsError {
    pub fn system_error_code(&self) -> libc::c_int {
        match self {
            FsError::Custom { error_code } => *error_code,
            FsError::NotImplemented => libc::ENOSYS,
            FsError::InvalidFileDescriptor { .. } => libc::EBADF,
            FsError::ReadOnWriteOnlyFileDescriptor => libc::EBADF,
            FsError::WriteOnReadOnlyFileDescriptor => libc::EBADF,
            FsError::NodeAlreadyExists { .. } => libc::EEXIST,
            FsError::NodeDoesNotExist { .. } => libc::ENOENT,
            FsError::NodeIsNotADirectory { .. } => libc::ENOTDIR,
            FsError::NodeIsADirectory { .. } => libc::EISDIR,
            FsError::NodeIsNotASymlink => libc::EINVAL,
            FsError::InvalidPath => libc::EINVAL,
            FsError::InvalidOperation => libc::EINVAL,
            FsError::UnknownError => libc::EIO,
            FsError::CorruptedFilesystem { .. } => libc::EIO,
        }
    }
}

pub type FsResult<T> = Result<T, FsError>;
