use std::fmt::Debug;

use cryfs_utils::async_drop::AsyncDrop;

mod dropping;
mod intent;
mod loaded;
mod loading;
mod waiter;

/// Represents the state of an entry in the concurrent store.
///
/// The state machine has 3 physical states:
/// - Loading: Entry is being loaded
/// - Loaded: Entry is loaded and available
/// - Dropping: Entry is being dropped (async drop in progress)
///
/// Each state can have an optional `intent` (or `reload` for Dropping) that indicates
/// future operations to perform. See [Intent] and [ReloadInfo] for details.
pub enum EntryState<V, E>
where
    V: AsyncDrop + Debug + Send + 'static,
    E: Clone + Debug + Send + Sync + 'static,
{
    Loading(EntryStateLoading<V, E>),
    Loaded(EntryStateLoaded<V, E>),
    Dropping(EntryStateDropping<V, E>),
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
        }
    }
}

pub use crate::entry::{
    dropping::EntryStateDropping,
    intent::{Intent, ReloadInfo, RequestImmediateDropResponse},
    loaded::EntryStateLoaded,
    loading::{EntryStateLoading, LoadingResult},
};
pub use waiter::EntryLoadingWaiter;
