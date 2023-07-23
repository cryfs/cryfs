use async_trait::async_trait;
use std::fmt::Debug;

use crate::common::{FileHandle, FileHandleWithGeneration, HandlePool};
use crate::FsError;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropHashMap};

/// A [HandleMap] stores objects keyed by a unique handle. You can add
/// new objects to the map using [Self::insert], which will return the
/// handle of the new entry for you.
#[derive(Debug)]
pub struct HandleMap<T>
where
    T: AsyncDrop<Error = FsError> + Send + Debug,
{
    available_handles: HandlePool,

    // We use a hashmap instead of Vec so that space gets reused when an object gets removed, even before the handle gets reused.
    objects: AsyncDropGuard<AsyncDropHashMap<FileHandle, T>>,
}

impl<T> HandleMap<T>
where
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
    pub fn block_handle(&mut self, handle: FileHandle) {
        self.available_handles.acquire_specific(handle);
    }

    pub fn add(&mut self, file: AsyncDropGuard<T>) -> FileHandleWithGeneration {
        let handle = self.available_handles.acquire();
        self.objects
            .try_insert(handle.handle, file)
            .expect("Tried to add a file to the HandleMap but the handle was already in use");
        handle
    }

    pub fn remove(&mut self, handle: FileHandle) -> AsyncDropGuard<T> {
        let file = self
            .objects
            .remove(&handle)
            .expect("Tried to remove a file from the HandleMap but the object didn't exist");
        self.available_handles.release(handle);
        file
    }

    pub fn get(&self, handle: FileHandle) -> Option<&AsyncDropGuard<T>> {
        self.objects.get(&handle)
    }
}

#[async_trait]
impl<T> AsyncDrop for HandleMap<T>
where
    T: AsyncDrop<Error = FsError> + Send + Debug,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.objects.async_drop().await
    }
}

// TODO Tests
