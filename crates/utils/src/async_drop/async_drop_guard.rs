use std::fmt::Debug;
use std::future::Future;
use std::ops::{Deref, DerefMut};

use super::AsyncDrop;
use crate::safe_panic;

/// [AsyncDropGuard] allows async dropping of the contained value with a safety check.
///
/// Values wrapped in [AsyncDropGuard] offer an async [AsyncDropGuard::async_drop] function
/// that can be called to asynchronously drop the value. You must always manually call
/// [AsyncDropGuard::async_drop]. If the [AsyncDropGuard] leaves scope without a call to
/// [AsyncDropGuard::async_drop], a safety check will trigger and cause a panic.
///
/// [AsyncDropGuard::async_drop] consumes the guard, so using a value after it was dropped
/// is a compile error rather than a runtime failure.
///
/// Types wrapped in [AsyncDropGuard] must implement [AsyncDrop] to define what exactly
/// should happen when [AsyncDropGuard::async_drop] gets called.
///
/// **Warning:** If a type `T` is supposed to be used with [AsyncDropGuard], you must ensure
/// that there is no way to create instances of `T` that aren't wrapped in [AsyncDropGuard].
/// Ideally, `T`'s constructor directly creates an `AsyncDropGuard[T]`. If a `T` object
/// exists without being wrapped in [AsyncDropGuard], the safety check will not run and
/// call sites might forget to correctly drop `T`.
#[derive(Debug)]
#[must_use = "You have to call async_drop() on this value to drop it"]
pub struct AsyncDropGuard<T: Debug>(
    // Invariant: This is `Some` for the whole lifetime of the guard. The only code that sets it
    // to `None` consumes the guard, so `None` is only ever observed by the [Drop] impl.
    Option<T>,
);

impl<T: Debug> AsyncDropGuard<T> {
    /// Wrap a value into an [AsyncDropGuard]. This enables the safety checks and will enforce
    /// that [AsyncDropGuard::async_drop] gets called before the [AsyncDropGuard] instance leaves scope.
    pub fn new(v: T) -> Self {
        Self(Some(v))
    }

    pub fn into_box(self) -> AsyncDropGuard<Box<T>> {
        self.map_unsafe(Box::new)
    }

    // Warning: The resulting AsyncDropGuard will call async_drop on U instead of T.
    // There will be no call to async_drop for T anymore.
    // Callers of this function need to make sure that this is correct behavior for T, U.
    pub fn map_unsafe<U: Debug>(self, fun: impl FnOnce(T) -> U) -> AsyncDropGuard<U> {
        AsyncDropGuard(Some(fun(self.into_inner_unchecked())))
    }

    /// Extract the inner value **without** dropping it. This bypasses the protection of the guard.
    pub fn unsafe_into_inner_dont_drop(self) -> T {
        self.into_inner_unchecked()
    }

    /// Take the value out of the guard. This consumes the guard, so the safety check in [Drop]
    /// sees `None` and passes.
    fn into_inner_unchecked(mut self) -> T {
        self.0
            .take()
            .expect("Invariant violated: AsyncDropGuard must hold a value for its whole lifetime")
    }
}

impl<T: Debug + AsyncDrop> AsyncDropGuard<T> {
    /// Asynchronously drop the value. This will call [AsyncDrop::async_drop_impl]
    /// on the contained value.
    ///
    /// This consumes the guard, so the value cannot be used anymore afterwards:
    ///
    /// ```compile_fail
    /// use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
    ///
    /// #[derive(Debug)]
    /// struct Value;
    ///
    /// impl AsyncDrop for Value {
    ///     type Error = std::convert::Infallible;
    ///     async fn async_drop_impl(self) -> Result<(), Self::Error> {
    ///         Ok(())
    ///     }
    /// }
    ///
    /// # futures::executor::block_on(async {
    /// let guard = AsyncDropGuard::new(Value);
    /// guard.async_drop().await.unwrap();
    /// let _use_after_drop: &Value = &guard; // error[E0382]: borrow of moved value: `guard`
    /// # });
    /// ```
    ///
    /// If this function does not get executed and the [AsyncDropGuard] instance leaves scope,
    /// that will cause a panic.
    pub fn async_drop(self) -> impl Future<Output = Result<(), T::Error>> + Send {
        self.into_inner_unchecked().async_drop_impl()
    }

