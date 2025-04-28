mod interface;
pub use interface::Block;

mod implementations;
pub use implementations::{LockingBlock, LockingBlockStore};
