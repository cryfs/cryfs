//! Result wrapper for types that implement [`AsyncDrop`].
//!
//! This module provides [`AsyncDropResult`], which wraps a `Result<AsyncDropGuard<T>, E>`.
//! When dropped, it only calls async_drop on the `Ok` variant; the `Err` variant is a no-op.

use async_trait::async_trait;
use std::fmt::Debug;

use crate::async_drop::{AsyncDrop, AsyncDropGuard};

/// A Result wrapper for types that implement [`AsyncDrop`].
///
/// `AsyncDropResult` wraps a `Result<AsyncDropGuard<T>, E>`. When async_drop is called:
/// - For the `Ok` variant: calls the inner value's async_drop
/// - For the `Err` variant: does nothing (succeeds immediately)
///
/// This is useful when you have a fallible operation that returns an `AsyncDropGuard`
/// and you want to ensure proper cleanup regardless of success or failure.
#[derive(Debug)]
pub struct AsyncDropResult<T, E>
where
    T: Debug + AsyncDrop + Send,
    E: Debug + Send,
{
    v: Result<AsyncDropGuard<T>, E>,
}

impl<T, E> AsyncDropResult<T, E>
where
    T: Debug + AsyncDrop + Send,
    E: Debug + Send,
{
    /// Creates a new `AsyncDropResult` wrapping the given result.
    pub fn new(v: Result<AsyncDropGuard<T>, E>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { v })
    }

    /// Returns a reference to the error if this is an `Err` variant.
    pub fn err(&self) -> Option<&E> {
        self.v.as_ref().err()
    }

    /// Returns a reference to the inner value if this is an `Ok` variant.
    pub fn ok(&self) -> Option<&T> {
        match &self.v {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    }

    /// Returns a reference to the inner result.
    pub fn as_inner(&self) -> Result<&AsyncDropGuard<T>, &E> {
        match &self.v {
            Ok(t) => Ok(t),
            Err(e) => Err(e),
        }
    }

    /// Extracts the inner result, consuming the wrapper without calling async_drop.
    pub fn into_inner(this: AsyncDropGuard<Self>) -> Result<AsyncDropGuard<T>, E> {
        this.unsafe_into_inner_dont_drop().v
    }
}

#[async_trait]
impl<T, E> AsyncDrop for AsyncDropResult<T, E>
where
    T: Debug + AsyncDrop + Send,
    E: Debug + Send,
{
    type Error = <T as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        match &mut self.v {
            Ok(v) => v.async_drop().await,
            Err(_) => Ok(()),
        }
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
    async fn test_ok_variant_accessors() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut result: AsyncDropGuard<AsyncDropResult<TestValue, &str>> =
            AsyncDropResult::new(Ok(inner));

        assert!(result.ok().is_some());
        assert_eq!(42, result.ok().unwrap().value);
        assert!(result.err().is_none());
        assert!(result.as_inner().is_ok());

        result.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_err_variant_accessors() {
        let mut result: AsyncDropGuard<AsyncDropResult<TestValue, &str>> =
            AsyncDropResult::new(Err("error"));

        assert!(result.ok().is_none());
        assert!(result.err().is_some());
        assert_eq!(&"error", result.err().unwrap());
        assert!(result.as_inner().is_err());

        result.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_into_inner_ok() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let result: AsyncDropGuard<AsyncDropResult<TestValue, &str>> =
            AsyncDropResult::new(Ok(inner));

        let inner_result = AsyncDropResult::into_inner(result);
        assert!(inner_result.is_ok());

        let mut inner = inner_result.unwrap();
        assert_eq!(42, inner.value);
        inner.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_into_inner_err() {
        let result: AsyncDropGuard<AsyncDropResult<TestValue, &str>> =
            AsyncDropResult::new(Err("error"));

        let inner_result = AsyncDropResult::into_inner(result);
        assert!(inner_result.is_err());
        assert_eq!("error", inner_result.unwrap_err());
    }

    #[tokio::test]
    async fn test_async_drop_on_ok_calls_inner() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut result: AsyncDropGuard<AsyncDropResult<TestValue, &str>> =
            AsyncDropResult::new(Ok(inner));

        assert_eq!(0, counter.load(Ordering::SeqCst));
        result.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_async_drop_on_err_succeeds() {
        let mut result: AsyncDropGuard<AsyncDropResult<TestValue, &str>> =
            AsyncDropResult::new(Err("error"));

        // Should succeed without doing anything
        let drop_result = result.async_drop().await;
        assert!(drop_result.is_ok());
    }
}
