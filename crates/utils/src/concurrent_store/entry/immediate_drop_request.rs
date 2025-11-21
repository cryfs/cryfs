use anyhow::Error;
use futures::{FutureExt as _, future::BoxFuture};
use std::fmt::Debug;
use std::sync::Arc;

use crate::{
    async_drop::{AsyncDrop, AsyncDropGuard},
    mr_oneshot_channel,
};

/// Represents a request to immediately drop an entry that is currently loading or loaded.
pub enum ImmediateDropRequest<V, D>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    /// No immediate drop has been requested.
    NotRequested,
    /// An immediate drop has been requested.
    Requested {
        /// The entry gets sent here once all other references to it are gone and we have exclusive access to it.
        /// This function is then expected to drop it.
        // TODO No Box<dyn> but impl Fn?
        drop_fn: Box<
            dyn FnOnce(Option<AsyncDropGuard<V>>) -> BoxFuture<'static, Result<D, Arc<Error>>>
                + Send
                + Sync,
        >,
        /// Sender to notify the requester when the drop is complete.
        completion_sender: mr_oneshot_channel::Sender<Result<D, Arc<Error>>>,
    },
}

impl<V, D> ImmediateDropRequest<V, D>
where
    V: AsyncDrop + Debug + Send + Sync + 'static,
    D: Clone + Debug + Send + Sync + 'static,
{
    /// Request an immediate drop of the entry.
    /// If an immediate drop has already been requested, returns a receiver to wait for the completion of that request.
    /// If no immediate drop has been requested yet, sets up the request with the provided drop function and returns a receiver to wait for its completion.
    pub fn request_immediate_drop_if_not_yet_requested<F>(
        &mut self,
        drop_fn: impl FnOnce(Option<AsyncDropGuard<V>>) -> F + Send + Sync + 'static,
    ) -> ImmediateDropRequestResponse<D>
    where
        F: Future<Output = Result<D, Arc<Error>>> + Send,
    {
        match self {
            ImmediateDropRequest::Requested {
                completion_sender, ..
            } => ImmediateDropRequestResponse::NotRequestedBecauseItWasAlreadyRequestedEarlier {
                on_dropped: completion_sender.subscribe(),
            },
            ImmediateDropRequest::NotRequested => {
                let (completion_sender, entry_receiver) = mr_oneshot_channel::channel();
                *self = ImmediateDropRequest::Requested {
                    drop_fn: Box::new(move |i| async move { drop_fn(i).await }.boxed()),
                    completion_sender: completion_sender,
                };
                ImmediateDropRequestResponse::Requested {
                    on_dropped: entry_receiver,
                }
            }
        }
    }

    pub fn immediate_drop_requested(
        &self,
    ) -> Option<mr_oneshot_channel::Receiver<Result<D, Arc<Error>>>> {
        match self {
            ImmediateDropRequest::Requested {
                completion_sender, ..
            } => Some(completion_sender.subscribe()),
            ImmediateDropRequest::NotRequested => None,
        }
    }
}

pub enum ImmediateDropRequestResponse<R> {
    Requested {
        on_dropped: mr_oneshot_channel::Receiver<Result<R, Arc<Error>>>,
    },
    NotRequestedBecauseItWasAlreadyRequestedEarlier {
        on_dropped: mr_oneshot_channel::Receiver<Result<R, Arc<Error>>>,
    },
}
