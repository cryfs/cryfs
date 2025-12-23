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

/// Executes a block of code and ensures `async_drop` is called on the value afterward.
///
/// This macro takes an `AsyncDropGuard` and a block of code to execute. After the block
/// completes (whether successfully or with an error), it calls `async_drop` on the value.
///
/// # Forms
///
/// - `with_async_drop_2!(value, { ... })` - Propagates async_drop errors directly
/// - `with_async_drop_2!(value, { ... }, err_map)` - Maps async_drop errors using `err_map`
#[macro_export]
macro_rules! with_async_drop_2 {
    ($value:ident, $f:block) => {
        async {
            let result = (async || $f)().await;
            let mut value = $value;
            value.async_drop().await?;
            result
        }
        .await
    };
    ($value:ident, $f:block, $err_map:expr) => {
        async {
            let result = (async || $f)().await;
            let mut value = $value;
            value.async_drop().await.map_err($err_map)?;
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
    ($value:ident, $f:block) => {
        async {
            use lockable::InfallibleUnwrap as _;
            let result = (async || $f)().await;
            let mut value = $value;
            value.async_drop().await.infallible_unwrap();
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
