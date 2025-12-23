//! Utility for combining two Result values containing AsyncDropGuards.
//!
//! This module provides [`flatten_async_drop`] which safely combines two fallible
//! results, ensuring proper cleanup of any successfully created values when
//! either result is an error.

use std::fmt::Debug;

use super::{AsyncDrop, AsyncDropGuard};

/// Flattens two Result values that contain AsyncDropGuards, making sure that we correctly drop things if errors happen.
///
/// This function handles four cases:
/// - Both Ok: returns both values as a tuple
/// - First Ok, Second Err: drops the first value, returns the second error
/// - First Err, Second Ok: drops the second value, returns the first error
/// - Both Err: returns the first error (second error is currently lost)
pub async fn flatten_async_drop<E, T, E1, U, E2>(
    first: Result<AsyncDropGuard<T>, E1>,
    second: Result<AsyncDropGuard<U>, E2>,
) -> Result<(AsyncDropGuard<T>, AsyncDropGuard<U>), E>
where
    T: AsyncDrop + Debug,
    U: AsyncDrop + Debug,
    E: From<E1> + From<E2> + From<<T as AsyncDrop>::Error> + From<<U as AsyncDrop>::Error>,
{
    match (first, second) {
        (Ok(first), Ok(second)) => Ok((first, second)),
        (Ok(mut first), Err(second)) => {
            // TODO Report both errors if async_drop fails
            first.async_drop().await?;
            Err(second.into())
        }
        (Err(first), Ok(mut second)) => {
            // TODO Report both errors if async_drop fails
            second.async_drop().await?;
            Err(first.into())
        }
        (Err(first), Err(_second)) => {
            // TODO Report both errors
            Err(first.into())
        }
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
        id: &'static str,
        drop_counter: Arc<AtomicUsize>,
    }

    impl TestValue {
        fn new(id: &'static str, drop_counter: Arc<AtomicUsize>) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self { id, drop_counter })
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

    #[derive(Debug)]
    struct TestError(&'static str);

    impl From<&'static str> for TestError {
        fn from(s: &'static str) -> Self {
            TestError(s)
        }
    }

    #[tokio::test]
    async fn test_both_ok() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new("first", Arc::clone(&counter));
        let second = TestValue::new("second", Arc::clone(&counter));

        let first_result: Result<_, &'static str> = Ok(first);
        let second_result: Result<_, &'static str> = Ok(second);
        let result: Result<_, TestError> = flatten_async_drop(first_result, second_result).await;
        assert!(result.is_ok());

        let (mut first, mut second) = result.unwrap();
        assert_eq!("first", first.id);
        assert_eq!("second", second.id);

        // Clean up
        first.async_drop().await.unwrap();
        second.async_drop().await.unwrap();
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_first_ok_second_err() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new("first", Arc::clone(&counter));

        let first_result: Result<_, &'static str> = Ok(first);
        let second_result: Result<AsyncDropGuard<TestValue>, _> = Err("second error");
        let result: Result<(AsyncDropGuard<TestValue>, AsyncDropGuard<TestValue>), TestError> =
            flatten_async_drop(first_result, second_result).await;

        assert!(result.is_err());
        assert_eq!("second error", result.unwrap_err().0);
        // First value should have been dropped
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_first_err_second_ok() {
        let counter = Arc::new(AtomicUsize::new(0));
        let second = TestValue::new("second", Arc::clone(&counter));

        let first_result: Result<AsyncDropGuard<TestValue>, _> = Err("first error");
        let second_result: Result<_, &'static str> = Ok(second);
        let result: Result<(AsyncDropGuard<TestValue>, AsyncDropGuard<TestValue>), TestError> =
            flatten_async_drop(first_result, second_result).await;

        assert!(result.is_err());
        assert_eq!("first error", result.unwrap_err().0);
        // Second value should have been dropped
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_both_err() {
        let result: Result<(AsyncDropGuard<TestValue>, AsyncDropGuard<TestValue>), TestError> =
            flatten_async_drop(Err("first error"), Err("second error")).await;

        assert!(result.is_err());
        // Returns the first error
        assert_eq!("first error", result.unwrap_err().0);
    }
}
