use anyhow::Result;
use futures::{
    future,
    stream::{FuturesUnordered, Stream, StreamExt},
};
use std::fmt::Debug;
use std::future::Future;

/// Run the stream to completion and log all errors encountered, except one error which is returned
pub async fn run_to_completion<E: Debug>(
    stream: impl Stream<Item = Result<(), E>>,
) -> Result<(), E> {
    let errors = stream.filter_map(|result| match result {
        Ok(()) => future::ready(None),
        Err(err) => future::ready(Some(err)),
    });
    let mut errors = Box::pin(errors);
    let mut first_error = None;

    // This while loop drives the whole stream (successes and errors) but only enters the loop body for errors.
    while let Some(error) = errors.next().await {
        if first_error.is_none() {
            first_error = Some(error);
        } else {
            // TODO Return a list of all errors instead of logging swallowed ones
            log::error!("Error while processing stream: {:?}", error);
        }
    }

    if let Some(error) = first_error {
        Err(error)
    } else {
        Ok(())
    }
}

/// Run the given async func concurrently on each item of the iterator.
/// If one item fails, the other items will still be run to completion
/// and all errors will be logged in the end. This is different from
/// [TryStreamExt::try_for_each_concurrent](futures::stream::TryStreamExt::try_for_each_concurrent).
pub async fn for_each_unordered<T, E, F>(
    items: impl Iterator<Item = T>,
    func: impl Fn(T) -> F,
) -> Result<(), E>
where
    F: Future<Output = Result<(), E>>,
    E: Debug,
{
    // TODO Is stream::iter().buffer_unordered() here faster than FuturesUnordered? It was in other places.
    let tasks: FuturesUnordered<_> = items.map(func).collect();
    run_to_completion(tasks).await
}

// TODO Tests
