use async_trait::async_trait;
use futures::future::Future;
use futures::task::{ArcWake, Context, Poll, Waker, waker_ref};
use slab::Slab;
use std::cell::UnsafeCell;
use std::fmt;
use std::fmt::Debug;
use std::hash::Hasher;
use std::pin::Pin;
use std::ptr;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Acquire, SeqCst};
use std::sync::{Arc, Mutex};

use crate::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

/// Same as [futures::future::Shared], but the future can return an [AsyncDrop] type.
/// Implementation is mostly copy&paste from [futures-util] 0.3.31, but with some feature removed (e.g. [futures::future::WeakShared] removed)
///
/// One main difference is that the inner future is supposed to return AsyncDropGuard<O>,
/// and awaiting the [AsyncDropShared] future always returns AsyncDropGuard<AsyncDropArc<O>>.
/// The optimization in [futures::future::Shared] where a refcount=1 [Shared] future returns
/// its result without a reference (and then invalidates future clones) is not implemented here.
/// Futures can be freely cloned and will still return their value, even after being polled to completion.
/// TODO We could implement that optimization here as well, but it would complicate the AsyncDropArc handling a bit,
///      most likely our version of [futures::future::shared::Inner::take_or_clone_output] would have to be async
///      because it has to call async_drop() on `inner`. Or alternatively, we'll have to find a way for AsyncDropArc::try_unwrap()
///      to return a reference to the inner value in its Err(_) if there are other Arc's alive (and then internally drop the Arc without async).
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct AsyncDropShared<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    inner: Option<AsyncDropGuard<AsyncDropArc<Inner<O, Fut>>>>,
    waker_key: usize,
}

struct Inner<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    future_or_output: UnsafeCell<FutureOrOutput<O, Fut>>,
    notifier: Arc<Notifier>,
}

#[async_trait]
impl<O, Fut> AsyncDrop for Inner<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    type Error = <O as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        let inner = std::mem::replace(
            &mut self.future_or_output,
            UnsafeCell::new(FutureOrOutput::Output(AsyncDropGuard::new_invalid())),
        );
        match inner.into_inner() {
            FutureOrOutput::Future(future) => {
                // To ensure that any AsyncDropGuards that may temporarily exist in the future state,
                // or maybe even are returned from the future, are dropped, we have to await the future here.
                let mut output = future.await;
                output.async_drop().await
            }
            FutureOrOutput::Output(mut output) => output.async_drop().await,
        }
    }
}

struct Notifier {
    state: AtomicUsize,
    wakers: Mutex<Option<Slab<Option<Waker>>>>,
}

impl<O, Fut> fmt::Debug for AsyncDropShared<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shared")
            .field("inner", &self.inner)
            .field("waker_key", &self.waker_key)
            .finish()
    }
}

impl<O, Fut> fmt::Debug for Inner<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inner").finish()
    }
}

enum FutureOrOutput<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    Future(Fut),
    Output(AsyncDropGuard<AsyncDropArc<O>>),
}

unsafe impl<O, Fut> Send for Inner<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
}

unsafe impl<O, Fut> Sync for Inner<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
}

const IDLE: usize = 0;
const POLLING: usize = 1;
const COMPLETE: usize = 2;
const POISONED: usize = 3;

const NULL_WAKER_KEY: usize = usize::MAX;

impl<O, Fut: Future> AsyncDropShared<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    pub fn new(future: Fut) -> AsyncDropGuard<Self> {
        let inner = AsyncDropGuard::new(Inner {
            future_or_output: UnsafeCell::new(FutureOrOutput::Future(future)),
            notifier: Arc::new(Notifier {
                state: AtomicUsize::new(IDLE),
                wakers: Mutex::new(Some(Slab::new())),
            }),
        });

