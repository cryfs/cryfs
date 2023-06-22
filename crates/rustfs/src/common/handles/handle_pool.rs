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

    /// Acquires a handle with a given value. If the handle is already acquired, this will panic.
    pub fn acquire_specific(&mut self, handle: FileHandle) {
        if handle.0 >= self.next_handle.0 {
            let inbetween_handles = (self.next_handle.0..handle.0).map(FileHandle);
            self.released_handles.extend(inbetween_handles);
            self.next_handle = FileHandle(handle.0 + 1);
        } else {
            if let Some(pos_in_released_handles) =
                self.released_handles.iter().position(|h| *h == handle)
            {
                self.released_handles.swap_remove(pos_in_released_handles);
            } else {
                panic!("Tried to acquire a specific handle but it was already acquired");
            }
        }
    }

    pub fn release(&mut self, handle: FileHandle) {
        self.released_handles.push(handle);
    }
}

// TODO Test
