use lockable::{Lockable, LockableLruCache};
use std::fmt::{self, Debug};

use super::entry::BlockCacheEntry;
use crate::blockstore::BlockId;

pub struct BlockCacheEntryGuard<B>
where
    B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static,
{
    pub(super) guard: <LockableLruCache<BlockId, BlockCacheEntry<B>> as Lockable<
        BlockId,
        BlockCacheEntry<B>,
    >>::OwnedGuard,
}

impl<B> BlockCacheEntryGuard<B>
where
    B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static,
{
    pub fn key(&self) -> &BlockId {
        self.guard.key()
    }

    pub fn value(&self) -> Option<&BlockCacheEntry<B>> {
        self.guard.value()
    }

    pub fn value_mut(&mut self) -> Option<&mut BlockCacheEntry<B>> {
        self.guard.value_mut()
    }

    pub fn insert(&mut self, v: BlockCacheEntry<B>) -> Option<BlockCacheEntry<B>> {
        self.guard.insert(v)
    }
}

impl<B> Debug for BlockCacheEntryGuard<B>
where
    B: crate::blockstore::low_level::BlockStore + Send + Sync + Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockCacheEntryGuard({:?})", self.guard)
    }
}
