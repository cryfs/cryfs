//! Arc wrapper for types that implement [`AsyncDrop`].
//!
//! This module provides [`AsyncDropArc`], which wraps an [`AsyncDropGuard<T>`] in an
//! [`Arc`] for shared ownership. The async drop is only called when the last reference
//! is dropped.

use async_trait::async_trait;
use futures::future::BoxFuture;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::{AsyncDrop, AsyncDropGuard};

/// A reference-counted wrapper for types that implement [`AsyncDrop`].
///
/// `AsyncDropArc` provides shared ownership of a value that requires async cleanup.
/// The contained value's [`AsyncDrop::async_drop_impl`] is only called when the last
/// `AsyncDropArc` reference is dropped via [`AsyncDropGuard::async_drop`].
///
/// Unlike [`std::sync::Arc`], cloning is done via the [`AsyncDropArc::clone`] associated
/// function rather than the `Clone` trait, because each clone is wrapped in its own
/// [`AsyncDropGuard`].
#[derive(Debug)]
pub struct AsyncDropArc<T: AsyncDrop + Debug + Send> {
    // Always Some except during destruction
    v: Option<Arc<AsyncDropGuard<T>>>,
}

impl<T: AsyncDrop + Debug + Send> AsyncDropArc<T> {
    /// Creates a new `AsyncDropArc` wrapping the given value.
    ///
    /// The returned guard must have `async_drop()` called on it before being dropped.
    pub fn new(v: AsyncDropGuard<T>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            v: Some(Arc::new(v)),
        })
    }

    /// Creates a new reference to the same underlying value.
    ///
    /// This is equivalent to `Arc::clone` but returns an `AsyncDropGuard` that
    /// must have `async_drop()` called on it.
    pub fn clone(this: &AsyncDropGuard<Self>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            v: this.v.as_ref().map(Arc::clone),
        })
    }

    /// Returns the number of strong references to the underlying value.
    pub fn strong_count(this: &AsyncDropGuard<Self>) -> usize {
        Arc::strong_count(this.v.as_ref().expect("Already dropped"))
    }

    /// Attempts to extract the inner `AsyncDropGuard<T>` if this is the only reference.
    ///
    /// Returns `Some` if this is the last reference, `None` otherwise.
    /// This consumes the `AsyncDropArc` without calling its async drop.
    pub fn into_inner(this: AsyncDropGuard<Self>) -> Option<AsyncDropGuard<T>> {
        let v = this
            .unsafe_into_inner_dont_drop()
            .v
            .expect("Already dropped");
        Arc::into_inner(v)
    }

    /// Returns a raw pointer to the underlying `AsyncDropGuard<T>`.
    pub fn as_ptr(&self) -> *const AsyncDropGuard<T> {
        Arc::as_ptr(self.v.as_ref().expect("Already dropped"))
    }

    /// Returns `true` if both `AsyncDropArc`s point to the same allocation.
    pub fn ptr_eq(a: &AsyncDropGuard<Self>, b: &AsyncDropGuard<Self>) -> bool {
        let lhs = a.v.as_ref().expect("Already dropped");
        let rhs = b.v.as_ref().expect("Already dropped");
        Arc::ptr_eq(lhs, rhs)
    }
}

#[async_trait]
impl<T: AsyncDrop + Debug + Send> AsyncDrop for AsyncDropArc<T> {
    type Error = T::Error;

    fn async_drop_impl<'s, 'async_trait>(
        &'s mut self,
    ) -> BoxFuture<'async_trait, Result<(), Self::Error>>
    where
        's: 'async_trait,
        Self: 'async_trait,
    {
        let v = self.v.take().expect("Already destructed");
        if let Some(mut v) = Arc::into_inner(v) {
            Box::pin(async move { v.async_drop().await })
        } else {
            Box::pin(async { Ok(()) })
        }
    }
}

impl<T: AsyncDrop + Debug + Send> Deref for AsyncDropArc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.v.as_ref().expect("Already destructed").deref()
    }
}

impl<T: AsyncDrop + Debug + Send> Borrow<T> for AsyncDropArc<T> {
    fn borrow(&self) -> &T {
        Borrow::<AsyncDropGuard<T>>::borrow(Borrow::<Arc<AsyncDropGuard<T>>>::borrow(
            self.v.as_ref().expect("Already destructed"),
        ))
        .borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
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
    async fn test_new_creates_guard() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut arc = AsyncDropArc::new(inner);

        assert_eq!(42, arc.value);
        arc.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_clone_shares_underlying_value() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let arc1 = AsyncDropArc::new(inner);
        let arc2 = AsyncDropArc::clone(&arc1);

        // Both should see the same value
        assert_eq!(42, arc1.value);
        assert_eq!(42, arc2.value);

        // Should point to the same allocation
        assert!(AsyncDropArc::ptr_eq(&arc1, &arc2));

        // Clean up - drop both arcs via async_drop
        // The actual value is only dropped when the last reference is dropped
        let mut arc1 = arc1;
        let mut arc2 = arc2;
        arc1.async_drop().await.unwrap();
        arc2.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_strong_count() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let arc1 = AsyncDropArc::new(inner);

        assert_eq!(1, AsyncDropArc::strong_count(&arc1));

        let arc2 = AsyncDropArc::clone(&arc1);
        assert_eq!(2, AsyncDropArc::strong_count(&arc1));
        assert_eq!(2, AsyncDropArc::strong_count(&arc2));

        // Drop one clone
        let mut arc2 = arc2;
        arc2.async_drop().await.unwrap();

        assert_eq!(1, AsyncDropArc::strong_count(&arc1));

        // Drop the last one
        let mut arc1 = arc1;
        arc1.async_drop().await.unwrap();

        // async_drop should have been called exactly once (on the last reference)
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_into_inner_when_single_reference() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let arc = AsyncDropArc::new(inner);

        let mut inner = AsyncDropArc::into_inner(arc).expect("Should return inner when single ref");
        assert_eq!(42, inner.value);

        inner.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_into_inner_when_multiple_references_returns_none() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let arc1 = AsyncDropArc::new(inner);
        let mut arc2 = AsyncDropArc::clone(&arc1);

        // Should return None when there are multiple references
        let result = AsyncDropArc::into_inner(arc1);
        assert!(result.is_none());

        // Clean up the remaining reference
        arc2.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_async_drop_called_on_last_reference() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let arc1 = AsyncDropArc::new(inner);
        let mut arc2 = AsyncDropArc::clone(&arc1);
        let mut arc3 = AsyncDropArc::clone(&arc1);
        let mut arc1 = arc1;

        // Drop first two - should not call async_drop_impl
        arc1.async_drop().await.unwrap();
        assert_eq!(0, counter.load(Ordering::SeqCst));

        arc2.async_drop().await.unwrap();
        assert_eq!(0, counter.load(Ordering::SeqCst));

        // Drop last one - should call async_drop_impl
        arc3.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_deref() {
        let counter = Arc::new(AtomicUsize::new(0));
        let inner = TestValue::new(42, Arc::clone(&counter));
        let mut arc = AsyncDropArc::new(inner);

        // Test Deref
        assert_eq!(42, arc.value);

        arc.async_drop().await.unwrap();
    }
}
