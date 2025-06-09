mod interface;
pub use interface::FilesystemDriver;

#[cfg(not(feature = "benchmark"))]
mod fuser;
#[cfg(not(feature = "benchmark"))]
pub use fuser::{FuserFilesystemDriver, WithInodeCache, WithoutInodeCache};

#[cfg(not(feature = "benchmark"))]
mod fuse_mt;
#[cfg(not(feature = "benchmark"))]
pub use fuse_mt::FusemtFilesystemDriver;

#[cfg(feature = "benchmark")]
mod mounting;
#[cfg(feature = "benchmark")]
pub use mounting::{FusemtMountingFilesystemDriver, FuserMountingFilesystemDriver};