    /// Drops the guard if `result` is an error, otherwise hands the value and the guard back.
    ///
    /// This is for functions that keep the guard on their success path (e.g. move it into a
    /// new object) but have to drop it on every error path:
    ///
    /// ```
    /// # use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard};
    /// # #[derive(Debug)] struct Blob;
    /// # impl AsyncDrop for Blob {
    /// #     type Error = std::convert::Infallible;
    /// #     async fn async_drop_impl(self) -> Result<(), Self::Error> { Ok(()) }
    /// # }
    /// # struct Node { blob: AsyncDropGuard<Blob> }
    /// # async fn load_blob() -> AsyncDropGuard<Blob> { AsyncDropGuard::new(Blob) }
    /// # async fn create_child() -> Result<u32, &'static str> { Ok(1) }
    /// # async fn example() -> Result<Node, &'static str> {
    /// let blob = load_blob().await;
    /// // If `create_child` failed, `blob` is dropped and the error is returned here.
    /// let (child_id, blob) = blob.async_drop_on_err(create_child().await).await?;
    /// Ok(Node { blob })
    /// # }
    /// # futures::executor::block_on(async { example().await.unwrap().blob.async_drop().await.unwrap() });
    /// ```
    ///
    /// If dropping fails on the error path, the drop error is logged and the original error
    /// is still returned, because that is the error the caller needs to know about.
    pub async fn async_drop_on_err<R, E>(self, result: Result<R, E>) -> Result<(R, Self), E> {
        match result {
            Ok(value) => Ok((value, self)),
            Err(error) => {
                self.async_drop_on_error_path().await;
                Err(error)
            }
        }
    }

    /// Drops the guard while already handling another error. A failure to drop is logged
    /// instead of returned, so that the original error stays the one that gets reported.
    pub(super) async fn async_drop_on_error_path(self) {
        if let Err(drop_error) = self.async_drop().await {
            log::error!(
                "Error while dropping a value on an error path. Reporting the original error instead. Drop error: {drop_error:?}"
            );
        }
    }
}

impl<T: Debug> Drop for AsyncDropGuard<T> {
    #[track_caller]
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => {
                safe_panic!("Forgot to call async_drop on {:?}", v);
            }
            None => {
                // Everything is ok
            }
        }
    }
}

impl<T: Debug> Deref for AsyncDropGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
            .as_ref()
            .expect("Invariant violated: AsyncDropGuard must hold a value for its whole lifetime")
    }
}

