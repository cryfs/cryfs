use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HandleWithGeneration<Handle> {
    pub handle: Handle,
    pub generation: u64,
}

/// A [HandlePool] can be used to acquire and release unique handles, e.g. inodes.
/// Each handle is a unique number and [Self::acquire] will return a different number
/// each time. [Self::release] can be used to return a handle to the pool, after which
/// [Self::acquire] may return the same handle again.
#[derive(Debug)]
pub struct HandlePool<Handle>
where
    // TODO Instead of From<u64> + Into<u64>, `Step` would be better, but that's unstable.
    Handle: From<u64> + Into<u64> + Clone + Eq + Ord + Hash + Debug,
{
    /// Handles that are currently in use, mapping to their current generation.
    in_use_handles: HashMap<Handle, u64>,

    /// Handles that were used previously but then returned and are now free to be reused
    /// The generation number is the number it was last used with.
    released_handles: Vec<HandleWithGeneration<Handle>>,

    /// The lowest handle that hasn't been used yet
    next_handle: Handle,
}

impl<Handle> HandlePool<Handle>
where
    Handle: From<u64> + Into<u64> + Clone + Eq + Ord + Hash + Debug,
{
    pub fn new() -> Self {
        Self {
            in_use_handles: HashMap::new(),
            released_handles: Vec::new(),
            next_handle: Handle::from(0),
        }
    }

    pub fn acquire(&mut self) -> HandleWithGeneration<Handle> {
        match self.released_handles.pop() {
            Some(HandleWithGeneration {
                handle,
                generation: last_used_generation,
            }) => {
                assert!(last_used_generation < u64::MAX);
                self._acquire(handle, last_used_generation + 1)
            }
            _ => {
                let handle = self.next_handle.clone();
                assert!(self.next_handle < Handle::from(u64::MAX));
                self.next_handle = Self::increment(self.next_handle.clone());
                self._acquire(handle, 0)
            }
        }
    }

    /// Acquires a handle with a given value. If the handle is already acquired, this will panic.
    pub fn acquire_specific(&mut self, handle: Handle) -> HandleWithGeneration<Handle> {
        if handle >= self.next_handle {
            // TODO Requiring `Handle: Step` would allow us to create a Range right from the Handle
            let inbetween_handles =
                (self.next_handle.clone().into()..handle.clone().into()).map(Handle::from);
            self.released_handles
                .extend(inbetween_handles.map(|handle| HandleWithGeneration {
                    handle,
                    generation: 0,
                }));
            self.next_handle = Self::increment(handle.clone());
            self._acquire(handle, 0)
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
                    self._acquire(handle, released_handle.generation + 1)
                }
                _ => {
                    panic!("Tried to acquire a specific handle but it was already acquired");
                }
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
        self.released_handles
            .push(HandleWithGeneration { handle, generation });
    }

    fn increment(handle: Handle) -> Handle {
        Handle::from(handle.into() + 1)
    }
}

// TODO Test