        AsyncDropGuard::new(Self {
            inner: Some(AsyncDropArc::new(inner)),
            waker_key: NULL_WAKER_KEY,
        })
    }

    // TODO Tests
    pub fn new_ready(output: AsyncDropGuard<O>) -> AsyncDropGuard<Self> {
        let inner = AsyncDropGuard::new(Inner {
            future_or_output: UnsafeCell::new(FutureOrOutput::Output(AsyncDropArc::new(output))),
            notifier: Arc::new(Notifier {
                state: AtomicUsize::new(COMPLETE),
                wakers: Mutex::new(None),
            }),
        });

        AsyncDropGuard::new(Self {
            inner: Some(AsyncDropArc::new(inner)),
            waker_key: NULL_WAKER_KEY,
        })
    }
}

impl<O, Fut> AsyncDropShared<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    /// Returns [`Some`] containing a reference to this [`Shared`]'s output if
    /// it has already been computed by a clone or [`None`] if it hasn't been
    /// computed yet or this [`Shared`] already returned its output from
    /// [`poll`](Future::poll).
    pub fn peek(&self) -> Option<&AsyncDropGuard<AsyncDropArc<O>>> {
        if let Some(inner) = self.inner.as_ref() {
            match inner.notifier.state.load(SeqCst) {
                COMPLETE => unsafe { return Some(inner.output()) },
                POISONED => panic!("inner future panicked during poll"),
                _ => {}
            }
        }
        None
    }

    /// Gets the number of strong pointers to this allocation.
    ///
    /// Returns [`None`] if it has already been polled to completion.
    ///
    /// # Safety
    ///
    /// This method by itself is safe, but using it correctly requires extra care. Another thread
    /// can change the strong count at any time, including potentially between calling this method
    /// and acting on the result.
    #[allow(clippy::unnecessary_safety_doc)]
    pub fn strong_count(&self) -> Option<usize> {
        self.inner
            .as_ref()
            .map(|arc| AsyncDropArc::strong_count(arc))
    }

    /// Hashes the internal state of this `Shared` in a way that's compatible with `ptr_eq`.
    pub fn ptr_hash<H: Hasher>(&self, state: &mut H) {
        match self.inner.as_ref() {
            Some(arc) => {
                state.write_u8(1);
                ptr::hash(AsyncDropArc::as_ptr(arc), state);
            }
            None => {
                state.write_u8(0);
            }
        }
    }

    /// Returns `true` if the two `Shared`s point to the same future (in a vein similar to
    /// `Arc::ptr_eq`).
    ///
    /// Returns `false` if either `Shared` has terminated.
    pub fn ptr_eq(&self, rhs: &Self) -> bool {
        let lhs = match self.inner.as_ref() {
            Some(lhs) => lhs,
            None => return false,
        };
        let rhs = match rhs.inner.as_ref() {
            Some(rhs) => rhs,
            None => return false,
        };
        AsyncDropArc::ptr_eq(lhs, rhs)
    }

    pub fn clone(this: &AsyncDropGuard<Self>) -> AsyncDropGuard<Self> {
        AsyncDropGuard::new(Self {
            inner: this.inner.as_ref().map(|inner| AsyncDropArc::clone(inner)),
            waker_key: NULL_WAKER_KEY,
        })
    }
}

impl<O, Fut> Inner<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    /// Safety: callers must first ensure that `self.inner.state`
    /// is `COMPLETE`
    unsafe fn output(&self) -> &AsyncDropGuard<AsyncDropArc<O>> {
        match unsafe { &*self.future_or_output.get() } {
            FutureOrOutput::Output(item) => item,
            FutureOrOutput::Future(_) => unreachable!(),
        }
    }
}

impl<O, Fut> Inner<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    /// Registers the current task to receive a wakeup when we are awoken.
    fn record_waker(&self, waker_key: &mut usize, cx: &mut Context<'_>) {
        let mut wakers_guard = self.notifier.wakers.lock().unwrap();

        let wakers = match wakers_guard.as_mut() {
            Some(wakers) => wakers,
            None => return,
        };

        let new_waker = cx.waker();

        if *waker_key == NULL_WAKER_KEY {
            *waker_key = wakers.insert(Some(new_waker.clone()));
        } else {
            match wakers[*waker_key] {
                Some(ref old_waker) if new_waker.will_wake(old_waker) => {}
                // Could use clone_from here, but Waker doesn't specialize it.
                ref mut slot => *slot = Some(new_waker.clone()),
            }
        }
        debug_assert!(*waker_key != NULL_WAKER_KEY);
    }
}

