use std::fmt::Debug;

use super::{AsyncDrop, AsyncDropGuard};

/// Flattens two Result values that contain AsyncDropGuards, making sure that we correctly drop things if errors happen.
pub async fn flatten_async_drop<E, T, E1, U, E2>(
    first: Result<AsyncDropGuard<T>, E1>,
    second: Result<AsyncDropGuard<U>, E2>,
) -> Result<(AsyncDropGuard<T>, AsyncDropGuard<U>), E>
where
    T: AsyncDrop + Debug,
    U: AsyncDrop + Debug,
    E: From<E1> + From<E2> + From<<T as AsyncDrop>::Error> + From<<U as AsyncDrop>::Error>,
{
    match (first, second) {
        (Ok(first), Ok(second)) => Ok((first, second)),
        (Ok(mut first), Err(second)) => {
            // TODO Report both errors if async_drop fails
            first.async_drop().await?;
            Err(second.into())
        }
        (Err(first), Ok(mut second)) => {
            // TODO Report both errors if async_drop fails
            second.async_drop().await?;
            Err(first.into())
        }
        (Err(first), Err(_second)) => {
            // TODO Report both errors
            Err(first.into())
        }
    }
}
