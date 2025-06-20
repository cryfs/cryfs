use async_trait::async_trait;
use std::fmt::Debug;
use tokio::sync::{Mutex, MutexGuard};

use crate::async_drop::{AsyncDrop, AsyncDropGuard};

#[derive(Debug)]
// TODO Why do we need Send? tokio::sync::Mutex doesn't seem to need it
pub struct AsyncDropTokioMutex<T: AsyncDrop + Debug + Send> {
    // Always Some except during destruction
    v: Option<Mutex<AsyncDropGuard<T>>>,
}

impl<T: AsyncDrop + Debug + Send> AsyncDropTokioMutex<T> {
    pub fn new(v: AsyncDropGuard<T>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            v: Some(Mutex::new(v)),
        })
    }

    pub async fn lock(&self) -> MutexGuard<'_, AsyncDropGuard<T>> {
        self.v.as_ref().expect("Already destructed").lock().await
    }

    pub fn into_inner(this: AsyncDropGuard<Self>) -> AsyncDropGuard<T> {
        let inner = this
            .unsafe_into_inner_dont_drop()
            .v
            .take()
            .expect("Already destructed");
        inner.into_inner()
    }
}

#[async_trait]
impl<T: AsyncDrop + Debug + Send> AsyncDrop for AsyncDropTokioMutex<T> {
    type Error = T::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let v = self.v.take().expect("Already destructed");
        let mut v = v.into_inner();
        v.async_drop().await?;
        Ok(())
    }
}
