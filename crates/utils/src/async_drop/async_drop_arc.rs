use async_trait::async_trait;
use futures::future::BoxFuture;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::{Arc, Weak};

use super::{AsyncDrop, AsyncDropGuard};

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
    pub fn new(v: AsyncDropGuard<T>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            v: Some(Arc::new(v)),
        })
    }

    pub fn clone(this: &AsyncDropGuard<Self>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            v: this.v.as_ref().map(Arc::clone),
        })
    }

    pub fn strong_count(this: &AsyncDropGuard<Self>) -> usize {
        Arc::strong_count(this.v.as_ref().expect("Already dropped"))
    }

    pub fn into_inner(this: AsyncDropGuard<Self>) -> Option<AsyncDropGuard<T>> {
        let v = this
            .unsafe_into_inner_dont_drop()
            .v
            .expect("Already dropped");
        Arc::into_inner(v)
    }

    pub fn as_ptr(&self) -> *const AsyncDropGuard<T> {
        Arc::as_ptr(self.v.as_ref().expect("Already dropped"))
    }

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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    #[derive(Debug)]
    struct TestValue {
        dropped: Arc<AtomicBool>,
        value: i32,
    }

    impl TestValue {
        fn new(dropped: Arc<AtomicBool>) -> Self {
            Self { dropped, value: 42 }
        }

        fn with_value(dropped: Arc<AtomicBool>, value: i32) -> Self {
            Self { dropped, value }
        }
    }

    #[async_trait]
    impl AsyncDrop for TestValue {
        type Error = ();

        async fn async_drop_impl(&mut self) -> Result<(), ()> {
            self.dropped.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[derive(Debug)]
    struct FailingTestValue {
        dropped: Arc<AtomicBool>,
    }

    #[async_trait]
    impl AsyncDrop for FailingTestValue {
        type Error = &'static str;

        async fn async_drop_impl(&mut self) -> Result<(), &'static str> {
            self.dropped.store(true, Ordering::SeqCst);
            Err("async drop failed")
        }
    }

    // ==================== AsyncDropArc Basic Tests ====================

    #[tokio::test]
    async fn new_creates_arc_with_strong_count_one() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        assert_eq!(AsyncDropArc::strong_count(&arc), 1);

        let mut arc = arc;
        arc.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn async_drop_calls_inner_async_drop() {
        let dropped = Arc::new(AtomicBool::new(false));
        let mut arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        assert!(!dropped.load(Ordering::SeqCst));
        arc.async_drop().await.unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn async_drop_propagates_error() {
        let dropped = Arc::new(AtomicBool::new(false));
        let mut arc = AsyncDropArc::new(AsyncDropGuard::new(FailingTestValue {
            dropped: dropped.clone(),
        }));

        let result = arc.async_drop().await;
        assert_eq!(result, Err("async drop failed"));
        assert!(dropped.load(Ordering::SeqCst)); // Still called even though it returns error
    }

    // ==================== Clone Tests ====================

    #[tokio::test]
    async fn clone_increments_strong_count() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        assert_eq!(AsyncDropArc::strong_count(&arc), 1);

        let clone1 = AsyncDropArc::clone(&arc);
        assert_eq!(AsyncDropArc::strong_count(&arc), 2);
        assert_eq!(AsyncDropArc::strong_count(&clone1), 2);

        let clone2 = AsyncDropArc::clone(&arc);
        assert_eq!(AsyncDropArc::strong_count(&arc), 3);

        // Clean up
        let mut arc = arc;
        let mut clone1 = clone1;
        let mut clone2 = clone2;
        arc.async_drop().await.unwrap();
        clone1.async_drop().await.unwrap();
        clone2.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn clone_shares_same_value() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));
        let clone = AsyncDropArc::clone(&arc);

        assert!(AsyncDropArc::ptr_eq(&arc, &clone));

        let mut arc = arc;
        let mut clone = clone;
        arc.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn dropping_clone_decrements_strong_count() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));
        let clone = AsyncDropArc::clone(&arc);

        assert_eq!(AsyncDropArc::strong_count(&arc), 2);

        let mut clone = clone;
        clone.async_drop().await.unwrap();

        assert_eq!(AsyncDropArc::strong_count(&arc), 1);
        assert!(!dropped.load(Ordering::SeqCst)); // Not dropped yet

        let mut arc = arc;
        arc.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn last_clone_dropped_triggers_async_drop() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));
        let clone1 = AsyncDropArc::clone(&arc);
        let clone2 = AsyncDropArc::clone(&arc);

        // Drop first two - should not trigger async_drop
        let mut arc = arc;
        arc.async_drop().await.unwrap();
        assert!(!dropped.load(Ordering::SeqCst));

        let mut clone1 = clone1;
        clone1.async_drop().await.unwrap();
        assert!(!dropped.load(Ordering::SeqCst));

        // Drop last one - should trigger async_drop
        let mut clone2 = clone2;
        clone2.async_drop().await.unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    // ==================== Async Drop Timing Tests ====================

    #[tokio::test]
    async fn async_drop_not_called_while_strong_refs_remain() {
        let drop_count = Arc::new(AtomicUsize::new(0));
        let dropped = Arc::new(AtomicBool::new(false));

        #[derive(Debug)]
        struct CountingValue {
            drop_count: Arc<AtomicUsize>,
            dropped: Arc<AtomicBool>,
        }

        #[async_trait]
        impl AsyncDrop for CountingValue {
            type Error = ();
            async fn async_drop_impl(&mut self) -> Result<(), ()> {
                self.drop_count.fetch_add(1, Ordering::SeqCst);
                self.dropped.store(true, Ordering::SeqCst);
                Ok(())
            }
        }

        let arc = AsyncDropArc::new(AsyncDropGuard::new(CountingValue {
            drop_count: drop_count.clone(),
            dropped: dropped.clone(),
        }));

        // Create multiple clones
        let clone1 = AsyncDropArc::clone(&arc);
        let clone2 = AsyncDropArc::clone(&arc);
        let clone3 = AsyncDropArc::clone(&arc);

        assert_eq!(drop_count.load(Ordering::SeqCst), 0);

        // Drop clones one by one
        let mut arc = arc;
        arc.async_drop().await.unwrap();
        assert_eq!(drop_count.load(Ordering::SeqCst), 0);

        let mut clone1 = clone1;
        clone1.async_drop().await.unwrap();
        assert_eq!(drop_count.load(Ordering::SeqCst), 0);

        let mut clone2 = clone2;
        clone2.async_drop().await.unwrap();
        assert_eq!(drop_count.load(Ordering::SeqCst), 0);

        // Last one triggers the drop
        let mut clone3 = clone3;
        clone3.async_drop().await.unwrap();
        assert_eq!(drop_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn async_drop_called_exactly_once_when_last_ref_dropped() {
        let drop_count = Arc::new(AtomicUsize::new(0));

        #[derive(Debug)]
        struct CountingValue {
            drop_count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl AsyncDrop for CountingValue {
            type Error = ();
            async fn async_drop_impl(&mut self) -> Result<(), ()> {
                self.drop_count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let arc = AsyncDropArc::new(AsyncDropGuard::new(CountingValue {
            drop_count: drop_count.clone(),
        }));
        let clone = AsyncDropArc::clone(&arc);

        let mut arc = arc;
        let mut clone = clone;

        arc.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();

        assert_eq!(drop_count.load(Ordering::SeqCst), 1);
    }

    // ==================== into_inner Tests ====================

    #[tokio::test]
    async fn into_inner_returns_some_when_only_reference() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        let inner = AsyncDropArc::into_inner(arc);
        assert!(inner.is_some());

        let mut inner = inner.unwrap();
        inner.async_drop().await.unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn into_inner_returns_none_when_multiple_references() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));
        let clone = AsyncDropArc::clone(&arc);

        let inner = AsyncDropArc::into_inner(arc);
        assert!(inner.is_none());

        // Clone still works
        assert_eq!(AsyncDropArc::strong_count(&clone), 1);

        let mut clone = clone;
        clone.async_drop().await.unwrap();
        assert!(dropped.load(Ordering::SeqCst));
    }

    // ==================== Pointer Operations Tests ====================

    #[tokio::test]
    async fn ptr_eq_returns_true_for_clones() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));
        let clone = AsyncDropArc::clone(&arc);

        assert!(AsyncDropArc::ptr_eq(&arc, &clone));

        let mut arc = arc;
        let mut clone = clone;
        arc.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn ptr_eq_returns_false_for_different_arcs() {
        let dropped1 = Arc::new(AtomicBool::new(false));
        let dropped2 = Arc::new(AtomicBool::new(false));
        let arc1 = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped1.clone())));
        let arc2 = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped2.clone())));

        assert!(!AsyncDropArc::ptr_eq(&arc1, &arc2));

        let mut arc1 = arc1;
        let mut arc2 = arc2;
        arc1.async_drop().await.unwrap();
        arc2.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn as_ptr_returns_consistent_pointer() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));
        let clone = AsyncDropArc::clone(&arc);

        let ptr1 = arc.as_ptr();
        let ptr2 = clone.as_ptr();
        assert_eq!(ptr1, ptr2);

        let mut arc = arc;
        let mut clone = clone;
        arc.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();
    }

    // ==================== Deref Tests ====================

    #[tokio::test]
    async fn deref_provides_access_to_inner_value() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::with_value(
            dropped.clone(),
            123,
        )));

        assert_eq!(arc.value, 123);

        let mut arc = arc;
        arc.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn borrow_provides_access_to_inner_value() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::with_value(
            dropped.clone(),
            456,
        )));

        let borrowed: &TestValue = arc.borrow();
        assert_eq!(borrowed.value, 456);

        let mut arc = arc;
        arc.async_drop().await.unwrap();
    }

    // ==================== AsyncDropWeak Tests ====================

    #[tokio::test]
    async fn downgrade_and_upgrade_works() {
        let dropped = Arc::new(AtomicBool::new(false));
        let mut arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        let weak = AsyncDropArc::downgrade(&arc);
        assert_eq!(weak.strong_count(), 1);

        let mut upgraded = weak.upgrade().expect("upgrade should succeed");
        assert_eq!(weak.strong_count(), 2);

        upgraded.async_drop().await.unwrap();
        assert!(!dropped.load(Ordering::SeqCst)); // Not dropped yet, original arc still exists

        arc.async_drop().await.unwrap();
        assert!(dropped.load(Ordering::SeqCst)); // Now it's dropped
    }

    #[tokio::test]
    async fn upgrade_returns_none_after_strong_ref_dropped() {
        let dropped = Arc::new(AtomicBool::new(false));
        let mut arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        let weak = AsyncDropArc::downgrade(&arc);

        // Upgrade and check it works, then drop
        let mut upgraded = weak.upgrade().expect("should succeed");
        upgraded.async_drop().await.unwrap();

        // Create another upgraded value
        let mut upgraded2 = weak.upgrade().expect("original still exists");
        upgraded2.async_drop().await.unwrap();

        assert!(!dropped.load(Ordering::SeqCst)); // Original arc still exists

        // Drop the original
        arc.async_drop().await.unwrap();

        // Now upgrade should return None
        assert!(weak.upgrade().is_none());
        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn weak_clone_works() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

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
        let dropped = Arc::new(AtomicBool::new(false));
        let mut arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        let weak = AsyncDropArc::downgrade(&arc);
        assert_eq!(weak.strong_count(), 1);

        // Drop the only strong reference - weak should not prevent async_drop
        arc.async_drop().await.unwrap();

        assert!(dropped.load(Ordering::SeqCst));
        assert_eq!(weak.strong_count(), 0);
        assert!(weak.upgrade().is_none());
    }

    #[tokio::test]
    async fn multiple_weak_refs_can_all_upgrade() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        let weak1 = AsyncDropArc::downgrade(&arc);
        let weak2 = AsyncDropArc::downgrade(&arc);
        let weak3 = weak1.clone();

        // All can upgrade simultaneously
        let mut upgraded1 = weak1.upgrade().expect("should succeed");
        let mut upgraded2 = weak2.upgrade().expect("should succeed");
        let mut upgraded3 = weak3.upgrade().expect("should succeed");

        assert_eq!(weak1.strong_count(), 4); // original + 3 upgraded

        // All point to same value
        assert!(AsyncDropArc::ptr_eq(&arc, &upgraded1));
        assert!(AsyncDropArc::ptr_eq(&arc, &upgraded2));
        assert!(AsyncDropArc::ptr_eq(&arc, &upgraded3));

        // Clean up
        let mut arc = arc;
        arc.async_drop().await.unwrap();
        upgraded1.async_drop().await.unwrap();
        upgraded2.async_drop().await.unwrap();
        upgraded3.async_drop().await.unwrap();

        assert!(dropped.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn upgraded_weak_keeps_value_alive() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

        let weak = AsyncDropArc::downgrade(&arc);
        let upgraded = weak.upgrade().expect("should succeed");

        // Drop original
        let mut arc = arc;
        arc.async_drop().await.unwrap();

        // Value should still be alive because upgraded holds a strong ref
        assert!(!dropped.load(Ordering::SeqCst));
        assert_eq!(weak.strong_count(), 1);

        // Can still upgrade
        let mut another_upgraded = weak.upgrade().expect("upgraded still holds strong ref");
        another_upgraded.async_drop().await.unwrap();

        // Now drop the last strong ref
        let mut upgraded = upgraded;
        upgraded.async_drop().await.unwrap();

        assert!(dropped.load(Ordering::SeqCst));
        assert!(weak.upgrade().is_none());
    }

    #[tokio::test]
    async fn weak_count_accurate_with_multiple_refs() {
        let dropped = Arc::new(AtomicBool::new(false));
        let arc = AsyncDropArc::new(AsyncDropGuard::new(TestValue::new(dropped.clone())));

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