impl<O, Fut> Future for AsyncDropShared<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    type Output = AsyncDropGuard<AsyncDropArc<O>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;

        let inner = this
            .inner
            .as_ref()
            .expect("Shared future polled again after completion");

        // Fast path for when the wrapped future has already completed
        if inner.notifier.state.load(Acquire) == COMPLETE {
            // Safety: We're in the COMPLETE state
            return unsafe { Poll::Ready(AsyncDropArc::clone(inner.output())) };
        }

        inner.record_waker(&mut this.waker_key, cx);

        match inner
            .notifier
            .state
            .compare_exchange(IDLE, POLLING, SeqCst, SeqCst)
            .unwrap_or_else(|x| x)
        {
            IDLE => {
                // Lock acquired, fall through
            }
            POLLING => {
                // Another task is currently polling, at this point we just want
                // to ensure that the waker for this task is registered
                return Poll::Pending;
            }
            COMPLETE => {
                // Safety: We're in the COMPLETE state
                return unsafe { Poll::Ready(AsyncDropArc::clone(inner.output())) };
            }
            POISONED => panic!("inner future panicked during poll"),
            _ => unreachable!(),
        }

        let waker = waker_ref(&inner.notifier);
        let mut cx = Context::from_waker(&waker);

        struct Reset<'a> {
            state: &'a AtomicUsize,
            did_not_panic: bool,
        }

        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                if !self.did_not_panic {
                    self.state.store(POISONED, SeqCst);
                }
            }
        }

        let mut reset = Reset {
            state: &inner.notifier.state,
            did_not_panic: false,
        };

        let output = {
            let future = unsafe {
                match &mut *inner.future_or_output.get() {
                    FutureOrOutput::Future(fut) => Pin::new_unchecked(fut),
                    _ => unreachable!(),
                }
            };

            let poll_result = future.poll(&mut cx);
            reset.did_not_panic = true;

            match poll_result {
                Poll::Pending => {
                    if inner
                        .notifier
                        .state
                        .compare_exchange(POLLING, IDLE, SeqCst, SeqCst)
                        .is_ok()
                    {
                        // Success
                        drop(reset);
                        return Poll::Pending;
                    } else {
                        unreachable!()
                    }
                }
                Poll::Ready(output) => output,
            }
        };

        unsafe {
            *inner.future_or_output.get() = FutureOrOutput::Output(AsyncDropArc::new(output));
        }

        inner.notifier.state.store(COMPLETE, SeqCst);

        // Wake all tasks and drop the slab
        let mut wakers_guard = inner.notifier.wakers.lock().unwrap();
        let mut wakers = wakers_guard.take().unwrap();
        for waker in wakers.drain().flatten() {
            waker.wake();
        }

        drop(reset); // Make borrow checker happy
        drop(wakers_guard);

        // Safety: We're in the COMPLETE state
        unsafe { Poll::Ready(AsyncDropArc::clone(inner.output())) }
    }
}

#[async_trait]
impl<O, Fut> AsyncDrop for AsyncDropShared<O, Fut>
where
    O: Debug + AsyncDrop + Send + Sync,
    Fut: Future<Output = AsyncDropGuard<O>> + Send,
{
    type Error = <O as AsyncDrop>::Error;

    async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
        if self.waker_key != NULL_WAKER_KEY {
            if let Some(ref inner) = self.inner {
                if let Ok(mut wakers) = inner.notifier.wakers.lock() {
                    if let Some(wakers) = wakers.as_mut() {
                        wakers.remove(self.waker_key);
                    }
                }
            }
        }

        if let Some(inner) = &mut self.inner {
            inner.async_drop().await?;
        }
        Ok(())
    }
}

