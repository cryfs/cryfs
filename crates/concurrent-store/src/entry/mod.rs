use std::fmt::Debug;

use cryfs_utils::async_drop::AsyncDrop;

mod dropping;
mod dropping_then_loading;
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
    Dropping(EntryStateDropping<V>),
    DroppingThenLoading(EntryStateDroppingThenLoading<V, E>),
}

impl<V, E> Debug for EntryState<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryState::Loading(l) => f.debug_tuple("Loading").field(l).finish(),
            EntryState::Loaded(l) => f.debug_tuple("Loaded").field(l).finish(),
            EntryState::Dropping(d) => f.debug_tuple("Dropping").field(d).finish(),
            EntryState::DroppingThenLoading(dtl) => {
                f.debug_tuple("DroppingThenLoading").field(dtl).finish()
            }
        }
    }
}

pub use crate::entry::{
    dropping::EntryStateDropping,
    dropping_then_loading::EntryStateDroppingThenLoading,
    loaded::EntryStateLoaded,
    loading::{EntryStateLoading, LoadingResult},
};
pub use immediate_drop_request::{ImmediateDropRequest, ImmediateDropRequestResponse};
pub use waiter::EntryLoadingWaiter;
