use std::fmt::{self, Debug};
use std::ops::{DerefMut, Deref};

use super::lockable_cache::OwnedGuard;
use super::entry::BlockCacheEntry;
use crate::blockstore::BlockId;

pub struct BlockCacheEntryGuard<B> where B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static {
    pub(super) guard: OwnedGuard<BlockId, BlockCacheEntry<B>>,
}

impl <B> BlockCacheEntryGuard<B>where B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static {
    pub fn key(&self) -> &BlockId {
        self.guard.key()
    }
}

impl<B> Deref for BlockCacheEntryGuard<B>
where
    B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static
{
    type Target = Option<BlockCacheEntry<B>>;
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

impl<B> DerefMut for BlockCacheEntryGuard<B>
where
    B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.deref_mut()
    }
}

impl<B> Debug for BlockCacheEntryGuard<B>
where
    B: crate::blockstore::low_level::BlockStore + Send + Sync + 'static
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BlockCacheEntryGuard({:?})", self.guard)
    }
}
