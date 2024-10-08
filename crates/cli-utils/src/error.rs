use derive_more::Display;
use std::{error::Error, process::ExitCode};

// Don't derive `Error` for `CliError` because we don't want a thrown CliError to be converted back to `anyhow::Error`, that might swallow the exit code.
#[derive(Display, Debug)]
#[display("{error}")]
pub struct CliError {
    pub kind: CliErrorKind,
    pub error: anyhow::Error,
}

/// An extension trait to map a Result with an anyhow::Error error type into a Result with a `CliError` error type.
pub trait CliResultExt {
    type T;
    fn map_cli_error(self, kind: CliErrorKind) -> Result<Self::T, CliError>;
}
impl<T> CliResultExt for Result<T, anyhow::Error> {
    type T = T;
    fn map_cli_error(self, kind: CliErrorKind) -> Result<T, CliError> {
        self.map_err(|error| CliError { kind, error })
    }
}

/// An extension trait to map a Result with an arbitrary `impl Error` type into a Result with a `CliError` error type.
pub trait CliResultExtFn {
    type T;
    type E;
    fn map_cli_error(
        self,
        kind: impl FnOnce(&Self::E) -> CliErrorKind,
    ) -> Result<Self::T, CliError>;
}
impl<T, E> CliResultExtFn for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    type T = T;
    type E = E;
    fn map_cli_error(self, kind: impl FnOnce(&E) -> CliErrorKind) -> Result<T, CliError> {
        self.map_err(|error| CliError {
            kind: kind(&error),
            error: anyhow::Error::from(error),
        })
    }
}

// TODO Ensure that all of these errors, where C++ throws them, we throw them too and return them correctly to the shell. Grep our C++ code for when they're thrown.
// TODO Test error scenarios actually return the correct exit code to the shell
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CliErrorKind {
    /// No error happened, everything is ok
    Success,

    /// An error happened that doesn't have an error code associated with it
    UnspecifiedError,

    /// The command line arguments are invalid.
    InvalidArguments,

    /// Couldn't load config file. Either the password is wrong or the config file is corrupted.
    WrongPasswordOrCorruptedConfigFile,

    /// Password cannot be empty
    EmptyPassword,

    /// The file system format is too new for this CryFS version. Please update your CryFS version.
    TooNewFilesystemFormat,

    /// The file system format is too old for this CryFS version. Run with --allow-filesystem-upgrade to upgrade it.
    TooOldFilesystemFormat,

    /// The file system uses a different cipher than the one specified on the command line using the --cipher argument.
    WrongCipher,

    /// Base directory doesn't exist or is inaccessible (i.e. not read or writable or not a directory)
    InaccessibleBaseDir,

    /// Mount directory doesn't exist or is inaccessible (i.e. not read or writable or not a directory)
    InaccessibleMountDir,

    /// Local state directory doesn't exist or is inaccessible (i.e. not read or writable or not a directory)
    InaccessibleLocalStateDir,

    /// Base directory can't be a subdirectory of the mount directory
    BaseDirInsideMountDir,

    /// Something's wrong with the file system.
    InvalidFilesystem,

    /// The filesystem id in the config file is different to the last time we loaded a filesystem from this basedir. This could mean an attacker replaced the file system with a different one. You can pass the --allow-replaced-filesystem option to allow this.
    FilesystemIdChanged,

    /// The filesystem encryption key differs from the last time we loaded this filesystem. This could mean an attacker replaced the file system with a different one. You can pass the --allow-replaced-filesystem option to allow this.
    EncryptionKeyChanged,

    /// The command line options and the file system disagree on whether missing blocks should be treated as integrity violations.
    FilesystemHasDifferentIntegritySetup,

    /// File system is in single-client mode and can only be used from the client that created it.
    SingleClientFileSystem,

    /// A previous run of the file system detected an integrity violation. Preventing access to make sure the user notices. The file system will be accessible again after the user deletes the integrity state file.
    IntegrityViolationOnPreviousRun,

    /// An integrity violation was detected and the file system unmounted to make sure the user notices.
    IntegrityViolation,
}

impl CliErrorKind {
    /// Exit code to report to the shell
    pub fn exit_code(&self) -> ExitCode {
        ExitCode::from(match self {
            Self::Success => 0,
            Self::UnspecifiedError => 1,
            Self::InvalidArguments => 10,
            Self::WrongPasswordOrCorruptedConfigFile => 11,
            Self::EmptyPassword => 12,
            Self::TooNewFilesystemFormat => 13,
            Self::TooOldFilesystemFormat => 14,
            Self::WrongCipher => 15,
            Self::InaccessibleBaseDir => 16,
            Self::InaccessibleMountDir => 17,
            Self::BaseDirInsideMountDir => 18,
            Self::InvalidFilesystem => 19,
            Self::FilesystemIdChanged => 20,
            Self::EncryptionKeyChanged => 21,
            Self::FilesystemHasDifferentIntegritySetup => 22,
            Self::SingleClientFileSystem => 23,
            Self::IntegrityViolationOnPreviousRun => 24,
            Self::IntegrityViolation => 25,
            Self::InaccessibleLocalStateDir => 26,
        })
    }
}
