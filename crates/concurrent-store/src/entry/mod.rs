use std::fmt::Debug;

use cryfs_utils::async_drop::AsyncDrop;

mod dropping;
mod immediate_drop_request;
mod loaded;
mod loading;
mod waiter;

pub enum EntryState<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    Loading(EntryStateLoading<V, E>),
    Loaded(EntryStateLoaded<V>),
    Dropping(EntryStateDropping),
}

pub use crate::entry::{
    dropping::EntryStateDropping,
    loaded::EntryStateLoaded,
    loading::{EntryStateLoading, LoadingResult},
};
pub use immediate_drop_request::{ImmediateDropRequest, ImmediateDropRequestResponse};
pub use waiter::EntryLoadingWaiter;
