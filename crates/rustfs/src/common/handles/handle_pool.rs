use derive_more::{From, Into};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, From, Into)]
pub struct FileHandle(pub u64);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FileHandleWithGeneration {
    pub handle: FileHandle,
    pub generation: u64,
}

/// A [HandlePool] can be used to acquire and release unique handles, e.g. inodes.
/// Each handle is a unique number and [Self::acquire] will return a different number
/// each time. [Self::release] can be used to return a handle to the pool, after which
/// [Self::acquire] may return the same handle again.
#[derive(Debug)]
pub struct HandlePool {
    /// Handles that are currently in use, mapping to their current generation.
    in_use_handles: HashMap<FileHandle, u64>,

    /// Handles that were used previously but then returned and are now free to be reused
    /// The generation number is the number it was last used with.
    released_handles: Vec<FileHandleWithGeneration>,

    /// The lowest handle that hasn't been used yet
    next_handle: FileHandle,
}

impl HandlePool {
    pub fn new() -> Self {
        Self {
            in_use_handles: HashMap::new(),
            released_handles: Vec::new(),
            next_handle: FileHandle(0),
        }
    }

    pub fn acquire(&mut self) -> FileHandleWithGeneration {
        if let Some(FileHandleWithGeneration {
            handle,
            generation: last_used_generation,
        }) = self.released_handles.pop()
        {
            assert!(last_used_generation < u64::MAX);
            self._acquire(handle, last_used_generation + 1)
        } else {
            let handle = self.next_handle;
            assert!(self.next_handle.0 < u64::MAX);
            self.next_handle.0 += 1;
            self._acquire(handle, 0)
        }
    }

    /// Acquires a handle with a given value. If the handle is already acquired, this will panic.
    pub fn acquire_specific(&mut self, handle: FileHandle) -> FileHandleWithGeneration {
        if handle.0 >= self.next_handle.0 {
            let inbetween_handles = (self.next_handle.0..handle.0).map(FileHandle);
            self.released_handles
                .extend(inbetween_handles.map(|handle| FileHandleWithGeneration {
                    handle,
                    generation: 0,
                }));
            self.next_handle = FileHandle(handle.0 + 1);
            self._acquire(handle, 0)
        } else {
            if let Some(pos_in_released_handles) = self
                .released_handles
                .iter()
                .position(|h| h.handle == handle)
            {
                let released_handle = self.released_handles.swap_remove(pos_in_released_handles);
                assert_eq!(handle, released_handle.handle);
                self._acquire(handle, released_handle.generation + 1)
            } else {
                panic!("Tried to acquire a specific handle but it was already acquired");
            }
        }
    }

    fn _acquire(&mut self, handle: FileHandle, generation: u64) -> FileHandleWithGeneration {
        self.in_use_handles.insert(handle, generation);
        FileHandleWithGeneration { handle, generation }
    }

    pub fn release(&mut self, handle: FileHandle) {
        let generation = self
            .in_use_handles
            .remove(&handle)
            .expect("Tried to release a handle that wasn't in use");
        self.released_handles
            .push(FileHandleWithGeneration { handle, generation });
    }
}

// TODO Test
