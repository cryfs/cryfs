use async_trait::async_trait;
use futures::future::BoxFuture;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::{AsyncDrop, AsyncDropGuard};

#[derive(Debug)]
pub struct AsyncDropArc<T: AsyncDrop + Debug + Send> {
    // Always Some except during destruction
    v: Option<Arc<AsyncDropGuard<T>>>,
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

// TODO Tests
