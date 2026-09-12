//! Dropping several [AsyncDropGuard]s at once.
//!
//! [async_drop_all] takes a tuple of guards, drops all of them concurrently, and waits for
//! all of them even if some fail. See [with_async_drop_2!](crate::with_async_drop_2) for the
//! macro form that also runs a block of code before dropping.

use std::fmt::Debug;
use std::future::Future;

use super::{AsyncDrop, AsyncDropGuard};

/// An element that [async_drop_all] can drop: an [AsyncDropGuard], or an [Option] of one
/// where [None] means there is nothing to drop.
pub trait AsyncDropAllElement {
    type Error: Debug;

    fn async_drop_element(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<T: AsyncDrop + Debug> AsyncDropAllElement for AsyncDropGuard<T> {
    type Error = T::Error;

    fn async_drop_element(self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        self.async_drop()
    }
}

impl<T: AsyncDrop + Debug> AsyncDropAllElement for Option<AsyncDropGuard<T>> {
    type Error = T::Error;

    fn async_drop_element(self) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let drop_future = self.map(AsyncDropGuard::async_drop);
        async move {
            match drop_future {
                Some(drop_future) => drop_future.await,
                None => Ok(()),
            }
        }
    }
}

/// A tuple of [AsyncDropAllElement]s that all share the same error type.
///
/// Implemented for tuples of one to eight elements.
pub trait AsyncDropTuple {
    type Error: Debug;

    /// See [async_drop_all].
    fn async_drop_all(self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

/// Drops all guards in the tuple concurrently.
///
/// All drops are started together and all of them are awaited, even if some of them fail.
/// If several fail, the first error in tuple order is returned and the others are logged.
///
/// Elements can be [AsyncDropGuard]s or `Option<AsyncDropGuard<_>>`s, and all of them
/// must have the same [AsyncDrop::Error] type.
///
/// ```
/// use cryfs_utils::async_drop::{AsyncDrop, AsyncDropGuard, async_drop_all};
///
/// #[derive(Debug)]
/// struct Connection;
///
/// impl AsyncDrop for Connection {
///     type Error = std::convert::Infallible;
///
///     async fn async_drop_impl(self) -> Result<(), Self::Error> {
///         Ok(())
///     }
/// }
///
/// # futures::executor::block_on(async {
/// let first = AsyncDropGuard::new(Connection);
/// let second = AsyncDropGuard::new(Connection);
/// let maybe_third: Option<AsyncDropGuard<Connection>> = None;
/// async_drop_all((first, second, maybe_third)).await.unwrap();
/// # });
/// ```
pub fn async_drop_all<T: AsyncDropTuple>(
    guards: T,
) -> impl Future<Output = Result<(), T::Error>> + Send {
    guards.async_drop_all()
}

// TODO Instead of only returning the first error and logging the others, we might want to
//      return all of them, e.g. with a list error type.
fn record_error<E: Debug>(first_error: &mut Option<E>, result: Result<(), E>) {
    if let Err(error) = result {
        if first_error.is_none() {
            *first_error = Some(error);
        } else {
            log::error!(
                "Error while dropping several values at once. Only the first error is returned, this one is dropped: {error:?}"
            );
        }
    }
}

macro_rules! impl_async_drop_tuple {
    ($($element:ident . $index:tt),+) => {
        impl<E: Debug + Send, $($element: AsyncDropAllElement<Error = E>),+> AsyncDropTuple
            for ($($element,)+)
        {
            type Error = E;

            fn async_drop_all(self) -> impl Future<Output = Result<(), E>> + Send {
                // Create all futures before the async block so that it only captures the
                // futures (which are `Send` by the contract of `AsyncDrop`) and not the guards.
                let futures = ($(self.$index.async_drop_element(),)+);
                async move {
                    let results = futures::join!($(futures.$index),+);
                    let mut first_error = None;
                    $(record_error(&mut first_error, results.$index);)+
                    match first_error {
                        Some(error) => Err(error),
                        None => Ok(()),
                    }
                }
            }
        }
    };
}

impl_async_drop_tuple!(A.0);
impl_async_drop_tuple!(A.0, B.1);
impl_async_drop_tuple!(A.0, B.1, C.2);
impl_async_drop_tuple!(A.0, B.1, C.2, D.3);
impl_async_drop_tuple!(A.0, B.1, C.2, D.3, F.4);
impl_async_drop_tuple!(A.0, B.1, C.2, D.3, F.4, G.5);
impl_async_drop_tuple!(A.0, B.1, C.2, D.3, F.4, G.5, H.6);
impl_async_drop_tuple!(A.0, B.1, C.2, D.3, F.4, G.5, H.6, I.7);

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use tokio::sync::Barrier;

    #[derive(Debug)]
    struct TestValue {
        drop_counter: Arc<AtomicUsize>,
        fail_with: Option<&'static str>,
        barrier: Option<Arc<Barrier>>,
    }

    impl TestValue {
        fn new(drop_counter: &Arc<AtomicUsize>) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self {
                drop_counter: Arc::clone(drop_counter),
                fail_with: None,
                barrier: None,
            })
        }

