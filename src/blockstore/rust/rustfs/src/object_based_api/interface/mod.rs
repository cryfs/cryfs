//! This module defines the filesystem interface.

mod device;
pub use device::Device;

mod node;
pub use node::Node;

mod dir;
pub use dir::Dir;

mod symlink;
pub use symlink::Symlink;

mod file;
pub use file::File;

mod open_file;
pub use open_file::OpenFile;
