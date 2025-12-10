use std::collections::HashMap;
use std::fmt::Debug;

use crate::common::HandleWithGeneration;
use crate::common::handles::handle_trait::HandleTrait;

/// A [HandlePool] can be used to acquire and release unique handles, e.g. inodes.
/// Each handle is a unique number and [Self::acquire] will return a different number
/// each time. [Self::release] can be used to return a handle to the pool, after which
/// [Self::acquire] may return the same handle again.
#[derive(Debug)]
pub struct HandlePool<Handle>
where
    Handle: HandleTrait,
{
    /// Handles that are currently in use, mapping to their current generation.
    in_use_handles: HashMap<Handle, u64>,

    /// Handles that were used previously but then returned and are now free to be reused
    /// The generation number is the number it can be used next with
    released_handles: Vec<HandleWithGeneration<Handle>>,

    /// The lowest handle that hasn't been used yet
    next_handle: Handle,
}

impl<Handle> HandlePool<Handle>
where
    Handle: HandleTrait,
{
    pub fn new() -> Self {
        Self {
            in_use_handles: HashMap::new(),
            released_handles: Vec::new(),
            next_handle: Handle::MIN,
        }
    }

    pub fn acquire(&mut self) -> HandleWithGeneration<Handle> {
        match self.released_handles.pop() {
            Some(HandleWithGeneration { handle, generation }) => self._acquire(handle, generation),
            _ => {
                let handle = self.next_handle.clone();
                assert!(self.next_handle < Handle::MAX);
                self.next_handle = self.next_handle.incremented();
                self._acquire(handle, 0)
            }
        }
    }

    /// Acquires a handle with a given value. If the handle is already acquired, this will panic.
    pub fn acquire_specific(&mut self, handle: Handle) -> HandleWithGeneration<Handle> {
        self.try_acquire_specific(handle)
            .expect("Tried to acquire a specific handle but it was already acquired")
    }

    pub fn try_acquire_specific(&mut self, handle: Handle) -> Option<HandleWithGeneration<Handle>> {
        if handle >= self.next_handle {
            let inbetween_handles = Handle::range(&self.next_handle, &handle);
            self.released_handles
                .extend(inbetween_handles.map(|handle| HandleWithGeneration {
                    handle,
                    generation: 0,
                }));
            self.next_handle = handle.incremented();
            Some(self._acquire(handle, 0))
        } else {
            match self
                .released_handles
                .iter()
                .position(|h| h.handle == handle)
            {
                Some(pos_in_released_handles) => {
                    let released_handle =
                        self.released_handles.swap_remove(pos_in_released_handles);
                    assert_eq!(handle, released_handle.handle);
                    Some(self._acquire(handle, released_handle.generation))
                }
                _ => None,
            }
        }
    }

    fn _acquire(&mut self, handle: Handle, generation: u64) -> HandleWithGeneration<Handle> {
        self.in_use_handles.insert(handle.clone(), generation);
        HandleWithGeneration { handle, generation }
    }

    pub fn release(&mut self, handle: Handle) {
        let generation = self
            .in_use_handles
            .remove(&handle)
            .expect("Tried to release a handle that wasn't in use");
        assert!(generation < u64::MAX);
        self.released_handles.push(HandleWithGeneration {
            handle,
            generation: generation + 1,
        });
    }

    /// Releases a handle same as [Self::release], but doesn't increase the generation.
    pub fn undo_acquire(&mut self, handle: Handle) {
        let generation = self
            .in_use_handles
            .remove(&handle)
            .expect("Tried to undo_acquire a handle that wasn't in use");
        self.released_handles
            .push(HandleWithGeneration { handle, generation });
    }

    pub fn lookup(&self, handle: Handle) -> Option<HandleWithGeneration<Handle>> {
        self.in_use_handles
            .get(&handle)
            .map(|generation| HandleWithGeneration {
                handle,
                generation: *generation,
            })
    }
}

// TODO Test
