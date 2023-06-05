use std::fmt::Debug;
use std::future::Future;

use super::{AsyncDrop, AsyncDropGuard};

pub async fn with_async_drop<T, R, E, F>(
    mut value: AsyncDropGuard<T>,
    f: impl FnOnce(&mut T) -> F,
) -> Result<R, E>
where
    T: AsyncDrop + Debug,
    E: From<<T as AsyncDrop>::Error>,
    F: Future<Output = Result<R, E>>,
{
    let result = f(&mut value).await;
    value.async_drop().await?;
    result
}

pub async fn with_async_drop_err_map<T, R, E, F>(
    mut value: AsyncDropGuard<T>,
    f: impl FnOnce(&mut T) -> F + 'static,
    err_map: impl FnOnce(<T as AsyncDrop>::Error) -> E,
) -> Result<R, E>
where
    T: AsyncDrop + Debug,
    F: Future<Output = Result<R, E>>,
{
    let result = f(&mut value).await;
    value.async_drop().await.map_err(err_map)?;
    result
}

// TODO Tests
