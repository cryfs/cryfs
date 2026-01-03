//! Utilities for processing async streams with error handling.
//!
//! This module provides functions for running streams to completion while
//! collecting and handling errors. Unlike typical stream processing that stops
//! on the first error, these utilities continue processing all items and report
//! all errors.

use anyhow::Result;
use futures::{
    future,
    stream::{FuturesUnordered, Stream, StreamExt},
};
use std::fmt::Debug;
use std::future::Future;

/// Runs a stream to completion, returning the first error encountered.
///
/// Unlike typical stream processing that stops on the first error, this function
/// continues processing the entire stream. The first error encountered is returned,
/// and any subsequent errors are logged via the `log` crate.
///
/// # Arguments
///
/// * `stream` - A stream of `Result<(), E>` items to process
///
/// # Returns
///
/// * `Ok(())` if all items in the stream succeeded
/// * `Err(E)` with the first error encountered if any items failed
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

/// Runs an async function concurrently on each item of an iterator.
///
/// If one item fails, the other items will still be run to completion
/// and all errors will be logged in the end. This is different from
/// [`TryStreamExt::try_for_each_concurrent`](futures::stream::TryStreamExt::try_for_each_concurrent)
/// which stops processing on the first error.
///
/// # Arguments
///
/// * `items` - An iterator of items to process
/// * `func` - An async function to apply to each item
///
/// # Returns
///
/// * `Ok(())` if all items were processed successfully
/// * `Err(E)` with the first error encountered if any items failed
pub async fn for_each_unordered<T, E, F>(
    items: impl Iterator<Item = T>,
    func: impl Fn(T) -> F,
) -> Result<(), E>
where
    F: Future<Output = Result<(), E>>,
    E: Debug,
{
    let tasks: FuturesUnordered<_> = items.map(func).collect();
    // TODO Is stream::iter().buffer_unordered() here faster than FuturesUnordered? It was in other places.
    run_to_completion(tasks).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    #[derive(Debug, PartialEq)]
    struct TestError(i32);

    #[tokio::test]
    async fn test_run_to_completion_all_success() {
        let stream = stream::iter(vec![Ok(()), Ok(()), Ok(())]);
        let result: Result<(), TestError> = run_to_completion(stream).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_to_completion_single_error() {
        let stream = stream::iter(vec![Ok(()), Err(TestError(1)), Ok(())]);
        let result = run_to_completion(stream).await;
        assert_eq!(Err(TestError(1)), result);
    }

    #[tokio::test]
    async fn test_run_to_completion_multiple_errors_returns_first() {
        let stream = stream::iter(vec![
            Err(TestError(1)),
            Err(TestError(2)),
            Err(TestError(3)),
        ]);
        let result = run_to_completion(stream).await;
        // Should return the first error
        assert_eq!(Err(TestError(1)), result);
    }

    #[tokio::test]
    async fn test_run_to_completion_empty_stream() {
        let stream = stream::iter(Vec::<Result<(), TestError>>::new());
        let result = run_to_completion(stream).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_for_each_unordered_all_success() {
        let items = vec![1, 2, 3];
        let result: Result<(), TestError> =
            for_each_unordered(items.into_iter(), |_| async { Ok(()) }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_for_each_unordered_with_error() {
        let items = vec![1, 2, 3];
        let result = for_each_unordered(items.into_iter(), |i| async move {
            if i == 2 { Err(TestError(i)) } else { Ok(()) }
        })
        .await;
        assert_eq!(Err(TestError(2)), result);
    }

    #[tokio::test]
    async fn test_for_each_unordered_empty_iterator() {
        let items: Vec<i32> = vec![];
        let result: Result<(), TestError> =
            for_each_unordered(items.into_iter(), |_| async { Ok(()) }).await;
        assert!(result.is_ok());
    }
}
