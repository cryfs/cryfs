//! This module defines the filesystem interface.

mod device;
pub use device::Device;

mod node;
pub use node::{Node, NodeAttrs};

mod dir;
pub use dir::{Dir, DirEntry};

mod symlink;
pub use symlink::Symlink;

mod error;
pub use error::{FsError, FsResult};
