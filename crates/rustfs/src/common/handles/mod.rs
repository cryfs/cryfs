#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
mod handle_pool;
#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
pub use handle_pool::HandlePool;

mod handle_with_generation;
pub use handle_with_generation::HandleWithGeneration;

#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
mod handle_map;
#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
pub use handle_map::HandleMap;

#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
mod handle_trait;
#[cfg(any(feature = "fuser", feature = "fuse_mt"))]
pub use handle_trait::HandleTrait;
