// TODO This should probably live in fspp, not here

use thiserror::Error;

// TODO Should we have a more fine grained error structure that allows each operation to define their own, more specific errors?
//      That would also allow us to handle errors better. There are likely instances where we want to hard-error if looking up
//      an entry fails because we're not looking up the main entry the operation is about but something else that is supposed
//      to be guaranteed to be there unless the file system is corrupt.

#[derive(Error, Debug)]
pub enum FsError {
    #[error("ENOENT: {msg}")]
    ENOENT { msg: String },

    #[error("EISDIR: {msg}")]
    EISDIR { msg: String },

    #[error("ENOTDIR: {msg}")]
    ENOTDIR { msg: String },

    #[error("EEXIST: {msg}")]
    EEXIST { msg: String },
}
