use owning_ref::OwningHandle;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard, TryLockError};

// TODO Test + Documentation

/// A LockedMutexGuard carries an Arc<Mutex<T>> together with a MutexGuard locking that Data.
pub struct LockedMutexGuard<T: 'static> {
    // TODO Is 'static needed?
    mutex_and_guard: OwningHandle<Arc<Mutex<T>>, MutexGuard<'static, T>>,
}

impl<T: 'static> LockedMutexGuard<T> {
    /// Lock the given mutex and return a [LockedMutexGuard] pointing to the data behind the mutex.
    pub fn blocking_lock(mutex: Arc<Mutex<T>>) -> Self {
        let mutex_and_guard = OwningHandle::new_with_fn(mutex, |mutex: *const Mutex<T>| {
            let mutex: &Mutex<T> = unsafe { &*mutex };
            let guard = mutex.blocking_lock();
            guard
        });
        Self { mutex_and_guard }
    }

    /// Lock the given mutex and return a [LockedMutexGuard] pointing to the data behind the mutex.
    pub async fn async_lock(mutex: Arc<Mutex<T>>) -> Self {
        let mutex_and_guard = OwningHandle::new_with_async_fn(mutex, |mutex: *const Mutex<T>| {
            let mutex: &Mutex<T> = unsafe { &*mutex };
            mutex.lock()
        })
        .await;
        // TODO Test that this can be held across an await point. With the following code, holding it across
        // an await point would cause a compiler error:
        // let mutex_and_guard =
        //     OwningHandle::new_with_async_fn(mutex, |mutex: *const Mutex<T>| async move {
        //         let mutex: &Mutex<T> = unsafe { &*mutex };
        //         mutex.lock().await
        //     })
        //     .await;
        Self { mutex_and_guard }
    }

    pub fn try_lock(mutex: Arc<Mutex<T>>) -> Result<Self, TryLockError> {
        let mutex_and_guard = OwningHandle::try_new(mutex, |mutex: *const Mutex<T>| {
            let mutex: &Mutex<T> = unsafe { &*mutex };
            let guard = mutex.try_lock()?;
            Ok(guard)
        })?;
        Ok(Self { mutex_and_guard })
    }
}

impl<T> Deref for LockedMutexGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.mutex_and_guard
    }
}

impl<T> DerefMut for LockedMutexGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.mutex_and_guard
    }
}
