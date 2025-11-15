use async_trait::async_trait;
use std::fmt::Debug;

use crate::async_drop::{AsyncDrop, AsyncDropGuard};

#[derive(Debug)]
pub struct AsyncDropResult<T, E>
where
    T: Debug + AsyncDrop + Send,
    E: Debug + Send,
{
    v: Result<AsyncDropGuard<T>, E>,
}

impl<T, E> AsyncDropResult<T, E>
where
    T: Debug + AsyncDrop + Send,
    E: Debug + Send,
{
    pub fn new(v: Result<AsyncDropGuard<T>, E>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self { v })
    }
}

#[async_trait]
impl<T, E> AsyncDrop for AsyncDropResult<T, E>
where
    T: Debug + AsyncDrop + Send,
    E: Debug + Send,
{
    type Error = <T as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        match &mut self.v {
            Ok(v) => v.async_drop().await,
            Err(_) => Ok(()),
        }
    }
}