impl ArcWake for Notifier {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        let wakers = &mut *arc_self.wakers.lock().unwrap();
        if let Some(wakers) = wakers.as_mut() {
            for (_key, opt_waker) in wakers {
                if let Some(waker) = opt_waker.take() {
                    waker.wake();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc as StdArc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    #[derive(Debug)]
    struct TestValue {
        value: u32,
        drop_called: StdArc<AtomicBool>,
    }

    #[async_trait]
    impl AsyncDrop for TestValue {
        type Error = ();

        async fn async_drop_impl(&mut self) -> Result<(), Self::Error> {
            assert!(
                !self.drop_called.load(Ordering::SeqCst),
                "Drop called twice",
            );
            self.drop_called.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_drop_result_then_drop_future() {
        let drop_called = StdArc::new(AtomicBool::new(false));
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 42,
                drop_called: drop_called.clone(),
            })
        };

        let mut shared = AsyncDropShared::new(future);
        let mut result = (&mut *shared).await;

        assert!(
            !drop_called.load(Ordering::SeqCst),
            "Value should not be dropped yet"
        );

        // Dropping
        result.async_drop().await.unwrap();
        assert!(
            !drop_called.load(Ordering::SeqCst),
            "Value should not be dropped yet"
        );

        shared.async_drop().await.unwrap();
        assert!(
            drop_called.load(Ordering::SeqCst),
            "Value should be dropped"
        );
    }

    #[tokio::test]
    async fn test_drop_future_then_drop_result() {
        let drop_called = StdArc::new(AtomicBool::new(false));
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 42,
                drop_called: drop_called.clone(),
            })
        };

        let mut shared = AsyncDropShared::new(future);
        let mut result = (&mut *shared).await;

        assert!(
            !drop_called.load(Ordering::SeqCst),
            "Value should not be dropped yet"
        );

