//! This module defines the filesystem interface.

mod device;
pub use device::{Device, Statfs};

mod node;
pub use node::{Node, NodeAttrs};

mod dir;
pub use dir::{Dir, DirEntry};

mod symlink;
pub use symlink::Symlink;

mod file;
pub use file::File;

mod open_file;
pub use open_file::OpenFile;

mod error;
pub use error::{FsError, FsResult};
