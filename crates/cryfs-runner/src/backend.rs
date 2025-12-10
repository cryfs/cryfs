// If fuse_mt is disabled and fuser is enabled, use fuser as the backend
// This is the default configuration because the `fuser` feature is enabled by default but `fuse_mt` is not
#[cfg(all(not(feature = "fuse_mt"), feature = "fuser"))]
pub type Backend = cryfs_rustfs::object_based_api::RustfsFuserBackend;

// If the fuse_mt feature is enabled, use fuse_mt as the backend
// This is non-default, and can be enabled by enabling the `fuse_mt` feature
#[cfg(feature = "fuse_mt")]
pub type Backend = cryfs_rustfs::object_based_api::RustfsFusemtBackend;
