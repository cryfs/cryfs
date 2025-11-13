use async_trait::async_trait;
use std::fmt::Debug;

use super::{HandlePool, HandleWithGeneration};
use crate::{FsError, common::handles::handle_trait::HandleTrait};
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropHashMap};

/// A [HandleMap] stores objects keyed by a unique handle. You can add
/// new objects to the map using [Self::insert], which will return the
/// handle of the new entry for you.
#[derive(Debug)]
pub struct HandleMap<Handle, T>
where
    Handle: HandleTrait + Send,
    T: AsyncDrop<Error = FsError> + Send + Debug,
{
    available_handles: HandlePool<Handle>,

    // We use a hashmap instead of Vec so that space gets reused when an object gets removed, even before the handle gets reused.
    // TODO It might actually be faster to use a `Vec<Handle, Option<T>>` here and just set entries to None when they get removed.
    //      Then we could also store the generation number right in this struct at each entry, instead of having HandlePool manage it.
    //      This is also what fuse-mt does. Or maybe, even better, use the slab crate?
    objects: AsyncDropGuard<AsyncDropHashMap<Handle, T>>,
}

impl<Handle, T> HandleMap<Handle, T>
where
    Handle: HandleTrait + Send,
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

    pub fn insert(&mut self, handle: Handle, file: AsyncDropGuard<T>) {
        self.available_handles.acquire_specific(handle.clone());
        self.objects
            .try_insert(handle, file)
            .expect("Tried to insert a file to the HandleMap but the handle was already in use");
    }

    pub fn remove(&mut self, handle: Handle) -> AsyncDropGuard<T> {
        let file = self
            .objects
            .remove(&handle)
            .expect("Tried to remove a file from the HandleMap but the object didn't exist");
        self.available_handles.release(handle);
        file
    }

    pub fn try_remove(&mut self, handle: Handle) -> Option<AsyncDropGuard<T>> {
        let Some(file) = self.objects.remove(&handle) else {
            return None;
        };
        self.available_handles.release(handle);
        Some(file)
    }

    pub fn get(&self, handle: Handle) -> Option<&AsyncDropGuard<T>> {
        self.objects.get(&handle)
    }

    pub fn get_mut(&mut self, handle: Handle) -> Option<&mut AsyncDropGuard<T>> {
        self.objects.get_mut(&handle)
    }

    #[cfg(feature = "testutils")]
    pub fn drain(&mut self) -> impl Iterator<Item = (Handle, AsyncDropGuard<T>)> {
        self.available_handles = HandlePool::new();
        self.objects.drain()
    }

    #[cfg(any(test, feature = "testutils"))]
    pub fn iter(&self) -> impl Iterator<Item = (&Handle, &AsyncDropGuard<T>)> {
        self.objects.iter()
    }
}

#[async_trait]
impl<Handle, T> AsyncDrop for HandleMap<Handle, T>
where
    Handle: HandleTrait + Send,
    T: AsyncDrop<Error = FsError> + Send + Debug,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.objects.async_drop().await
    }
}

// TODO Tests
