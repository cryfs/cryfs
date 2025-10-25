use async_trait::async_trait;
use std::fmt::Debug;
use std::hash::Hash;

use super::{HandlePool, HandleWithGeneration};
use crate::FsError;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropHashMap};

/// A [HandleMap] stores objects keyed by a unique handle. You can add
/// new objects to the map using [Self::insert], which will return the
/// handle of the new entry for you.
#[derive(Debug)]
pub struct HandleMap<Handle, T>
where
    // TODO Instead of From<u64> + Into<u64>, `Step` would be better, but that's unstable.
    Handle: From<u64> + Into<u64> + Clone + Eq + Ord + Hash + Send + Debug,
    T: AsyncDrop<Error = FsError> + Send + Debug,
{
    available_handles: HandlePool<Handle>,

    // We use a hashmap instead of Vec so that space gets reused when an object gets removed, even before the handle gets reused.
    objects: AsyncDropGuard<AsyncDropHashMap<Handle, T>>,
}

impl<Handle, T> HandleMap<Handle, T>
where
    Handle: From<u64> + Into<u64> + Clone + Eq + Ord + Hash + Send + Debug,
    T: AsyncDrop<Error = FsError> + Send + Debug,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            available_handles: HandlePool::new(),
            objects: AsyncDropHashMap::new(),
        })
    }

    /// Blocks the given handle from being used for new entries.
    /// Panics if the handle is already used for an entry.
    pub fn block_handle(&mut self, handle: Handle) {
        self.available_handles.acquire_specific(handle);
    }

    pub fn add(&mut self, file: AsyncDropGuard<T>) -> HandleWithGeneration<Handle> {
        let handle = self.available_handles.acquire();
        self.objects
            .try_insert(handle.handle.clone(), file)
            .expect("Tried to add a file to the HandleMap but the handle was already in use");
        handle
    }

    pub fn remove(&mut self, handle: Handle) -> AsyncDropGuard<T> {
        let file = self
            .objects
            .remove(&handle)
            .expect("Tried to remove a file from the HandleMap but the object didn't exist");
        self.available_handles.release(handle);
        file
    }

    pub fn get(&self, handle: Handle) -> Option<&AsyncDropGuard<T>> {
        self.objects.get(&handle)
    }

    #[cfg(feature = "testutils")]
    pub fn drain(&mut self) -> impl Iterator<Item = (Handle, AsyncDropGuard<T>)> {
        self.available_handles = HandlePool::new();
        self.objects.drain()
    }
}

#[async_trait]
impl<Handle, T> AsyncDrop for HandleMap<Handle, T>
where
    Handle: From<u64> + Into<u64> + Clone + Eq + Ord + Hash + Send + Debug,
    T: AsyncDrop<Error = FsError> + Send + Debug,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.objects.async_drop().await
    }
}

// TODO Tests
