use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParsePathError {
    #[error("Path must be absolute")]
    NotAbsolute,

    #[error("Path component is empty")]
    EmptyComponent,

    #[error("Path is not UTF-8")]
    NotUtf8,

    #[error("Path is in an invalid format")]
    InvalidFormat,
}
