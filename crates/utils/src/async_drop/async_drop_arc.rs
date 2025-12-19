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
use std::sync::{Arc, Weak};

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

/// A weak reference to an `AsyncDropArc<T>`.
///
/// Does not prevent the value from being dropped. When upgraded,
/// returns an `AsyncDropArc` to ensure proper async drop semantics.
#[derive(Debug)]
pub struct AsyncDropWeak<T: AsyncDrop + Debug + Send> {
    v: Weak<AsyncDropGuard<T>>,
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

    /// Creates a weak reference to this `AsyncDropArc`.
    pub fn downgrade(this: &AsyncDropGuard<Self>) -> AsyncDropWeak<T> {
        AsyncDropWeak {
            v: Arc::downgrade(this.v.as_ref().expect("Already destructed")),
        }
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

impl<T: AsyncDrop + Debug + Send> AsyncDropWeak<T> {
    /// Attempts to upgrade the weak reference to an `AsyncDropArc`.
    ///
    /// Returns `None` if the inner value has already been dropped,
    /// or `Some(AsyncDropGuard<AsyncDropArc<T>>)` if there are still
    /// strong references.
    pub fn upgrade(&self) -> Option<AsyncDropGuard<AsyncDropArc<T>>> {
        self.v
            .upgrade()
            .map(|arc| AsyncDropGuard::new(AsyncDropArc { v: Some(arc) }))
    }

    /// Returns the number of strong references to the value.
    pub fn strong_count(&self) -> usize {
        self.v.strong_count()
    }

    /// Returns the number of weak references to the value.
    pub fn weak_count(&self) -> usize {
        self.v.weak_count()
    }
}

impl<T: AsyncDrop + Debug + Send> Clone for AsyncDropWeak<T> {
    fn clone(&self) -> Self {
        AsyncDropWeak { v: self.v.clone() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::borrow::Borrow;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestValue {
        drop_counter: Arc<AtomicUsize>,
        value: i32,
    }

    impl TestValue {
        fn new(drop_counter: Arc<AtomicUsize>) -> AsyncDropGuard<Self> {
            Self::with_value(drop_counter, 42)
        }

        fn with_value(drop_counter: Arc<AtomicUsize>, value: i32) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self {
                drop_counter,
                value,
            })
        }
    }

    #[async_trait]
    impl AsyncDrop for TestValue {
        type Error = ();

        async fn async_drop_impl(&mut self) -> Result<(), ()> {
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct FailingTestValue {
        drop_counter: Arc<AtomicUsize>,
    }

    impl FailingTestValue {
        fn new(drop_counter: Arc<AtomicUsize>) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self { drop_counter })
        }
    }

    #[async_trait]
    impl AsyncDrop for FailingTestValue {
        type Error = &'static str;

        async fn async_drop_impl(&mut self) -> Result<(), &'static str> {
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
            Err("async drop failed")
        }
    }

    // ==================== AsyncDropArc Basic Tests ====================

    #[tokio::test]
    async fn new_creates_arc_with_strong_count_one() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(TestValue::with_value(counter.clone(), 42));

        assert_eq!(AsyncDropArc::strong_count(&arc), 1);
        assert_eq!(42, arc.value);

        arc.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn async_drop_calls_inner_async_drop_exactly_once() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        assert_eq!(0, counter.load(Ordering::SeqCst));
        arc.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn async_drop_propagates_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(FailingTestValue::new(counter.clone()));

        let result = arc.async_drop().await;
        assert_eq!(result, Err("async drop failed"));
        // async_drop_impl was still called even though it returned an error
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    // ==================== Clone Tests ====================

    #[tokio::test]
    async fn clone_increments_strong_count() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        assert_eq!(AsyncDropArc::strong_count(&arc), 1);

        let clone1 = AsyncDropArc::clone(&arc);
        assert_eq!(AsyncDropArc::strong_count(&arc), 2);
        assert_eq!(AsyncDropArc::strong_count(&clone1), 2);

        let clone2 = AsyncDropArc::clone(&arc);
        assert_eq!(AsyncDropArc::strong_count(&arc), 3);

        let mut arc = arc;
        let mut clone1 = clone1;
        let mut clone2 = clone2;
        arc.async_drop().await.unwrap();
        clone1.async_drop().await.unwrap();
        clone2.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn clone_shares_same_value() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::with_value(counter.clone(), 42));
        let clone = AsyncDropArc::clone(&arc);

        // Both references can read the shared value via Deref
        assert_eq!(42, arc.value);
        assert_eq!(42, clone.value);

        // ... and point to the same allocation
        assert!(AsyncDropArc::ptr_eq(&arc, &clone));

