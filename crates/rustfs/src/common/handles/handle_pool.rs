use derive_more::{From, Into};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, From, Into)]
pub struct FileHandle(pub u64);

/// A [HandlePool] can be used to acquire and release unique handles, e.g. inodes.
/// Each handle is a unique number and [Self::acquire] will return a different number
/// each time. [Self::release] can be used to return a handle to the pool, after which
/// [Self::acquire] may return the same handle again.
#[derive(Debug)]
pub struct HandlePool {
    // Handles that were used previously but then returned and are now free to be reused
    released_handles: Vec<FileHandle>,

    // The next handle to be used
    next_handle: FileHandle,
}

impl HandlePool {
    pub fn new() -> Self {
        Self {
            released_handles: Vec::new(),
            next_handle: FileHandle(0),
        }
    }

    pub fn acquire(&mut self) -> FileHandle {
        if let Some(handle) = self.released_handles.pop() {
            handle
        } else {
            let handle = self.next_handle;
            self.next_handle.0 += 1;
            handle
        }
    }

    pub fn release(&mut self, handle: FileHandle) {
        self.released_handles.push(handle);
    }
}

// TODO Test
