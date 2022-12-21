use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use super::AsyncDrop;

/// [AsyncDropGuard] allows async dropping of the contained value with a safety check.
///
/// Values wrapped in [AsyncDropGuard] offer an async [AsyncDropGuard::async_drop] function
/// that can be called to asynchronously drop the value. You must always manually call
/// [AsyncDropGuard::async_drop]. If the [AsyncDropGuard] leaves scope without a call to
/// [AsyncDropGuard::async_drop], a safety check will trigger and cause a panic.
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
pub struct AsyncDropGuard<T: Debug>(Option<T>);

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
    pub fn map_unsafe<U: Debug>(mut self, fun: impl FnOnce(T) -> U) -> AsyncDropGuard<U> {
        AsyncDropGuard(self.0.take().map(fun))
    }

    /// Extract the inner value **without** dropping it. This bypasses the protection of the guard.
    pub fn unsafe_into_inner_dont_drop(mut self) -> T {
        self.0.take().expect("Value already dropped")
    }
}

impl<T: Debug + AsyncDrop> AsyncDropGuard<T> {
    /// Asynchronously drop the value. This will call [AsyncDrop::async_drop_impl]
    /// on the contained value.
    /// Calling code must ensure that the `self` value is dropped after this is called.
    ///
    /// If this function does not get executed and the [AsyncDropGuard] instance leaves scope,
    /// that will cause a panic.
    pub async fn async_drop(&mut self) -> Result<(), T::Error> {
        self.0
            .take()
            // This expect cannot fail since the only place where we set it to None
            // is AsyncDropGuard::async_drop which consumes the whole AsyncDropGuard struct
            .expect("Value already dropped")
            .async_drop_impl()
            .await
    }
}

impl<T: Debug> Drop for AsyncDropGuard<T> {
    fn drop(&mut self) {
        match &self.0 {
            Some(v) => {
                // The AsyncDropGuard left scope without the user calling async_drop on it
                if std::thread::panicking() {
                    // We're already panicking, double panic wouldn't show a good error message anyways. Let's just log instead.
                    // A common scenario for this to happen is a failing test case.
                    log::error!("Forgot to call async_drop on {:?}", v);
                } else {
                    panic!("Forgot to call async_drop on {:?}", v);
                }
            }
            None => {
                // Everything is ok
                ()
            }
        }
    }
}

impl<T: Debug> Deref for AsyncDropGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.0
            .as_ref()
            // This expect cannot fail since the only place where we set it to None
            // is AsyncDropGuard::async_drop which consumes the whole AsyncDropGuard struct
            .expect("Value already dropped")
    }
}

impl<T: Debug> DerefMut for AsyncDropGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.0
            .as_mut()
            // This expect cannot fail since the only place where we set it to None
            // is AsyncDropGuard::async_drop which consumes the whole AsyncDropGuard struct
            .expect("Value already dropped")
    }
}

#[cfg(test)]
mod tests {
    use super::{AsyncDrop, AsyncDropGuard};

    use async_trait::async_trait;
    use std::fmt::{self, Debug};
    use std::future::Future;
    use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

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

    #[async_trait]
    impl<F, FA, FS> AsyncDrop for MyStructWithDrop<F, FA, FS>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
        FS: Fn() + Send,
    {
        type Error = &'static str;

        async fn async_drop_impl(&mut self) -> Result<(), &'static str> {
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

    #[async_trait]
    impl<F, FA> AsyncDrop for MyStructWithoutDrop<F, FA>
    where
        F: Future<Output = Result<(), &'static str>> + Send,
        FA: Fn() -> F + Send,
    {
        type Error = &'static str;

        async fn async_drop_impl(&mut self) -> Result<(), &'static str> {
            let r = (self.on_async_drop)();
            r.await
        }
    }

    #[tokio::test]
    #[should_panic(expected = "Forgot to call async_drop on MyStructWithoutDrop")]
    async fn given_type_without_drop_when_forgetting_to_call_async_drop_then_panics() {
        let _obj = AsyncDropGuard::new(MyStructWithoutDrop {
            on_async_drop: || async { Ok(()) },
        });
    }

    #[tokio::test]
    #[should_panic(expected = "Forgot to call async_drop on MyStructWithDrop")]
    async fn given_type_with_drop_when_forgetting_to_call_async_drop_then_panics() {
        let _obj = AsyncDropGuard::new(MyStructWithDrop {
            on_async_drop: || async { Ok(()) },
            on_sync_drop: || (),
        });
    }

    #[tokio::test]
    async fn given_type_without_drop_when_calling_async_drop_then_calls_async_drop_impl() {
        let called = AtomicI32::new(0);
        let mut obj = AsyncDropGuard::new(MyStructWithoutDrop {
            on_async_drop: || async {
                let prev_value = called.swap(1, Ordering::SeqCst);
                assert_eq!(0, prev_value);
                Ok(())
            },
        });
        obj.async_drop().await.unwrap();
        assert_eq!(1, called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_type_with_drop_when_calling_async_drop_then_calls_async_drop_impl_and_then_calls_drop(
    ) {
        let called = AtomicI32::new(0);
        let mut obj = AsyncDropGuard::new(MyStructWithDrop {
            on_async_drop: || async {
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
        let mut obj = AsyncDropGuard::new(MyStructWithoutDrop {
            on_async_drop: || async { Err("My error") },
        });
        assert_eq!(Err("My error"), obj.async_drop().await);
    }

    #[tokio::test]
    async fn given_type_with_drop_when_async_drop_fails_then_returns_error_and_still_calls_drop() {
        let called = AtomicBool::new(false);
        let mut obj = AsyncDropGuard::new(MyStructWithDrop {
            on_async_drop: || async { Err("My error") },
            on_sync_drop: || {
                called.store(true, Ordering::SeqCst);
            },
        });
        assert_eq!(Err("My error"), obj.async_drop().await);
        assert_eq!(true, called.load(Ordering::SeqCst));
    }
}
