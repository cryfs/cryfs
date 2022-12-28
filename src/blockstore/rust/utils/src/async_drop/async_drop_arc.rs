use async_trait::async_trait;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::Arc;

use super::{AsyncDrop, AsyncDropGuard};

#[derive(Debug)]
// TODO Why do we need Send + Sync? std::sync::Arc doesn't seem to need it
pub struct AsyncDropArc<T: AsyncDrop + Debug + Sync + Send> {
    // Always Some except during destruction
    v: Option<Arc<AsyncDropGuard<T>>>,
}

impl<T: AsyncDrop + Debug + Sync + Send> AsyncDropArc<T> {
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
}

#[async_trait]
impl<T: AsyncDrop + Debug + Sync + Send> AsyncDrop for AsyncDropArc<T> {
    type Error = T::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let v = self.v.take().expect("Already destructed");
        if let Ok(mut v) = Arc::try_unwrap(v) {
            v.async_drop().await?;
        }
        Ok(())
    }
}

impl<T: AsyncDrop + Debug + Sync + Send> Deref for AsyncDropArc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.v.as_ref().expect("Already destructed").deref()
    }
}

impl<T: AsyncDrop + Debug + Sync + Send> Borrow<T> for AsyncDropArc<T> {
    fn borrow(&self) -> &T {
        Borrow::<AsyncDropGuard<T>>::borrow(Borrow::<Arc<AsyncDropGuard<T>>>::borrow(
            self.v.as_ref().expect("Already destructed"),
        ))
        .borrow()
    }
}

// TODO Tests
