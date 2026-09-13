//! RAII-style helpers for ensuring async_drop is called.
//!
//! This module provides the [with_async_drop!](crate::with_async_drop) and
//! [with_async_drop_infallible!](crate::with_async_drop_infallible) macros, which ensure
//! `async_drop` is called on `AsyncDropGuard`s even if the block returns early or fails.

/// Executes a block of code and ensures `async_drop` is called on the values afterward.
///
/// This macro takes one or more `AsyncDropGuard`s and a block of code to execute. After the
/// block completes (whether successfully or with an error), it calls `async_drop` on the values.
/// It is a macro rather than a function so that the block can use the guards by name (and
/// borrow them) while the macro still owns them and can consume them afterwards.
/// Several values are dropped concurrently via [async_drop_all](crate::async_drop::async_drop_all),
/// so they must be independent of each other and share the same error type.
///
/// # Forms
///
/// - `with_async_drop!(value, { ... })` - Propagates async_drop errors directly
/// - `with_async_drop!(value, { ... }, err_map)` - Maps async_drop errors using `err_map`
/// - `with_async_drop!(first, second, ..., { ... })` - Drops all values concurrently afterward
/// - `with_async_drop!(first, second, ..., { ... }, err_map)` - Same, mapping async_drop errors
#[macro_export]
macro_rules! with_async_drop {
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

/// Variant of [`with_async_drop!`] for types that return a `Never` error in their async_drop.
///
/// Since the error type is `Never` (infallible), this macro unwraps the result directly.
#[macro_export]
macro_rules! with_async_drop_infallible {
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

#[cfg(test)]
mod tests {
    use crate::async_drop::{AsyncDrop, AsyncDropGuard};
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
    async fn test_with_async_drop_macro_multiple_guards_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new(1, Arc::clone(&counter));
        let second = TestValue::new(2, Arc::clone(&counter));
        let third = TestValue::new(3, Arc::clone(&counter));

        let result: Result<i32, &'static str> = with_async_drop!(first, second, third, {
            Ok(first.value + second.value + third.value)
        });

        assert_eq!(Ok(6), result);
        assert_eq!(3, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_macro_multiple_guards_block_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new(1, Arc::clone(&counter));
        let second = TestValue::new(2, Arc::clone(&counter));

        let result: Result<i32, &'static str> =
            with_async_drop!(first, second, { Err("block error") });

        assert_eq!(Err("block error"), result);
        // Both guards are dropped even though the block failed
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_macro_multiple_guards_with_error_map() {
        let counter = Arc::new(AtomicUsize::new(0));
        let first = TestValue::new(1, Arc::clone(&counter));
        let second = TestValue::new(2, Arc::clone(&counter));

        let result: Result<i32, String> =
            with_async_drop!(first, second, { Ok(84) }, |e: &str| e.to_string());

        assert_eq!(Ok(84), result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_infallible_macro_multiple_guards() {
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

        let result: i32 = with_async_drop_infallible!(first, second, { 42 });

        assert_eq!(42, result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_macro_success() {
        let counter = Arc::new(AtomicUsize::new(0));
        let value = TestValue::new(42, Arc::clone(&counter));

        let result: Result<i32, &'static str> = with_async_drop!(value, { Ok(84) });

        assert_eq!(Ok(84), result);
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_with_async_drop_macro_with_error_map() {
        let counter = Arc::new(AtomicUsize::new(0));
        let value = TestValue::new(42, Arc::clone(&counter));

        let result: Result<i32, String> =
            with_async_drop!(value, { Ok(84) }, |e: &str| e.to_string());

        assert_eq!(Ok(84), result);
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }
}
