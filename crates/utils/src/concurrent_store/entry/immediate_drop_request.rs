use futures::{
    FutureExt as _,
    future::{BoxFuture, Shared},
};
use std::fmt::Debug;

use crate::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    event::Event,
};

/// Represents a request to immediately drop an entry that is currently loading or loaded.
pub enum ImmediateDropRequest<V>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
{
    /// No immediate drop has been requested.
    NotRequested,
    /// An immediate drop has been requested.
    Requested {
        /// The entry gets sent here once all other references to it are gone and we have exclusive access to it.
        /// This function is then expected to drop it.
        // TODO No Box<dyn> but impl Fn?
        drop_fn: Box<dyn FnOnce(Option<AsyncDropGuard<V>>) -> BoxFuture<'static, ()> + Send + Sync>,
        /// Sender to notify the requester when the drop is complete.
        on_dropped: Event,
    },
}

impl<V> ImmediateDropRequest<V>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
{
    /// Request an immediate drop of the entry.
    /// If an immediate drop has already been requested, returns a receiver to wait for the completion of that request.
    /// If no immediate drop has been requested yet, sets up the request with the provided drop function and returns a receiver to wait for its completion.
    pub fn request_immediate_drop_if_not_yet_requested<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> ImmediateDropRequestResponse
    where
        F: Future<Output = ()> + Send,
    {
        match self {
            ImmediateDropRequest::Requested { on_dropped, .. } => {
                let on_dropped = on_dropped.clone();
                ImmediateDropRequestResponse::NotRequestedBecauseItWasAlreadyRequestedEarlier {
                    on_earlier_request_complete: async move { on_dropped.wait().await }
                        .boxed()
                        .shared(),
                }
            }
            ImmediateDropRequest::NotRequested => {
                let on_dropped = Event::new();
                let on_dropped_clone = on_dropped.clone();
                *self = ImmediateDropRequest::Requested {
                    drop_fn: Box::new(move |i| {
                        async move {
                            drop_fn(i).await;
                            on_dropped_clone.trigger();
                        }
                        .boxed()
                    }),
                    on_dropped,
                };
                ImmediateDropRequestResponse::Requested
            }
        }
    }

    pub fn immediate_drop_requested(&self) -> Option<&Event> {
        match self {
            ImmediateDropRequest::Requested { on_dropped, .. } => Some(on_dropped),
            ImmediateDropRequest::NotRequested => None,
        }
    }
}

pub enum ImmediateDropRequestResponse {
    Requested,
    NotRequestedBecauseItWasAlreadyRequestedEarlier {
        on_earlier_request_complete: Shared<BoxFuture<'static, ()>>,
    },
}
