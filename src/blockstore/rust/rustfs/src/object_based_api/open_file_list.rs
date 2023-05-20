use std::collections::HashMap;

use super::interface::OpenFile;
use crate::low_level_api::FileHandle;

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

pub struct OpenFileList<OF: OpenFile> {
    // We use a hashset instead of Vec so that space gets freed when a file gets closed.
    open_files: HashMap<FileHandle, OF>,

    available_handles: HandlePool,
}

impl<OF: OpenFile> Default for OpenFileList<OF> {
    fn default() -> Self {
        Self {
            open_files: HashMap::new(),
            available_handles: HandlePool::new(),
        }
    }
}

impl<OF: OpenFile> OpenFileList<OF> {
    pub fn add(&mut self, file: OF) -> FileHandle {
        let handle = self.available_handles.acquire();
        self.open_files.insert(handle, file);
        handle
    }

    pub fn remove(&mut self, handle: FileHandle) -> OF {
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
