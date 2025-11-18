use std::fmt::Debug;

use crate::async_drop::AsyncDrop;

mod dropping;
mod immediate_drop_request;
mod loaded;
mod loading;
mod waiter;

pub enum EntryState<V>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
{
    Loading(EntryStateLoading<V>),
    Loaded(EntryStateLoaded<V>),
    Dropping(EntryStateDropping),
}

pub use crate::concurrent_store::entry::{
    dropping::EntryStateDropping,
    loaded::EntryStateLoaded,
    loading::{EntryStateLoading, LoadingResult},
};
pub use immediate_drop_request::{ImmediateDropRequest, ImmediateDropRequestResponse};
pub use waiter::EntryLoadingWaiter;
