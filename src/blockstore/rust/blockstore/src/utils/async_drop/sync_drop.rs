use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use super::{AsyncDrop, AsyncDropGuard};

/// SyncDrop wraps an [AsyncDropGuard] and calls `AsyncDropGuard::async_drop` on it in its
/// synchronous [Drop] destructor.
///
/// WARNING: This can cause deadlocks, see https://stackoverflow.com/questions/71541765/rust-async-drop
/// Because of that, we only allow this in test code.
#[cfg(test)]
pub struct SyncDrop<T: Debug + AsyncDrop>(Option<AsyncDropGuard<T>>);

impl<T: Debug + AsyncDrop> SyncDrop<T> {
    pub fn new(v: AsyncDropGuard<T>) -> Self {
        Self(Some(v))
    }

    pub fn into_inner_dont_drop(mut self) -> AsyncDropGuard<T> {
        self.0.take().expect("Already dropped")
    }

    pub fn inner(&self) -> &AsyncDropGuard<T> {
        self.0.as_ref().expect("Already dropped")
    }
}

impl<T: Debug + AsyncDrop> Drop for SyncDrop<T> {
    fn drop(&mut self) {
        if let Some(mut v) = self.0.take() {
            futures::executor::block_on(v.async_drop()).unwrap();
        }
    }
}

impl<T: Debug + AsyncDrop> Deref for SyncDrop<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0.as_ref().expect("Already dropped")
    }
}

impl<T: Debug + AsyncDrop> DerefMut for SyncDrop<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0.as_mut().expect("Already dropped")
    }
}
