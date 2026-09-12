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
/// - Both Err: returns the first error and logs the second one
///
/// The error type is the one of the two inputs. If dropping a value fails while an error
/// is already being returned, the drop error is logged and the original error is returned.
pub async fn flatten_async_drop<E, T, U>(
    first: Result<AsyncDropGuard<T>, E>,
    second: Result<AsyncDropGuard<U>, E>,
) -> Result<(AsyncDropGuard<T>, AsyncDropGuard<U>), E>
where
    E: Debug,
    T: AsyncDrop + Debug,
    U: AsyncDrop + Debug,
{
    match (first, second) {
        (Ok(first), Ok(second)) => Ok((first, second)),
        (Ok(first), Err(second_error)) => {
            first.async_drop_on_error_path().await;
            Err(second_error)
        }
        (Err(first_error), Ok(second)) => {
            second.async_drop_on_error_path().await;
            Err(first_error)
        }
        (Err(first_error), Err(second_error)) => {
            log::error!(
                "Two operations failed. Reporting only the first error. Second error: {second_error:?}"
            );
            Err(first_error)
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
        id: &'static str,
        drop_counter: Arc<AtomicUsize>,
        fail_drop: bool,
    }

    impl TestValue {
        fn new(id: &'static str, drop_counter: &Arc<AtomicUsize>) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self {
                id,
                drop_counter: Arc::clone(drop_counter),
                fail_drop: false,
            })
        }

        fn with_failing_drop(
            id: &'static str,
            drop_counter: &Arc<AtomicUsize>,
        ) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self {
                id,
                drop_counter: Arc::clone(drop_counter),
                fail_drop: true,
            })
        }
    }

    impl AsyncDrop for TestValue {
        type Error = &'static str;

        async fn async_drop_impl(self) -> Result<(), Self::Error> {
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
            if self.fail_drop {
                Err("drop error")
            } else {
                Ok(())
            }
        }
    }

    /// An error type that is unrelated to the guards' `&'static str` drop error,
    /// so the tests show that the error type is taken from the inputs alone.
    #[derive(Debug, PartialEq, Eq)]
    struct TestError(&'static str);

    #[tokio::test]
    async fn test_both_ok() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new("first", &counter);
        let second = TestValue::new("second", &counter);

        let (first, second) = flatten_async_drop(Ok::<_, TestError>(first), Ok(second))
            .await
            .unwrap();
        assert_eq!("first", first.id);
        assert_eq!("second", second.id);
        assert_eq!(0, counter.load(Ordering::SeqCst));

        // Clean up
        first.async_drop().await.unwrap();
        second.async_drop().await.unwrap();
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_first_ok_second_err() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new("first", &counter);
        let second: Result<AsyncDropGuard<TestValue>, _> = Err(TestError("second error"));

        let result = flatten_async_drop(Ok(first), second).await;

        assert_eq!(TestError("second error"), result.unwrap_err());
        // First value should have been dropped
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_first_err_second_ok() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first: Result<AsyncDropGuard<TestValue>, _> = Err(TestError("first error"));
        let second = TestValue::new("second", &counter);

        let result = flatten_async_drop(first, Ok(second)).await;

        assert_eq!(TestError("first error"), result.unwrap_err());
        // Second value should have been dropped
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_both_err() {
        let first: Result<AsyncDropGuard<TestValue>, _> = Err(TestError("first error"));
        let second: Result<AsyncDropGuard<TestValue>, _> = Err(TestError("second error"));

        let result = flatten_async_drop(first, second).await;

        // Returns the first error
        assert_eq!(TestError("first error"), result.unwrap_err());
    }

    #[tokio::test]
    async fn test_first_ok_second_err_and_dropping_first_fails() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::with_failing_drop("first", &counter);
        let second: Result<AsyncDropGuard<TestValue>, _> = Err(TestError("second error"));

        let result = flatten_async_drop(Ok(first), second).await;

        // The original error is returned, the drop error is only logged
        assert_eq!(TestError("second error"), result.unwrap_err());
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_first_err_second_ok_and_dropping_second_fails() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first: Result<AsyncDropGuard<TestValue>, _> = Err(TestError("first error"));
        let second = TestValue::with_failing_drop("second", &counter);

        let result = flatten_async_drop(first, Ok(second)).await;

        assert_eq!(TestError("first error"), result.unwrap_err());
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }
}