impl<T: Debug> DerefMut for AsyncDropGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
            .as_mut()
            .expect("Invariant violated: AsyncDropGuard must hold a value for its whole lifetime")
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncDrop, AsyncDropGuard};

    use std::fmt::{self, Debug};
    use std::future::Future;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};

    struct MyStructWithDrop<F, FA, FS>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
        FS: Fn() + Send,
    {
        on_async_drop: FA,
        on_sync_drop: FS,
    }

    impl<F, FA, FS> Debug for MyStructWithDrop<F, FA, FS>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
        FS: Fn() + Send,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MyStructWithDrop").finish()
        }
    }

    impl<F, FA, FS> AsyncDrop for MyStructWithDrop<F, FA, FS>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
        FS: Fn() + Send,
    {
        type Error = &'static str;

        async fn async_drop_impl(self) -> Result<(), &'static str> {
            let r = (self.on_async_drop)();
            r.await
        }
    }

    impl<F, FA, FS> Drop for MyStructWithDrop<F, FA, FS>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
        FS: Fn() + Send,
    {
        fn drop(&mut self) {
            (self.on_sync_drop)();
        }
    }

    struct MyStructWithoutDrop<F, FA>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
    {
        on_async_drop: FA,
    }

    impl<F, FA> Debug for MyStructWithoutDrop<F, FA>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MyStructWithoutDrop").finish()
        }
    }

    impl<F, FA> AsyncDrop for MyStructWithoutDrop<F, FA>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
    {
        type Error = &'static str;

        async fn async_drop_impl(self) -> Result<(), &'static str> {
            let r = (self.on_async_drop)();
            r.await
        }
    }

    #[tokio::test]
    #[should_panic(expected = "Forgot to call async_drop on MyStructWithoutDrop")]
    async fn given_type_without_drop_when_forgetting_to_call_async_drop_then_panics() {
        let _obj = AsyncDropGuard::new(MyStructWithoutDrop {
            on_async_drop: async || Ok(()),
        });
    }

    #[tokio::test]
    #[should_panic(expected = "Forgot to call async_drop on MyStructWithDrop")]
    async fn given_type_with_drop_when_forgetting_to_call_async_drop_then_panics() {
        let _obj = AsyncDropGuard::new(MyStructWithDrop {
            on_async_drop: async || Ok(()),
            on_sync_drop: || (),
        });
    }

    #[tokio::test]
    async fn given_type_without_drop_when_calling_async_drop_then_calls_async_drop_impl() {
        let called = AtomicI32::new(0);
        let obj = AsyncDropGuard::new(MyStructWithoutDrop {
            on_async_drop: async || {
                let prev_value = called.swap(1, Ordering::SeqCst);
                assert_eq!(0, prev_value);
                Ok(())
            },
        });
        obj.async_drop().await.unwrap();
        assert_eq!(1, called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_type_with_drop_when_calling_async_drop_then_calls_async_drop_impl_and_then_calls_drop()
     {
        let called = AtomicI32::new(0);
        let obj = AsyncDropGuard::new(MyStructWithDrop {
            on_async_drop: async || {
                let prev_value = called.swap(1, Ordering::SeqCst);
                assert_eq!(0, prev_value);
                Ok(())
            },
            on_sync_drop: || {
                let prev_value = called.swap(2, Ordering::SeqCst);
                assert_eq!(1, prev_value);
            },
        });
        obj.async_drop().await.unwrap();
        assert_eq!(2, called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_type_without_drop_when_async_drop_fails_then_returns_error() {
        let obj = AsyncDropGuard::new(MyStructWithoutDrop {
            on_async_drop: async || Err("My error"),
        });
        assert_eq!(Err("My error"), obj.async_drop().await);
    }

    #[tokio::test]
    async fn given_type_with_drop_when_async_drop_fails_then_returns_error_and_still_calls_drop() {
        let called = AtomicBool::new(false);
        let obj = AsyncDropGuard::new(MyStructWithDrop {
            on_async_drop: async || Err("My error"),
            on_sync_drop: || {
                called.store(true, Ordering::SeqCst);
            },
        });
        assert_eq!(Err("My error"), obj.async_drop().await);
        assert_eq!(true, called.load(Ordering::SeqCst));
    }

    #[derive(Debug)]
    struct CountingValue {
        drop_counter: Arc<AtomicUsize>,
        fail_drop: bool,
    }

    impl AsyncDrop for CountingValue {
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

    #[tokio::test]
    async fn given_ok_result_when_async_drop_on_err_then_value_and_guard_are_returned_undropped() {
        let counter = Arc::new(AtomicUsize::new(0));
        let guard = AsyncDropGuard::new(CountingValue {
            drop_counter: Arc::clone(&counter),
            fail_drop: false,
        });

        let (value, guard) = guard
            .async_drop_on_err(Ok::<_, &'static str>(42))
            .await
            .unwrap();

        assert_eq!(42, value);
        assert_eq!(0, counter.load(Ordering::SeqCst));

        guard.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_err_result_when_async_drop_on_err_then_guard_is_dropped_and_error_returned() {
        let counter = Arc::new(AtomicUsize::new(0));
        let guard = AsyncDropGuard::new(CountingValue {
            drop_counter: Arc::clone(&counter),
            fail_drop: false,
        });

        let result = guard
            .async_drop_on_err(Err::<i32, _>("original error"))
            .await;

        assert_eq!("original error", result.unwrap_err());
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_err_result_and_failing_drop_when_async_drop_on_err_then_original_error_returned()
    {
        let counter = Arc::new(AtomicUsize::new(0));
        let guard = AsyncDropGuard::new(CountingValue {
            drop_counter: Arc::clone(&counter),
            fail_drop: true,
        });

        let result = guard
            .async_drop_on_err(Err::<i32, _>("original error"))
            .await;

        // The drop error is only logged, the original error is what the caller gets
        assert_eq!("original error", result.unwrap_err());
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }
}
