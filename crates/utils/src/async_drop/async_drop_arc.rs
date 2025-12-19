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

// TODO Tests