        let mut arc = arc;
        let mut clone = clone;
        arc.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();

        // Inner async_drop runs exactly once even though we dropped two arcs
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn dropping_clone_decrements_strong_count() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));
        let clone = AsyncDropArc::clone(&arc);

        assert_eq!(AsyncDropArc::strong_count(&arc), 2);

        let mut clone = clone;
        clone.async_drop().await.unwrap();

        assert_eq!(AsyncDropArc::strong_count(&arc), 1);
        assert_eq!(0, counter.load(Ordering::SeqCst)); // Not dropped yet

        let mut arc = arc;
        arc.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn last_clone_dropped_triggers_async_drop() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));
        let clone1 = AsyncDropArc::clone(&arc);
        let clone2 = AsyncDropArc::clone(&arc);

        // Drop first two - should not trigger async_drop_impl
        let mut arc = arc;
        arc.async_drop().await.unwrap();
        assert_eq!(0, counter.load(Ordering::SeqCst));

        let mut clone1 = clone1;
        clone1.async_drop().await.unwrap();
        assert_eq!(0, counter.load(Ordering::SeqCst));

        // Drop last one - should trigger async_drop_impl exactly once
        let mut clone2 = clone2;
        clone2.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn async_drop_not_called_while_strong_refs_remain() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let clone1 = AsyncDropArc::clone(&arc);
        let clone2 = AsyncDropArc::clone(&arc);
        let clone3 = AsyncDropArc::clone(&arc);

        assert_eq!(counter.load(Ordering::SeqCst), 0);

        let mut arc = arc;
        arc.async_drop().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        let mut clone1 = clone1;
        clone1.async_drop().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        let mut clone2 = clone2;
        clone2.async_drop().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 0);

        // Last one triggers the drop
        let mut clone3 = clone3;
        clone3.async_drop().await.unwrap();
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    // ==================== into_inner Tests ====================

    #[tokio::test]
    async fn into_inner_returns_some_when_only_reference() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::with_value(counter.clone(), 42));

        let mut inner =
            AsyncDropArc::into_inner(arc).expect("Should return inner when single ref");
        assert_eq!(42, inner.value);

        inner.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn into_inner_returns_none_when_multiple_references() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));
        let clone = AsyncDropArc::clone(&arc);

        let inner = AsyncDropArc::into_inner(arc);
        assert!(inner.is_none());

        // Failed into_inner consumed one strong ref
        assert_eq!(AsyncDropArc::strong_count(&clone), 1);

        let mut clone = clone;
        clone.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    // ==================== Pointer Operations Tests ====================

    #[tokio::test]
    async fn ptr_eq_returns_true_for_clones() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));
        let clone = AsyncDropArc::clone(&arc);

        assert!(AsyncDropArc::ptr_eq(&arc, &clone));

        let mut arc = arc;
        let mut clone = clone;
        arc.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn ptr_eq_returns_false_for_different_arcs() {
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));
        let arc1 = AsyncDropArc::new(TestValue::new(counter1.clone()));
        let arc2 = AsyncDropArc::new(TestValue::new(counter2.clone()));

        assert!(!AsyncDropArc::ptr_eq(&arc1, &arc2));

        let mut arc1 = arc1;
        let mut arc2 = arc2;
        arc1.async_drop().await.unwrap();
        arc2.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn as_ptr_returns_consistent_pointer() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));
        let clone = AsyncDropArc::clone(&arc);

        let ptr1 = arc.as_ptr();
        let ptr2 = clone.as_ptr();
        assert_eq!(ptr1, ptr2);

        let mut arc = arc;
        let mut clone = clone;
        arc.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();
    }

    // ==================== Deref / Borrow Tests ====================

    #[tokio::test]
    async fn deref_provides_access_to_inner_value() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(TestValue::with_value(counter.clone(), 123));

        assert_eq!(arc.value, 123);

        arc.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn borrow_provides_access_to_inner_value() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(TestValue::with_value(counter.clone(), 456));

        let borrowed: &TestValue = arc.borrow();
        assert_eq!(borrowed.value, 456);

        arc.async_drop().await.unwrap();
    }

    // ==================== AsyncDropWeak Tests ====================

    #[tokio::test]
    async fn downgrade_and_upgrade_works() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let weak = AsyncDropArc::downgrade(&arc);
        assert_eq!(weak.strong_count(), 1);

        let mut upgraded = weak.upgrade().expect("upgrade should succeed");
        assert_eq!(weak.strong_count(), 2);

        upgraded.async_drop().await.unwrap();
        assert_eq!(0, counter.load(Ordering::SeqCst)); // Not dropped yet, original arc still exists

        arc.async_drop().await.unwrap();
        assert_eq!(1, counter.load(Ordering::SeqCst)); // Now it's dropped
    }

    #[tokio::test]
    async fn upgrade_returns_none_after_strong_ref_dropped() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let weak = AsyncDropArc::downgrade(&arc);

        // Upgrade and drop, twice
        let mut upgraded = weak.upgrade().expect("should succeed");
        upgraded.async_drop().await.unwrap();

        let mut upgraded2 = weak.upgrade().expect("original still exists");
        upgraded2.async_drop().await.unwrap();

        assert_eq!(0, counter.load(Ordering::SeqCst)); // Original arc still exists

        arc.async_drop().await.unwrap();

        assert!(weak.upgrade().is_none());
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn weak_clone_works() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let weak1 = AsyncDropArc::downgrade(&arc);
        let weak2 = weak1.clone();

        assert_eq!(weak1.strong_count(), 1);
        assert_eq!(weak2.strong_count(), 1);
        assert_eq!(weak1.weak_count(), 2);
        assert_eq!(weak2.weak_count(), 2);

        let mut arc = arc;
        arc.async_drop().await.unwrap();

        assert!(weak1.upgrade().is_none());
        assert!(weak2.upgrade().is_none());
    }

    #[tokio::test]
    async fn weak_does_not_prevent_async_drop() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mut arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let weak = AsyncDropArc::downgrade(&arc);
        assert_eq!(weak.strong_count(), 1);

        // Drop the only strong reference - weak should not prevent async_drop
        arc.async_drop().await.unwrap();

        assert_eq!(1, counter.load(Ordering::SeqCst));
        assert_eq!(weak.strong_count(), 0);
        assert!(weak.upgrade().is_none());
    }

    #[tokio::test]
    async fn multiple_weak_refs_can_all_upgrade() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let weak1 = AsyncDropArc::downgrade(&arc);
        let weak2 = AsyncDropArc::downgrade(&arc);
        let weak3 = weak1.clone();

        // All can upgrade simultaneously
        let mut upgraded1 = weak1.upgrade().expect("should succeed");
        let mut upgraded2 = weak2.upgrade().expect("should succeed");
        let mut upgraded3 = weak3.upgrade().expect("should succeed");

        assert_eq!(weak1.strong_count(), 4); // original + 3 upgraded

        // All point to the same value
        assert!(AsyncDropArc::ptr_eq(&arc, &upgraded1));
        assert!(AsyncDropArc::ptr_eq(&arc, &upgraded2));
        assert!(AsyncDropArc::ptr_eq(&arc, &upgraded3));

        let mut arc = arc;
        arc.async_drop().await.unwrap();
        upgraded1.async_drop().await.unwrap();
        upgraded2.async_drop().await.unwrap();
        upgraded3.async_drop().await.unwrap();

        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn upgraded_weak_keeps_value_alive() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let weak = AsyncDropArc::downgrade(&arc);
        let upgraded = weak.upgrade().expect("should succeed");

        // Drop original
        let mut arc = arc;
        arc.async_drop().await.unwrap();

        // Value should still be alive because upgraded holds a strong ref
        assert_eq!(0, counter.load(Ordering::SeqCst));
        assert_eq!(weak.strong_count(), 1);

        // Can still upgrade
        let mut another_upgraded = weak.upgrade().expect("upgraded still holds strong ref");
        another_upgraded.async_drop().await.unwrap();

        // Now drop the last strong ref
        let mut upgraded = upgraded;
        upgraded.async_drop().await.unwrap();

        assert_eq!(1, counter.load(Ordering::SeqCst));
        assert!(weak.upgrade().is_none());
    }

    #[tokio::test]
    async fn weak_count_accurate_with_multiple_refs() {
        let counter = Arc::new(AtomicUsize::new(0));
        let arc = AsyncDropArc::new(TestValue::new(counter.clone()));

        let weak1 = AsyncDropArc::downgrade(&arc);
        assert_eq!(weak1.weak_count(), 1);

        let weak2 = AsyncDropArc::downgrade(&arc);
        assert_eq!(weak1.weak_count(), 2);
        assert_eq!(weak2.weak_count(), 2);

        let weak3 = weak1.clone();
        assert_eq!(weak1.weak_count(), 3);

        drop(weak2);
        assert_eq!(weak1.weak_count(), 2);

        drop(weak3);
        assert_eq!(weak1.weak_count(), 1);

        let mut arc = arc;
        arc.async_drop().await.unwrap();
    }
}
