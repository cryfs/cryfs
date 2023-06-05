use std::fmt::Debug;
use std::future::Future;

use super::{AsyncDrop, AsyncDropGuard};

// TODO It seems that actually most of our call sites only use sync callbacks
//      and have quite a hard time calling this because they need to wrap their callbacks into future::ready.
//      Offer sync versions instead.

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
    <F as Future>::Output: 'static,
    E: 'static,
    R: 'static,
{
    let result = f(&mut value).await;
    value.async_drop().await.map_err(err_map)?;
    result
}

// TODO Tests
