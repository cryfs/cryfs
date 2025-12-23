//! Synchronous drop wrapper for AsyncDrop types.
//!
//! This module provides [`SyncDrop`], which wraps an [`AsyncDropGuard`] and calls
//! `async_drop` synchronously in its `Drop` implementation. This is primarily
//! useful for test code where you want automatic cleanup without explicit async_drop calls.
//!
//! **WARNING**: This can cause deadlocks in certain scenarios. See the struct documentation.

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use super::{AsyncDrop, AsyncDropGuard};

/// Wraps an [`AsyncDropGuard`] and calls `async_drop` synchronously in its destructor.
///
/// This adapter allows `AsyncDrop` types to be used in synchronous contexts by
/// blocking on the async drop operation.
///
/// # Warning: Deadlock Risk
///
/// This can cause deadlocks if the async_drop code requires other tokio tasks to
/// make progress (e.g., releasing contended locks). See
/// <https://stackoverflow.com/questions/71541765/rust-async-drop>
///
/// This type attempts to mitigate deadlocks by using `tokio::task::block_in_place`
/// when inside a multi-threaded tokio runtime, but this is not a complete solution.
///
/// **Recommended usage**: Test code only.
pub struct SyncDrop<T: Debug + AsyncDrop>(Option<AsyncDropGuard<T>>);

impl<T: Debug + AsyncDrop> SyncDrop<T> {
    /// Creates a new `SyncDrop` wrapper.
    pub fn new(v: AsyncDropGuard<T>) -> Self {
        Self(Some(v))
    }

    /// Extracts the inner `AsyncDropGuard` without calling async_drop.
    ///
    /// The caller becomes responsible for calling `async_drop` on the returned guard.
    pub fn into_inner_dont_drop(mut self) -> AsyncDropGuard<T> {
        self.0.take().expect("Already dropped")
    }

    /// Returns a reference to the inner `AsyncDropGuard`.
    pub fn inner(&self) -> &AsyncDropGuard<T> {
        self.0.as_ref().expect("Already dropped")
    }
}

impl<T: Debug + AsyncDrop> Drop for SyncDrop<T> {
    fn drop(&mut self) {
        if let Some(mut v) = self.0.take() {
            // Use block_in_place if we're inside a tokio runtime to avoid deadlocks.
            // The async_drop code may use tokio::sync primitives that require other
            // tokio tasks to make progress (e.g., releasing contended locks).
            // If we just use futures::executor::block_on, we block the tokio worker
            // thread, preventing those tasks from running, causing a deadlock.
            if let Ok(handle) = tokio::runtime::Handle::try_current()
                && handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread
            {
                tokio::task::block_in_place(|| {
                    handle.block_on(v.async_drop()).unwrap();
                });
            } else {
                // No tokio runtime, use futures executor
                // Single threaded tokio runtime doesn't support block_on, so we also use this path.
                futures::executor::block_on(v.async_drop()).unwrap();
            }
        }
    }
}

impl<T: Debug + AsyncDrop> Deref for SyncDrop<T> {
    type Target = AsyncDropGuard<T>;
    fn deref(&self) -> &AsyncDropGuard<T> {
        self.0.as_ref().expect("Already dropped")
    }
}

impl<T: Debug + AsyncDrop> DerefMut for SyncDrop<T> {
    fn deref_mut(&mut self) -> &mut AsyncDropGuard<T> {
        self.0.as_mut().expect("Already dropped")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestValue {
        value: i32,
        drop_counter: Arc<AtomicUsize>,
    }

    impl TestValue {
        fn new(value: i32, drop_counter: Arc<AtomicUsize>) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self {
                value,
                drop_counter,
            })
        }
    }

    #[async_trait]
    impl AsyncDrop for TestValue {
        type Error = &'static str;

        async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_deref() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let sync_drop = SyncDrop::new(inner);

        // Test Deref - should access the inner guard
        assert_eq!(42, sync_drop.value);
    }

    #[test]
    fn test_deref_mut() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut sync_drop = SyncDrop::new(inner);

        // Test DerefMut - should allow mutation
        sync_drop.value = 100;
        assert_eq!(100, sync_drop.value);
    }

    #[test]
    fn test_inner() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let sync_drop = SyncDrop::new(inner);

        let guard = sync_drop.inner();
        assert_eq!(42, guard.value);
    }

    #[test]
    fn test_into_inner_dont_drop() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let sync_drop = SyncDrop::new(inner);

        let mut guard = sync_drop.into_inner_dont_drop();
        assert_eq!(42, guard.value);

        // async_drop was not called yet
        assert_eq!(0, counter.load(Ordering::SeqCst));

        // We need to manually drop it now
        futures::executor::block_on(guard.async_drop()).unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_drop_calls_async_drop_in_tokio_runtime() {
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let inner = TestValue::new(42, Arc::clone(&counter));
            let _sync_drop = SyncDrop::new(inner);
            assert_eq!(0, counter.load(Ordering::SeqCst));
        }

        // async_drop should have been called when sync_drop was dropped
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[test]
    fn test_drop_calls_async_drop_without_runtime() {
        let counter = Arc::new(AtomicUsize::new(0));

        {
            let inner = TestValue::new(42, Arc::clone(&counter));
            let _sync_drop = SyncDrop::new(inner);
            assert_eq!(0, counter.load(Ordering::SeqCst));
        }

        // async_drop should have been called when sync_drop was dropped
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }
}
