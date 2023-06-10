use std::fmt::Debug;

use super::{AsyncDrop, AsyncDropGuard};

/// Flattens two Result values that contain AsyncDropGuards, making sure that we correctly drop things if errors happen.
pub async fn flatten_async_drop_err_map<E, T, E1, U, E2>(
    first: Result<AsyncDropGuard<T>, E1>,
    second: Result<AsyncDropGuard<U>, E2>,
    // TODO Remove err_map_t and err_map_u, they're currently only needed because we aren't using FsError everywhere yet
    err_map_t: impl FnOnce(<T as AsyncDrop>::Error) -> E,
    err_map_u: impl FnOnce(<U as AsyncDrop>::Error) -> E,
) -> Result<(AsyncDropGuard<T>, AsyncDropGuard<U>), E>
where
    T: AsyncDrop + Debug,
    U: AsyncDrop + Debug,
    E: From<E1> + From<E2>,
{
    match (first, second) {
        (Ok(first), Ok(second)) => Ok((first, second)),
        (Ok(mut first), Err(second)) => {
            // TODO Report both errors if async_drop fails
            first.async_drop().await.map_err(err_map_t)?;
            Err(second.into())
        }
        (Err(first), Ok(mut second)) => {
            // TODO Report both errors if async_drop fails
            second.async_drop().await.map_err(err_map_u)?;
            Err(first.into())
        }
        (Err(first), Err(second)) => {
            // TODO Report both errors
            Err(first.into())
        }
    }
}
