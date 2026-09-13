//! RAII-style helpers for ensuring async_drop is called.
//!
//! This module provides macros and functions that ensure `async_drop` is called
//! on an `AsyncDropGuard` even if the callback returns early or fails.

use std::fmt::Debug;
use std::future::Future;

use super::{AsyncDrop, AsyncDropGuard};

// TODO It seems that actually most of our call sites only use sync callbacks
//      and have quite a hard time calling this because they need to wrap their callbacks into future::ready.
//      Offer sync versions instead.

// TODO Why does this need to be a macro? Can't call sites just use the function version?

/// Executes a block of code and ensures `async_drop` is called on the values afterward.
///
/// This macro takes one or more `AsyncDropGuard`s and a block of code to execute. After the
/// block completes (whether successfully or with an error), it calls `async_drop` on the values.
/// Several values are dropped concurrently via [async_drop_all](crate::async_drop::async_drop_all),
/// so they must be independent of each other and share the same error type.
///
/// # Forms
///
/// - `with_async_drop_2!(value, { ... })` - Propagates async_drop errors directly
/// - `with_async_drop_2!(value, { ... }, err_map)` - Maps async_drop errors using `err_map`
/// - `with_async_drop_2!(first, second, ..., { ... })` - Drops all values concurrently afterward
/// - `with_async_drop_2!(first, second, ..., { ... }, err_map)` - Same, mapping async_drop errors
#[macro_export]
macro_rules! with_async_drop_2 {
    ($($value:ident),+ , $f:block) => {
        async {
            let result = (async || $f)().await;
            $crate::async_drop::async_drop_all(($($value,)+)).await?;
            result
        }
        .await
    };
    ($($value:ident),+ , $f:block, $err_map:expr) => {
        async {
            let result = (async || $f)().await;
            $crate::async_drop::async_drop_all(($($value,)+))
                .await
                .map_err($err_map)?;
            result
        }
        .await
    };
}

/// Variant of [`with_async_drop_2!`] for types that return a `Never` error in their async_drop.
///
/// Since the error type is `Never` (infallible), this macro unwraps the result directly.
#[macro_export]
macro_rules! with_async_drop_2_infallible {
    ($($value:ident),+ , $f:block) => {
        async {
            use lockable::InfallibleUnwrap as _;
            let result = (async || $f)().await;
            $crate::async_drop::async_drop_all(($($value,)+))
                .await
                .infallible_unwrap();
            result
        }
        .await
    };
}

/// Executes a callback with a reference to the contained value, then calls async_drop.
///
/// This function provides a more functional approach to ensuring cleanup. The callback
/// receives a mutable reference to the inner value and can perform any operations needed.
/// After the callback completes, `async_drop` is called automatically.
///
/// # Arguments
///
/// * `value` - The `AsyncDropGuard` containing the value to operate on
/// * `f` - A callback that receives a mutable reference to the inner value
///
/// # Returns
///
/// Returns the result of the callback if both the callback and async_drop succeed.
/// Returns an error if either the callback or async_drop fails.
pub async fn with_async_drop<T, R, E, F>(
    mut value: AsyncDropGuard<T>,
    f: impl FnOnce(&mut T) -> F,
) -> Result<R, E>
where
    T: AsyncDrop + Debug,
    E: From<<T as AsyncDrop>::Error>,
    F: Future<Output = Result<R, E>>,
{
    let result = f(&mut value).await;
    value.async_drop().await?;
    result
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

    impl AsyncDrop for TestValue {
        type Error = &'static str;

        async fn async_drop_impl(self) -> Result<(), Self::Error> {
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_with_async_drop_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let guard = TestValue::new(42, Arc::clone(&counter));

        let result: Result<i32, &'static str> = with_async_drop(guard, |v| {
            let val = v.value;
            async move { Ok(val * 2) }
        })
        .await;

        assert_eq!(Ok(84), result);
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_callback_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let guard = TestValue::new(42, Arc::clone(&counter));

        let result: Result<i32, &'static str> =
            with_async_drop(guard, |_v| async move { Err("callback error") }).await;

        assert_eq!(Err("callback error"), result);
        // async_drop should still be called
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_2_macro_multiple_guards_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new(1, Arc::clone(&counter));
        let second = TestValue::new(2, Arc::clone(&counter));
        let third = TestValue::new(3, Arc::clone(&counter));

        let result: Result<i32, &'static str> = with_async_drop_2!(first, second, third, {
            Ok(first.value + second.value + third.value)
        });

        assert_eq!(Ok(6), result);
        assert_eq!(3, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_2_macro_multiple_guards_block_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new(1, Arc::clone(&counter));
        let second = TestValue::new(2, Arc::clone(&counter));

        let result: Result<i32, &'static str> =
            with_async_drop_2!(first, second, { Err("block error") });

        assert_eq!(Err("block error"), result);
        // Both guards are dropped even though the block failed
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_2_macro_multiple_guards_with_error_map() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new(1, Arc::clone(&counter));
        let second = TestValue::new(2, Arc::clone(&counter));

        let result: Result<i32, String> =
            with_async_drop_2!(first, second, { Ok(84) }, |e: &str| e.to_string());

        assert_eq!(Ok(84), result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_2_infallible_macro_multiple_guards() {
        #[derive(Debug)]
        struct InfallibleValue(Arc<AtomicUsize>);
        impl AsyncDrop for InfallibleValue {
            type Error = lockable::Never;
            async fn async_drop_impl(self) -> Result<(), Self::Error> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let first = AsyncDropGuard::new(InfallibleValue(Arc::clone(&counter)));
        let second = AsyncDropGuard::new(InfallibleValue(Arc::clone(&counter)));

        let result: i32 = with_async_drop_2_infallible!(first, second, { 42 });

        assert_eq!(42, result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_2_macro_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let value = TestValue::new(42, Arc::clone(&counter));

        let result: Result<i32, &'static str> = with_async_drop_2!(value, { Ok(84) });

        assert_eq!(Ok(84), result);
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_2_macro_with_error_map() {
        let counter = Arc::new(AtomicUsize::new(0));
        let value = TestValue::new(42, Arc::clone(&counter));

        let result: Result<i32, String> =
            with_async_drop_2!(value, { Ok(84) }, |e: &str| e.to_string());

        assert_eq!(Ok(84), result);
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }
}
