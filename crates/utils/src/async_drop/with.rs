use std::fmt::Debug;
use std::future::Future;

use super::{AsyncDrop, AsyncDropGuard};

// TODO It seems that actually most of our call sites only use sync callbacks
//      and have quite a hard time calling this because they need to wrap their callbacks into future::ready.
//      Offer sync versions instead.

// TODO Why does this need to be a macro? Can't call sites just use the function version?
#[macro_export]
macro_rules! with_async_drop_2 {
    ($value:ident, $f:block) => {
        async {
            let result = (|| async { $f })().await;
            let mut value = $value;
            value.async_drop().await?;
            result
        }
        .await
    };
}

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

// TODO Tests