        // Dropping
        shared.async_drop().await.unwrap();
        assert!(
            !drop_called.load(Ordering::SeqCst),
            "Value should not be dropped yet"
        );
        result.async_drop().await.unwrap();
        assert!(
            drop_called.load(Ordering::SeqCst),
            "Value should be dropped"
        );
    }

    #[tokio::test]
    async fn test_basic_poll_to_completion() {
        let drop_called = StdArc::new(AtomicBool::new(false));
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 42,
                drop_called: drop_called.clone(),
            })
        };

        let mut shared = AsyncDropShared::new(future);
        let result = (&mut *shared).await;

        assert_eq!(result.value, 42);
        assert!(
            !drop_called.load(Ordering::SeqCst),
            "Value should not be dropped yet"
        );

        // Clean up
        let mut result = result;
        result.async_drop().await.unwrap();
        shared.async_drop().await.unwrap();
        assert!(
            drop_called.load(Ordering::SeqCst),
            "Value should be dropped"
        );
    }

    #[tokio::test]
    async fn test_simple_clone_and_poll() {
        let drop_called = StdArc::new(AtomicBool::new(false));
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 42,
                drop_called: drop_called.clone(),
            })
        };

        let mut shared1 = AsyncDropShared::new(future);
        let mut shared2 = AsyncDropShared::clone(&shared1);

        // Poll the first one
        let result = (&mut *shared1).await;
        assert_eq!(result.value, 42);

        // Clean up
        let mut result = result;
        result.async_drop().await.unwrap();
        shared1.async_drop().await.unwrap();
        shared2.async_drop().await.unwrap();

        assert!(
            drop_called.load(Ordering::SeqCst),
            "Value should be dropped"
        );
    }

    #[tokio::test]
    async fn test_multiple_clones_get_same_value() {
        let drop_called = StdArc::new(AtomicBool::new(false));
        let poll_count = StdArc::new(AtomicU32::new(0));

        let poll_count_clone = poll_count.clone();
        let future = async {
            poll_count_clone.fetch_add(1, Ordering::SeqCst);
            AsyncDropGuard::new(TestValue {
                value: 123,
                drop_called: drop_called.clone(),
            })
        };

        let mut shared1 = AsyncDropShared::new(future);
        let mut shared2 = AsyncDropShared::clone(&shared1);
        let mut shared3 = AsyncDropShared::clone(&shared1);

        // Poll all three
        let result1 = (&mut *shared1).await;
        let result2 = (&mut *shared2).await;
        let result3 = (&mut *shared3).await;

        // All should have the same value
        assert_eq!(result1.value, 123);
        assert_eq!(result2.value, 123);
        assert_eq!(result3.value, 123);

        // The underlying future should only be polled once
        assert_eq!(poll_count.load(Ordering::SeqCst), 1);

        // Clean up
        let mut result = result1;
        result.async_drop().await.unwrap();
        let mut result = result2;
        result.async_drop().await.unwrap();
        let mut result = result3;
        result.async_drop().await.unwrap();
        shared1.async_drop().await.unwrap();
        shared2.async_drop().await.unwrap();
        shared3.async_drop().await.unwrap();
        assert!(
            drop_called.load(Ordering::SeqCst),
            "Value should be dropped"
        );
    }

    #[tokio::test]
    async fn test_peek_before_and_after_completion() {
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 99,
                drop_called: StdArc::new(AtomicBool::new(false)),
            })
        };

        let mut shared = AsyncDropShared::new(future);
        let mut shared2 = AsyncDropShared::clone(&shared);

        // Before polling, peek should return None
        assert!(shared.peek().is_none());
        assert!(shared2.peek().is_none());

        // Poll one clone to completion
        let result = (&mut *shared).await;

        // After one clone is polled, both clones should still be peekable
        assert!(shared.peek().is_some());
        assert!(shared2.peek().is_some());
        assert_eq!(shared2.peek().unwrap().value, 99);

        // Clean up - drop result first since it holds the actual value
        let mut result = result;
        result.async_drop().await.unwrap();
        shared.async_drop().await.unwrap();
        shared2.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_strong_count() {
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 1,
                drop_called: StdArc::new(AtomicBool::new(false)),
            })
        };

        let mut shared1 = AsyncDropShared::new(future);
        assert_eq!(shared1.strong_count(), Some(1));

        let mut shared2 = AsyncDropShared::clone(&shared1);
        assert_eq!(shared1.strong_count(), Some(2));
        assert_eq!(shared2.strong_count(), Some(2));

        let mut shared3 = AsyncDropShared::clone(&shared1);
        assert_eq!(shared1.strong_count(), Some(3));

        shared2.async_drop().await.unwrap();
        assert_eq!(shared1.strong_count(), Some(2));

        // Poll to completion
        let result = (&mut *shared1).await;

        assert_eq!(shared1.strong_count(), Some(2));
        assert_eq!(shared3.strong_count(), Some(2));

        // Clean up
        let mut result = result;
        result.async_drop().await.unwrap();
        shared1.async_drop().await.unwrap();
        shared3.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_ptr_eq() {
        let future1 = || async {
            AsyncDropGuard::new(TestValue {
                value: 1,
                drop_called: StdArc::new(AtomicBool::new(false)),
            })
        };

        let mut shared1a = AsyncDropShared::new(future1());
        let mut shared1b = AsyncDropShared::clone(&shared1a);
        let mut shared1c = AsyncDropShared::clone(&shared1a);

        // Clones should point to the same future
        assert!(shared1a.ptr_eq(&*shared1b));
        assert!(shared1b.ptr_eq(&*shared1a));
        assert!(shared1a.ptr_eq(&*shared1c));

        // Different futures should not be equal
        let mut shared2 = AsyncDropShared::new(future1());
        assert!(!shared1a.ptr_eq(&*shared2));
        shared2.async_drop().await.unwrap();

        // Clean up
        shared1a.async_drop().await.unwrap();
        shared1b.async_drop().await.unwrap();
        shared1c.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_async_drop_of_shared() {
        let drop_called = StdArc::new(AtomicBool::new(false));
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 1,
                drop_called: drop_called.clone(),
            })
        };

        let mut shared1 = AsyncDropShared::new(future);
        let mut shared2 = AsyncDropShared::clone(&shared1);

        // Poll to completion
        let result = (&mut *shared1).await;

        // Drop one shared without dropping the result
        shared1.async_drop().await.unwrap();
        assert!(
            !drop_called.load(Ordering::SeqCst),
            "Value should not be dropped yet"
        );

        // Drop the second shared
        shared2.async_drop().await.unwrap();
        assert!(
            !drop_called.load(Ordering::SeqCst),
            "Value should not be dropped yet"
        );

        // Finally drop the result
        let mut result = result;
        result.async_drop().await.unwrap();
        assert!(
            drop_called.load(Ordering::SeqCst),
            "Value should be dropped now"
        );
    }

    #[tokio::test]
    async fn test_async_drop_without_polling() {
        let drop_called = StdArc::new(AtomicBool::new(false));
        let drop_called_clone = drop_called.clone();

        let future = async move {
            AsyncDropGuard::new(TestValue {
                value: 1,
                drop_called: drop_called_clone,
            })
        };

        let mut shared = AsyncDropShared::new(future);
        let mut clone = AsyncDropShared::clone(&shared);

        // Drop without polling to completion
        shared.async_drop().await.unwrap();
        clone.async_drop().await.unwrap();

        // The value should be dropped as part of dropping the shared future
        assert!(drop_called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_clone_after_completion() {
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 42,
                drop_called: StdArc::new(AtomicBool::new(false)),
            })
        };

        let mut shared1 = AsyncDropShared::new(future);
        let result1 = (&mut *shared1).await;

        // Clone after completion
        let mut shared2 = AsyncDropShared::clone(&shared1);
        let result2 = (&mut *shared2).await;

        assert_eq!(result2.value, 42);

        // Clean up - drop results first
        let mut result1 = result1;
        result1.async_drop().await.unwrap();
        let mut result2 = result2;
        result2.async_drop().await.unwrap();
        shared1.async_drop().await.unwrap();
        shared2.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_poll_multiple_times() {
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 42,
                drop_called: StdArc::new(AtomicBool::new(false)),
            })
        };

        let mut shared1 = AsyncDropShared::new(future);
        let result1 = (&mut *shared1).await;
        let result2 = (&mut *shared1).await;

        assert_eq!(result1.value, 42);
        assert_eq!(result2.value, 42);

        // Clean up - drop results first
        let mut result1 = result1;
        result1.async_drop().await.unwrap();
        let mut result2 = result2;
        result2.async_drop().await.unwrap();
        shared1.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_debug_impl() {
        let future = async {
            AsyncDropGuard::new(TestValue {
                value: 1,
                drop_called: StdArc::new(AtomicBool::new(false)),
            })
        };

        let mut shared = AsyncDropShared::new(future);
        let debug_str = format!("{:?}", shared);
        assert!(debug_str.contains("Shared"));

        // Clean up
        let mut result = (&mut *shared).await;
        result.async_drop().await.unwrap();
        shared.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_ptr_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let future1 = async {
            AsyncDropGuard::new(TestValue {
                value: 1,
                drop_called: StdArc::new(AtomicBool::new(false)),
            })
        };

        let mut shared1a = AsyncDropShared::new(future1);
        let mut shared1b = AsyncDropShared::clone(&shared1a);

        let mut hasher1a = DefaultHasher::new();
        shared1a.ptr_hash(&mut hasher1a);
        let hash1a = hasher1a.finish();

        let mut hasher1b = DefaultHasher::new();
        shared1b.ptr_hash(&mut hasher1b);
        let hash1b = hasher1b.finish();

        // Clones should have the same hash
        assert_eq!(hash1a, hash1b);

        // Clean up
        let result1 = (&mut *shared1a).await;
        let mut result1 = result1;
        result1.async_drop().await.unwrap();
        shared1a.async_drop().await.unwrap();
        shared1b.async_drop().await.unwrap();
    }
}