        fn failing(drop_counter: &Arc<AtomicUsize>, error: &'static str) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self {
                drop_counter: Arc::clone(drop_counter),
                fail_with: Some(error),
                barrier: None,
            })
        }

        fn waiting_on(
            drop_counter: &Arc<AtomicUsize>,
            barrier: &Arc<Barrier>,
        ) -> AsyncDropGuard<Self> {
            AsyncDropGuard::new(Self {
                drop_counter: Arc::clone(drop_counter),
                fail_with: None,
                barrier: Some(Arc::clone(barrier)),
            })
        }
    }

    impl AsyncDrop for TestValue {
        type Error = &'static str;

        async fn async_drop_impl(self) -> Result<(), Self::Error> {
            if let Some(barrier) = &self.barrier {
                barrier.wait().await;
            }
            self.drop_counter.fetch_add(1, Ordering::SeqCst);
            match self.fail_with {
                Some(error) => Err(error),
                None => Ok(()),
            }
        }
    }

    #[tokio::test]
    async fn given_two_guards_when_dropping_all_then_both_are_dropped() {
        let counter = Arc::new(AtomicUsize::new(0));
        let a = TestValue::new(&counter);
        let b = TestValue::new(&counter);

        let result = async_drop_all((a, b)).await;

        assert_eq!(Ok(()), result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_single_guard_when_dropping_all_then_it_is_dropped() {
        let counter = Arc::new(AtomicUsize::new(0));
        let a = TestValue::new(&counter);

        let result = async_drop_all((a,)).await;

        assert_eq!(Ok(()), result);
        assert_eq!(1, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_eight_guards_when_dropping_all_then_all_are_dropped() {
        let counter = Arc::new(AtomicUsize::new(0));
        let guards = (
            TestValue::new(&counter),
            TestValue::new(&counter),
            TestValue::new(&counter),
            TestValue::new(&counter),
            TestValue::new(&counter),
            TestValue::new(&counter),
            TestValue::new(&counter),
            TestValue::new(&counter),
        );

        let result = async_drop_all(guards).await;

        assert_eq!(Ok(()), result);
        assert_eq!(8, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_first_drop_fails_when_dropping_all_then_second_is_still_dropped_and_error_is_returned()
     {
        let counter = Arc::new(AtomicUsize::new(0));
        let a = TestValue::failing(&counter, "error a");
        let b = TestValue::new(&counter);

        let result = async_drop_all((a, b)).await;

        assert_eq!(Err("error a"), result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_last_drop_fails_when_dropping_all_then_first_is_still_dropped_and_error_is_returned()
     {
        let counter = Arc::new(AtomicUsize::new(0));
        let a = TestValue::new(&counter);
        let b = TestValue::failing(&counter, "error b");

        let result = async_drop_all((a, b)).await;

        assert_eq!(Err("error b"), result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_all_drops_fail_when_dropping_all_then_first_error_in_tuple_order_is_returned() {
        let counter = Arc::new(AtomicUsize::new(0));
        let a = TestValue::failing(&counter, "error a");
        let b = TestValue::failing(&counter, "error b");
        let c = TestValue::failing(&counter, "error c");

        let result = async_drop_all((a, b, c)).await;

        assert_eq!(Err("error a"), result);
        assert_eq!(3, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_optional_guards_when_dropping_all_then_some_is_dropped_and_none_is_ok() {
        let counter = Arc::new(AtomicUsize::new(0));
        let a: Option<AsyncDropGuard<TestValue>> = None;
        let b = Some(TestValue::new(&counter));
        let c = TestValue::new(&counter);

        let result = async_drop_all((a, b, c)).await;

        assert_eq!(Ok(()), result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_guards_that_wait_for_each_other_when_dropping_all_then_they_are_dropped_concurrently()
     {
        // Each drop blocks until both drops have started. This only completes if the two drops
        // run concurrently, so a sequential implementation would hit the timeout.
        let counter = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(2));
        let a = TestValue::waiting_on(&counter, &barrier);
        let b = TestValue::waiting_on(&counter, &barrier);

        let result = tokio::time::timeout(Duration::from_secs(10), async_drop_all((a, b)))
            .await
            .expect("drops did not run concurrently");

        assert_eq!(Ok(()), result);
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_guard_and_optional_guard_with_different_value_types_when_dropping_all_then_both_dropped()
     {
        #[derive(Debug)]
        struct OtherValue(Arc<AtomicUsize>);
        impl AsyncDrop for OtherValue {
            type Error = &'static str;
            async fn async_drop_impl(self) -> Result<(), Self::Error> {
                self.0.fetch_add(10, Ordering::SeqCst);
                Ok(())
            }
        }

        let counter = Arc::new(AtomicUsize::new(0));
        let a = TestValue::new(&counter);
        let b = AsyncDropGuard::new(OtherValue(Arc::clone(&counter)));

        let result = async_drop_all((a, b)).await;

        assert_eq!(Ok(()), result);
        assert_eq!(11, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn given_guards_when_dropping_all_then_future_is_send() {
        fn assert_send<T: Send>(_: &T) {}

        let counter = Arc::new(AtomicUsize::new(0));
        let a = TestValue::new(&counter);
        let b = Some(TestValue::new(&counter));

        let future = async_drop_all((a, b));
        assert_send(&future);
        future.await.unwrap();
        assert_eq!(2, counter.load(Ordering::SeqCst));
    }
}
