use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use super::interface::OpenFile;
use crate::common::FsError;
use crate::low_level_api::FileHandle;
use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, AsyncDropHashMap};

#[derive(Debug)]
struct HandlePool {
    // Handles that were used previously but then returned and are now free to be reused
    released_handles: Vec<FileHandle>,

    // The next handle to be used
    next_handle: FileHandle,
}

impl HandlePool {
    fn new() -> Self {
        Self {
            released_handles: Vec::new(),
            next_handle: FileHandle(0),
        }
    }

    fn acquire(&mut self) -> FileHandle {
        if let Some(handle) = self.released_handles.pop() {
            handle
        } else {
            let handle = self.next_handle;
            self.next_handle.0 += 1;
            handle
        }
    }

    fn release(&mut self, handle: FileHandle) {
        self.released_handles.push(handle);
    }
}

#[derive(Debug)]
pub struct OpenFileList<OF>
where
    OF: OpenFile + AsyncDrop<Error = FsError> + Send,
{
    // We use a hashset instead of Vec so that space gets freed when a file gets closed.
    open_files: AsyncDropGuard<AsyncDropHashMap<FileHandle, OF>>,

    available_handles: HandlePool,
}

impl<OF> OpenFileList<OF>
where
    OF: OpenFile + AsyncDrop<Error = FsError> + Send,
{
    pub fn new() -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            open_files: AsyncDropHashMap::new(),
            available_handles: HandlePool::new(),
        })
    }

    pub fn add(&mut self, file: AsyncDropGuard<OF>) -> FileHandle {
        let handle = self.available_handles.acquire();
        self.open_files
            .try_insert(handle, file)
            .expect("Tried to add a file to the open file list but the handle was already in use");
        handle
    }

    pub fn remove(&mut self, handle: FileHandle) -> AsyncDropGuard<OF> {
        let file = self
            .open_files
            .remove(&handle)
            .expect("Tried to remove a file from the open file list but the handle didn't represent an open file");
        self.available_handles.release(handle);
        file
    }

    pub fn get(&self, handle: FileHandle) -> Option<&OF> {
        self.open_files.get(&handle)
    }
}

// TODO Tests

#[async_trait]
impl<OF> AsyncDrop for OpenFileList<OF>
where
    OF: OpenFile + AsyncDrop<Error = FsError> + Send,
{
    type Error = FsError;

    async fn async_drop_impl(&mut self) -> Result<(), FsError> {
        self.open_files.async_drop().await
    }
}
