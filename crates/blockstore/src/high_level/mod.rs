mod interface;
pub use interface::{Block, BlockStore};

mod implementations;
pub use implementations::{LockingBlock, LockingBlockStore};

#[cfg(any(test, feature = "testutils"))]
pub use implementations::{ActionCounts, SharedBlockStore, TrackingBlockStore};
