mod interface;
pub use interface::{Block, BlockStore};

mod implementations;
pub use implementations::{LockingBlock, LockingBlockStore, SharedBlockStore};
