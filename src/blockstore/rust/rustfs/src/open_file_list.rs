use derive_more::{From, Into};
use std::collections::HashMap;

use crate::interface::OpenFile;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, From, Into)]
pub struct OpenFileHandle(u64);

struct HandlePool {
    // Handles that were used previously but then returned and are now free to be reused
    released_handles: Vec<OpenFileHandle>,

    // The next handle to be used
    next_handle: OpenFileHandle,
}

impl HandlePool {
    fn new() -> Self {
        Self {
            released_handles: Vec::new(),
            next_handle: OpenFileHandle(0),
        }
    }

    fn acquire(&mut self) -> OpenFileHandle {
        if let Some(handle) = self.released_handles.pop() {
            handle
        } else {
            let handle = self.next_handle;
            self.next_handle.0 += 1;
            handle
        }
    }

    fn release(&mut self, handle: OpenFileHandle) {
        self.released_handles.push(handle);
    }
}

pub struct OpenFileList<OF: OpenFile> {
    // We use a hashset instead of Vec so that space gets freed when a file gets closed.
    open_files: HashMap<OpenFileHandle, OF>,

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
    pub fn add(&mut self, file: OF) -> OpenFileHandle {
        let handle = self.available_handles.acquire();
        self.open_files.insert(handle, file);
        handle
    }

    pub fn remove(&mut self, handle: OpenFileHandle) -> OF {
        let file = self
            .open_files
            .remove(&handle)
            .expect("Tried to remove a file from the open file list but the handle didn't represent an open file");
        self.available_handles.release(handle);
        file
    }

    pub fn get(&self, handle: OpenFileHandle) -> Option<&OF> {
        self.open_files.get(&handle)
    }
}
