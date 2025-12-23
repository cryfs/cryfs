//! Tokio mutex wrapper for types that implement [`AsyncDrop`].
//!
//! This module provides [`AsyncDropTokioMutex`], which wraps an [`AsyncDropGuard<T>`]
//! in a [`tokio::sync::Mutex`] for interior mutability in async contexts.

use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::{Mutex, MutexGuard};

use crate::async_drop::{AsyncDrop, AsyncDropGuard};

/// A tokio mutex wrapper for types that implement [`AsyncDrop`].
///
/// `AsyncDropTokioMutex` wraps an [`AsyncDropGuard<T>`] in a [`tokio::sync::Mutex`],
/// allowing multiple tasks to access the contained value with mutual exclusion.
/// When the mutex is dropped via [`AsyncDropGuard::async_drop`], it calls the
/// inner value's async drop.
#[derive(Debug)]
// TODO Why do we need Send? tokio::sync::Mutex doesn't seem to need it
pub struct AsyncDropTokioMutex<T: AsyncDrop + Debug + Send> {
    // Always Some except during destruction
    v: Option<Mutex<AsyncDropGuard<T>>>,
}

impl<T: AsyncDrop + Debug + Send> AsyncDropTokioMutex<T> {
    /// Creates a new `AsyncDropTokioMutex` wrapping the given value.
    ///
    /// The returned guard must have `async_drop()` called on it before being dropped.
    pub fn new(v: AsyncDropGuard<T>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            v: Some(Mutex::new(v)),
        })
    }

    /// Acquires the lock asynchronously.
    ///
    /// Returns a guard that provides mutable access to the inner `AsyncDropGuard<T>`.
    pub async fn lock(&self) -> MutexGuard<'_, AsyncDropGuard<T>> {
        self.v.as_ref().expect("Already destructed").lock().await
    }

    /// Extracts the inner `AsyncDropGuard<T>`, consuming the mutex.
    ///
    /// This bypasses the mutex's async drop and returns the inner value directly.
    pub fn into_inner(this: AsyncDropGuard<Self>) -> AsyncDropGuard<T> {
        let inner = this
            .unsafe_into_inner_dont_drop()
            .v
            .take()
            .expect("Already destructed");
        inner.into_inner()
    }
}

#[async_trait]
impl<T: AsyncDrop + Debug + Send> AsyncDrop for AsyncDropTokioMutex<T> {
    type Error = T::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let v = self.v.take().expect("Already destructed");
        let mut v = v.into_inner();
        v.async_drop().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[tokio::test]
    async fn test_new_and_lock() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut mutex = AsyncDropTokioMutex::new(inner);

        {
            let guard = mutex.lock().await;
            assert_eq!(42, guard.value);
        }

        mutex.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_lock_allows_mutation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut mutex = AsyncDropTokioMutex::new(inner);

        {
            let mut guard = mutex.lock().await;
            guard.value = 100;
        }

        {
            let guard = mutex.lock().await;
            assert_eq!(100, guard.value);
        }

        mutex.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_into_inner() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mutex = AsyncDropTokioMutex::new(inner);

        let mut inner = AsyncDropTokioMutex::into_inner(mutex);
        assert_eq!(42, inner.value);

        inner.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_async_drop_calls_inner_async_drop() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut mutex = AsyncDropTokioMutex::new(inner);

        assert_eq!(0, counter.load(Ordering::SeqCst));
        mutex.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }
}
