//! Tests for the ConcurrentStore state machine.
//!
//! Test categories:
//! 1. Basic Operations - Core CRUD operations
//! 2. Loading State - Loading behavior, waiter sharing
//! 3. Dropping State - Drop behavior, request_immediate_drop
//! 4. Synchronous Operation Verification - Verify operations don't block
//! 5. Chain Walking - Nested intent/reload chains
//! 6. Event Signaling - on_dropped event propagation
//! 7. Concurrent Access - Multi-threaded race scenarios
//! 8. Error Handling - Error propagation and cleanup

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use futures::FutureExt;
use lockable::Never;
use tokio::sync::{Barrier, Notify};
use tokio::time::timeout;

use cryfs_utils::async_drop::{AsyncDrop, AsyncDropArc, AsyncDropGuard};

use crate::{ConcurrentStore, RequestImmediateDropResult};

// ============================================================================
// Test Infrastructure
// ============================================================================

/// A simple test value type that tracks async drop calls.
#[derive(Debug)]
struct TestValue {
    id: usize,
    dropped: Arc<AtomicBool>,
}

impl TestValue {
    fn new(id: usize) -> Self {
        Self {
            id,
            dropped: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[async_trait]
impl AsyncDrop for TestValue {
    type Error = Never;

    async fn async_drop_impl(&mut self) -> Result<(), Never> {
        self.dropped.store(true, Ordering::SeqCst);
        Ok(())
    }
}

/// A simple test input type for the loading function.
/// This is required because get_loaded_or_insert_loading takes an input that
/// implements AsyncDrop<Error = anyhow::Error>.
#[derive(Debug)]
struct TestInput {
    _id: usize,
}

impl TestInput {
    fn new(id: usize) -> Self {
        Self { _id: id }
    }
}

#[async_trait]
impl AsyncDrop for TestInput {
    type Error = anyhow::Error;

    async fn async_drop_impl(&mut self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}

/// Create a test input for the loading function.
fn test_input() -> AsyncDropGuard<AsyncDropArc<TestInput>> {
    AsyncDropArc::new(AsyncDropGuard::new(TestInput::new(0)))
}

/// A test error type.
#[derive(Clone, Debug, PartialEq)]
struct TestError(String);

/// Type alias for our test store.
type TestStore = ConcurrentStore<usize, TestValue, TestError>;

/// Create a new empty test store.
fn test_store() -> AsyncDropGuard<TestStore> {
    ConcurrentStore::new()
}

/// Create a simple loading function that succeeds immediately.
fn simple_loader(
    id: usize,
) -> impl FnOnce(
    AsyncDropGuard<AsyncDropArc<TestInput>>,
) -> futures::future::BoxFuture<
    'static,
    Result<Option<AsyncDropGuard<TestValue>>, TestError>,
> {
    move |mut input| {
        async move {
            input.async_drop().await.unwrap();
            let value = TestValue::new(id);
            Ok(Some(AsyncDropGuard::new(value)))
        }
        .boxed()
    }
}

/// Create a loading function that waits for a signal before completing.
fn signaled_loader(
    id: usize,
    notify: Arc<Notify>,
) -> impl FnOnce(
    AsyncDropGuard<AsyncDropArc<TestInput>>,
) -> futures::future::BoxFuture<
    'static,
    Result<Option<AsyncDropGuard<TestValue>>, TestError>,
> {
    move |mut input| {
        async move {
            input.async_drop().await.unwrap();
            notify.notified().await;
            let value = TestValue::new(id);
            Ok(Some(AsyncDropGuard::new(value)))
        }
        .boxed()
    }
}

/// Create a loading function that fails with an error.
fn failing_loader(
    error: TestError,
) -> impl FnOnce(
    AsyncDropGuard<AsyncDropArc<TestInput>>,
) -> futures::future::BoxFuture<
    'static,
    Result<Option<AsyncDropGuard<TestValue>>, TestError>,
> {
    move |mut input| {
        async move {
            input.async_drop().await.unwrap();
            Err(error)
        }
        .boxed()
    }
}

/// Create a loading function that returns NotFound.
fn not_found_loader() -> impl FnOnce(
    AsyncDropGuard<AsyncDropArc<TestInput>>,
) -> futures::future::BoxFuture<
    'static,
    Result<Option<AsyncDropGuard<TestValue>>, TestError>,
> {
    move |mut input| {
        async move {
            input.async_drop().await.unwrap();
            Ok(None)
        }
        .boxed()
    }
}

/// Create a loader that tracks how many times it was called.
fn counting_loader(
    id: usize,
    counter: Arc<AtomicUsize>,
) -> impl FnOnce(
    AsyncDropGuard<AsyncDropArc<TestInput>>,
) -> futures::future::BoxFuture<
    'static,
    Result<Option<AsyncDropGuard<TestValue>>, TestError>,
> {
    move |mut input| {
        async move {
            input.async_drop().await.unwrap();
            counter.fetch_add(1, Ordering::SeqCst);
            let value = TestValue::new(id);
            Ok(Some(AsyncDropGuard::new(value)))
        }
        .boxed()
    }
}

/// Create a drop function that does nothing (but properly async_drops the value).
fn noop_drop_fn()
-> impl FnOnce(Option<AsyncDropGuard<TestValue>>) -> futures::future::BoxFuture<'static, ()>
+ Send
+ Sync
+ 'static {
    |value| {
        async move {
            if let Some(mut v) = value {
                v.async_drop().await.unwrap();
            }
        }
        .boxed()
    }
}

/// Create a drop function that signals when called.
fn signaling_drop_fn(
    signal: Arc<AtomicBool>,
) -> impl FnOnce(Option<AsyncDropGuard<TestValue>>) -> futures::future::BoxFuture<'static, ()>
+ Send
+ Sync
+ 'static {
    move |value| {
        async move {
            if let Some(mut v) = value {
                v.async_drop().await.unwrap();
            }
            signal.store(true, Ordering::SeqCst);
        }
        .boxed()
    }
}

/// Create a drop function that waits for a signal before completing.
fn waiting_drop_fn(
    notify: Arc<Notify>,
) -> impl FnOnce(Option<AsyncDropGuard<TestValue>>) -> futures::future::BoxFuture<'static, ()>
+ Send
+ Sync
+ 'static {
    move |value| {
        async move {
            if let Some(mut v) = value {
                v.async_drop().await.unwrap();
            }
            notify.notified().await;
        }
        .boxed()
    }
}

// ============================================================================
// Category 1: Basic Operations
// ============================================================================

mod basic_operations {
    use super::*;

    #[tokio::test]
    async fn test_new_creates_empty_store() {
        let mut store = test_store();
        assert!(store.is_fully_absent(&1));
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_or_insert_on_empty_starts_loading() {
        let mut store = test_store();
        let mut input = test_input();
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard.value().id, 1);
        guard.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_or_insert_on_loaded_returns_immediately() {
        let mut store = test_store();
        let mut input = test_input();

        // First load
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard1 = result.wait_until_loaded().await.unwrap().unwrap();

        // Second get should return the same value
        let result2 = store.get_loaded_or_insert_loading(1, &input, simple_loader(999));
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        // Both should have the same id (first load's value, not 999)
        assert_eq!(guard1.value().id, 1);
        assert_eq!(guard2.value().id, 1);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_if_loading_or_loaded_on_empty_returns_none() {
        let mut store = test_store();
        let result = store.get_if_loading_or_loaded(1);
        let loaded = result.wait_until_loaded().await.unwrap();
        assert!(loaded.is_none());
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_if_loading_or_loaded_on_loaded() {
        let mut store = test_store();
        let mut input = test_input();

        // First load
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard1 = result.wait_until_loaded().await.unwrap().unwrap();

        // get_if should find it
        let result2 = store.get_if_loading_or_loaded(1);
        let guard2 = result2.wait_until_loaded().await.unwrap();
        assert!(guard2.is_some());
        let mut guard2 = guard2.unwrap();
        assert_eq!(guard2.value().id, 1);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_is_fully_absent_on_empty() {
        let mut store = test_store();
        assert!(store.is_fully_absent(&1));
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_is_fully_absent_on_loaded() {
        let mut store = test_store();
        let mut input = test_input();
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        assert!(!store.is_fully_absent(&1));
        guard.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_guard_releases_entry() {
        let mut store = test_store();
        let mut input = test_input();
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        assert!(!store.is_fully_absent(&1));

        // Drop the guard
        guard.async_drop().await.unwrap();

        // Entry may still exist briefly due to async cleanup, but should become absent
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(store.is_fully_absent(&1));

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_try_insert_loaded_stores_value() {
        let mut store = test_store();
        let value = AsyncDropGuard::new(TestValue::new(42));
        let inserting = store.try_insert_loaded(1, value);
        let mut guard = inserting.unwrap();
        assert_eq!(guard.value().id, 42);
        guard.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 2: Loading State Tests
// ============================================================================

mod loading_state {
    use super::*;

    #[tokio::test]
    async fn test_multiple_gets_during_loading_share_future() {
        let mut store = test_store();
        let mut input = test_input();
        let load_count = Arc::new(AtomicUsize::new(0));
        let notify = Arc::new(Notify::new());

        // Start loading with signaled loader
        let result1 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(1, notify.clone()));

        // Second get before loading completes
        let result2 =
            store.get_loaded_or_insert_loading(1, &input, counting_loader(999, load_count.clone()));

        // Signal the loader to complete
        notify.notify_one();

        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        // Both should get the first load's value
        assert_eq!(guard1.value().id, 1);
        assert_eq!(guard2.value().id, 1);
        // The second loader should never have been called
        assert_eq!(load_count.load(Ordering::SeqCst), 0);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_loading_error_propagates_to_all_waiters() {
        let mut store = test_store();
        let mut input = test_input();
        let notify = Arc::new(Notify::new());

        // Start loading that will fail
        let result1 = store.get_loaded_or_insert_loading(1, &input, {
            let notify = notify.clone();
            move |mut input| {
                async move {
                    input.async_drop().await.unwrap();
                    notify.notified().await;
                    Err(TestError("load failed".to_string()))
                }
                .boxed()
            }
        });

        // Second waiter
        let result2 = store.get_loaded_or_insert_loading(1, &input, simple_loader(999));

        // Signal to complete with error
        notify.notify_one();

        let err1 = result1.wait_until_loaded().await;
        let err2 = result2.wait_until_loaded().await;

        assert!(err1.is_err());
        assert!(err2.is_err());
        assert_eq!(err1.unwrap_err(), TestError("load failed".to_string()));
        assert_eq!(err2.unwrap_err(), TestError("load failed".to_string()));

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_loading_not_found_propagates() {
        let mut store = test_store();
        let mut input = test_input();
        let result = store.get_loaded_or_insert_loading(1, &input, not_found_loader());
        let loaded = result.wait_until_loaded().await.unwrap();
        assert!(loaded.is_none());
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_loading_completion_transitions_to_loaded() {
        let mut store = test_store();
        let mut input = test_input();
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Now it's loaded, another get should return immediately
        let result2 = store.get_loaded_or_insert_loading(1, &input, simple_loader(999));
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard2.value().id, 1); // Still the first value

        guard.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 3: Dropping State Tests
// ============================================================================

mod dropping_state {
    use super::*;

    #[tokio::test]
    async fn test_request_immediate_drop_on_loaded() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_called = Arc::new(AtomicBool::new(false));

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        guard.async_drop().await.unwrap();

        // Request immediate drop
        let drop_result = store.request_immediate_drop(1, signaling_drop_fn(drop_called.clone()));

        match drop_result {
            RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                drop_result.await;
                assert!(drop_called.load(Ordering::SeqCst));
            }
            _ => panic!("Expected ImmediateDropRequested"),
        }

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_request_immediate_drop_on_loading() {
        let mut store = test_store();
        let mut input = test_input();
        let notify = Arc::new(Notify::new());
        let drop_called = Arc::new(AtomicBool::new(false));

        // Start loading
        let result =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(1, notify.clone()));

        // Request immediate drop while loading
        let drop_result = store.request_immediate_drop(1, signaling_drop_fn(drop_called.clone()));

        // Signal to complete loading
        notify.notify_one();

        match drop_result {
            RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                // Wait for the guard to get and release
                let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
                guard.async_drop().await.unwrap();
                drop_result.await;
                assert!(drop_called.load(Ordering::SeqCst));
            }
            _ => panic!("Expected ImmediateDropRequested"),
        }

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_request_immediate_drop_on_vacant() {
        let mut store = test_store();
        let drop_called = Arc::new(AtomicBool::new(false));

        // Request immediate drop on non-existent entry
        let drop_result = store.request_immediate_drop(1, signaling_drop_fn(drop_called.clone()));

        match drop_result {
            RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                drop_result.await;
                // drop_fn should be called with None
                assert!(drop_called.load(Ordering::SeqCst));
            }
            _ => panic!("Expected ImmediateDropRequested"),
        }

        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_entry_removed_after_drop_completes() {
        let mut store = test_store();
        let mut input = test_input();

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        guard.async_drop().await.unwrap();

        // Wait for cleanup
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Entry should be removed
        assert!(store.is_fully_absent(&1));

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 4: Synchronous Operation Verification
// ============================================================================

mod synchronous_operations {
    use super::*;

    #[tokio::test]
    async fn test_get_on_loaded_is_synchronous() {
        let mut store = test_store();
        let mut input = test_input();

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard1 = result.wait_until_loaded().await.unwrap().unwrap();

        // Second get should return IMMEDIATELY (no await needed for the operation itself)
        // The LoadingOrLoaded is returned synchronously, only wait_until_loaded might wait
        let result2 = store.get_loaded_or_insert_loading(1, &input, simple_loader(999));

        // Since entry is loaded, this should complete instantly
        let mut guard2 = timeout(Duration::from_millis(10), result2.wait_until_loaded())
            .await
            .expect("get on loaded entry should be instant")
            .unwrap()
            .unwrap();

        assert_eq!(guard2.value().id, 1);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_on_dropping_is_synchronous() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request drop with waiting drop_fn BEFORE releasing the guard
        // This creates an intent on the Loaded state
        let _drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Release the guard in a spawned task so we can do operations while the drop_fn is running
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });

        // Give some time for the drop to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // get_loaded_or_insert_loading should return SYNCHRONOUSLY (not block on drop)
        // The call returns immediately with a waiter, the waiter will complete when reload is done
        let result2 = store.get_loaded_or_insert_loading(1, &input, simple_loader(42));

        // The call itself returned, so we got a LoadingOrLoaded synchronously
        // Now signal the drop to complete
        drop_notify.notify_one();

        // Now wait for the reload
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard2.value().id, 42); // New value from reload

        guard2.async_drop().await.unwrap();
        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_request_drop_on_loaded_is_synchronous() {
        let mut store = test_store();
        let mut input = test_input();

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request drop - should return synchronously even though drop hasn't completed
        let drop_result = store.request_immediate_drop(1, noop_drop_fn());

        match drop_result {
            RequestImmediateDropResult::ImmediateDropRequested { .. } => {
                // Success - we got the result synchronously
            }
            _ => panic!("Expected ImmediateDropRequested"),
        }

        guard.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 5: Chain Walking Tests
// ============================================================================

mod chain_walking {
    use super::*;

    /// Scenario 1 from plan: get while Dropping -> second get -> request_immediate_drop
    #[tokio::test]
    async fn test_get_while_dropping_sets_reload() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request drop with waiting drop_fn BEFORE releasing guard
        let _drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Release guard in a spawned task so we can do operations while the drop_fn is running
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // First get while dropping - should set reload
        let result1 = store.get_loaded_or_insert_loading(1, &input, simple_loader(10));

        // Second get while dropping - should add waiter to same reload
        let load_count = Arc::new(AtomicUsize::new(0));
        let result2 =
            store.get_loaded_or_insert_loading(1, &input, counting_loader(20, load_count.clone()));

        // Signal drop to complete
        drop_notify.notify_one();

        // Both should get the first reload's value
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        assert_eq!(guard1.value().id, 10);
        assert_eq!(guard2.value().id, 10);
        // Second loader should not have been called
        assert_eq!(load_count.load(Ordering::SeqCst), 0);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// Test that drop request after reload sets new_intent
    #[tokio::test]
    async fn test_drop_request_after_reload_sets_new_intent() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());
        let drop2_called = Arc::new(AtomicBool::new(false));

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request first drop BEFORE releasing guard
        let _drop_result1 = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Release guard in a spawned task so we can do operations while the drop_fn is running
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Get sets reload
        let reload_notify = Arc::new(Notify::new());
        let result1 = store.get_loaded_or_insert_loading(
            1,
            &input,
            signaled_loader(10, reload_notify.clone()),
        );

        // Second drop request - should set new_intent on reload
        let drop_result2 = store.request_immediate_drop(1, signaling_drop_fn(drop2_called.clone()));

        match drop_result2 {
            RequestImmediateDropResult::ImmediateDropRequested { drop_result } => {
                // Signal first drop to complete
                drop_notify.notify_one();
                // Signal reload to complete
                reload_notify.notify_one();

                // Get the reloaded value (it will be dropped by intent)
                let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
                guard1.async_drop().await.unwrap();

                // Wait for second drop to complete
                drop_result.await;
                assert!(drop2_called.load(Ordering::SeqCst));
            }
            _ => panic!("Expected ImmediateDropRequested"),
        }

        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// Test AlreadyDropping when no reload to attach to
    #[tokio::test]
    async fn test_already_dropping_when_no_reload() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // First drop request BEFORE releasing guard
        let drop_result1 = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Release guard in a spawned task so we can do operations while the drop_fn is running
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Second drop request without any reload in between
        let drop_result2 = store.request_immediate_drop(1, noop_drop_fn());

        match drop_result2 {
            RequestImmediateDropResult::AlreadyDropping { future } => {
                // Signal first drop to complete
                drop_notify.notify_one();

                // Wait for drop should complete when first drop completes
                future.await;
            }
            other => panic!(
                "Expected AlreadyDropping, got {:?}",
                std::mem::discriminant(&other)
            ),
        }

        // Clean up first drop
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result1 {
            drop_result.await;
        }

        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// Scenario 2 from plan: Loaded with intent -> get -> get -> drop -> get
    #[tokio::test]
    async fn test_loaded_with_intent_chain() {
        let mut store = test_store();
        let mut input = test_input();
        let drop1_notify = Arc::new(Notify::new());

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request drop while still holding guard (creates intent)
        let _drop_result1 = store.request_immediate_drop(1, waiting_drop_fn(drop1_notify.clone()));

        // First get - should set reload on the intent
        let reload1_notify = Arc::new(Notify::new());
        let result1 = store.get_loaded_or_insert_loading(
            1,
            &input,
            signaled_loader(10, reload1_notify.clone()),
        );

        // Second get - should add waiter to same reload
        let result2 = store.get_loaded_or_insert_loading(1, &input, simple_loader(20));

        // Release guard in a spawned task so we can signal the notify
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });

        // Give the drop time to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Signal drop to complete
        drop1_notify.notify_one();
        // Signal reload to complete
        reload1_notify.notify_one();

        // Both waiters should get the reload value
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        assert_eq!(guard1.value().id, 10);
        assert_eq!(guard2.value().id, 10);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// Test deep nesting: intent -> reload -> new_intent -> reload
    #[tokio::test]
    async fn test_deep_chain_nesting() {
        let mut store = test_store();
        let mut input = test_input();
        let drop1_notify = Arc::new(Notify::new());
        let drop2_notify = Arc::new(Notify::new());
        let reload1_notify = Arc::new(Notify::new());
        let reload2_notify = Arc::new(Notify::new());

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Drop request 1 (intent)
        let _drop1 = store.request_immediate_drop(1, waiting_drop_fn(drop1_notify.clone()));

        // Get 1 (reload on intent)
        let result1 = store.get_loaded_or_insert_loading(
            1,
            &input,
            signaled_loader(10, reload1_notify.clone()),
        );

        // Drop request 2 (new_intent on reload)
        let _drop2 = store.request_immediate_drop(1, waiting_drop_fn(drop2_notify.clone()));

        // Get 2 (reload on new_intent) - this tests deep chain walking
        let result2 = store.get_loaded_or_insert_loading(
            1,
            &input,
            signaled_loader(20, reload2_notify.clone()),
        );

        // Release original guard in a spawned task so we can signal the notifies
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });

        // Give the drop time to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Complete the chain in order
        drop1_notify.notify_one();
        reload1_notify.notify_one();

        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();

        // guard1's drop also needs to be spawned because it has a new_intent (drop2)
        // that waits for drop2_notify which is signaled after
        let guard1_drop = tokio::spawn(async move { guard1.async_drop().await.unwrap() });
        tokio::time::sleep(Duration::from_millis(10)).await;

        drop2_notify.notify_one();
        reload2_notify.notify_one();

        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard2.value().id, 20);

        guard2.async_drop().await.unwrap();
        guard1_drop.await.unwrap();
        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// Test that when an entry has a reload on drop_intent (R1) and another task
    /// adds a reload on the Dropping state (R2), both waiters get the loaded value
    /// and waiter counts are correctly merged.
    ///
    /// This tests the fix for the bug where R2's waiters would be lost because
    /// make_drop_future_for_loaded_entry only used R1 and ignored R2.
    #[tokio::test]
    async fn test_reload_on_dropping_state_merged_with_intent_reload() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request drop while still holding guard (creates drop_intent on Loaded)
        let _drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // First get while in Loaded state with drop_intent -> creates reload R1 on drop_intent
        let reload_notify = Arc::new(Notify::new());
        let result1 = store.get_loaded_or_insert_loading(
            1,
            &input,
            signaled_loader(10, reload_notify.clone()),
        );

        // Release guard so drop starts -> entry transitions to Dropping
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });

        // Give time for the drop to start and entry to transition to Dropping
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Second get while in Dropping state -> this previously created R2 on Dropping state
        // which was lost. Now it should be merged with R1.
        let load2_count = Arc::new(AtomicUsize::new(0));
        let result2 =
            store.get_loaded_or_insert_loading(1, &input, counting_loader(20, load2_count.clone()));

        // Signal drop to complete
        drop_notify.notify_one();

        // Signal reload to complete
        reload_notify.notify_one();

        // Both waiters should get the reload value from R1
        // If the bug existed, the second waiter would panic when decrementing waiter count
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        assert_eq!(guard1.value().id, 10);
        assert_eq!(guard2.value().id, 10);

        // Note: R2's reload_future will still run (its loader is called, wasted work)
        // but its result is ignored. The waiter counts were merged so both waiters
        // get access without panicking.

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 6: Event Signaling Tests
// ============================================================================

mod event_signaling {
    use super::*;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_on_dropped_triggered_after_drop() {
        let mut store = test_store();
        let mut input = test_input();
        let (tx, rx) = oneshot::channel::<()>();

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        guard.async_drop().await.unwrap();

        // Request drop with signaling
        let drop_result = store.request_immediate_drop(1, move |_value| {
            async move {
                tx.send(()).ok();
            }
            .boxed()
        });

        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result {
            drop_result.await;
            // The channel should have been sent to
            assert!(rx.await.is_ok());
        }

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_reload_future_waits_on_event() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_started = Arc::new(AtomicBool::new(false));
        let reload_started = Arc::new(AtomicBool::new(false));
        let drop_notify = Arc::new(Notify::new());

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request drop that takes time BEFORE releasing guard
        let drop_started_clone = drop_started.clone();
        let drop_notify_clone = drop_notify.clone();
        let _drop_result = store.request_immediate_drop(1, move |value| {
            async move {
                if let Some(mut v) = value {
                    v.async_drop().await.unwrap();
                }
                drop_started_clone.store(true, Ordering::SeqCst);
                drop_notify_clone.notified().await;
            }
            .boxed()
        });

        // Release guard in a spawned task so we can do operations while the drop_fn is running
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Request reload
        let reload_started_clone = reload_started.clone();
        let result1 = store.get_loaded_or_insert_loading(1, &input, move |mut input| {
            async move {
                input.async_drop().await.unwrap();
                reload_started_clone.store(true, Ordering::SeqCst);
                Ok(Some(AsyncDropGuard::new(TestValue::new(10))))
            }
            .boxed()
        });

        // Reload should not start until drop completes
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(drop_started.load(Ordering::SeqCst));
        assert!(!reload_started.load(Ordering::SeqCst));

        // Signal drop to complete
        drop_notify.notify_one();

        // Now reload should proceed
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        assert!(reload_started.load(Ordering::SeqCst));
        assert_eq!(guard1.value().id, 10);

        guard1.async_drop().await.unwrap();
        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 7: Concurrent Access Tests
// ============================================================================

mod concurrent_access {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_gets_same_key() {
        let store = Arc::new(test_store());
        let input = Arc::new(test_input());
        let load_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(10));

        let mut handles = vec![];

        for i in 0..10 {
            let store = Arc::clone(&store);
            let input = Arc::clone(&input);
            let load_count = Arc::clone(&load_count);
            let barrier = Arc::clone(&barrier);

            handles.push(tokio::spawn(async move {
                barrier.wait().await;
                let result =
                    store.get_loaded_or_insert_loading(1, &input, counting_loader(i, load_count));
                result.wait_until_loaded().await.unwrap().unwrap()
            }));
        }

        let guards: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // All guards should have the same id (first loader's value)
        let first_id = guards[0].value().id;
        for guard in &guards {
            assert_eq!(guard.value().id, first_id);
        }

        // Only one loader should have been called
        assert_eq!(load_count.load(Ordering::SeqCst), 1);

        for mut guard in guards {
            guard.async_drop().await.unwrap();
        }

        // Unwrap the Arcs to get the inner values
        let mut input = Arc::try_unwrap(input).unwrap();
        input.async_drop().await.unwrap();
        let mut store = Arc::try_unwrap(store).unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_get_and_drop() {
        let store = Arc::new(test_store());
        let input = Arc::new(test_input());

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();

        let store1 = Arc::clone(&store);
        let store2 = Arc::clone(&store);
        let input1 = Arc::clone(&input);

        // Spawn concurrent get and drop
        let get_handle = tokio::spawn(async move {
            let result = store1.get_loaded_or_insert_loading(1, &input1, simple_loader(10));
            result.wait_until_loaded().await
        });

        let drop_handle =
            tokio::spawn(async move { store2.request_immediate_drop(1, noop_drop_fn()) });

        guard.async_drop().await.unwrap();

        let get_result = get_handle.await;
        let _ = drop_handle.await;

        // Clean up the guard if get succeeded
        if let Ok(Ok(Some(mut guard))) = get_result {
            guard.async_drop().await.unwrap();
        }

        let mut input = Arc::try_unwrap(input).unwrap();
        input.async_drop().await.unwrap();
        let mut store = Arc::try_unwrap(store).unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_concurrent_drops_same_key() {
        let store = Arc::new(test_store());
        let mut input = test_input();

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        guard.async_drop().await.unwrap();

        let store1 = Arc::clone(&store);
        let store2 = Arc::clone(&store);

        let barrier = Arc::new(Barrier::new(2));
        let barrier1 = Arc::clone(&barrier);
        let barrier2 = Arc::clone(&barrier);

        let drop_notify = Arc::new(Notify::new());
        let drop_notify1 = Arc::clone(&drop_notify);

        let handle1 = tokio::spawn(async move {
            barrier1.wait().await;
            store1.request_immediate_drop(1, waiting_drop_fn(drop_notify1))
        });

        let handle2 = tokio::spawn(async move {
            barrier2.wait().await;
            store2.request_immediate_drop(1, noop_drop_fn())
        });

        drop_notify.notify_one();

        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        // One should get ImmediateDropRequested, the other AlreadyDropping
        // (or both ImmediateDropRequested if there was a reload in between)
        // Either way, no panics
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = result1 {
            drop_result.await;
        }
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = result2 {
            drop_result.await;
        }

        input.async_drop().await.unwrap();
        let mut store = Arc::try_unwrap(store).unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 8: Error Handling Tests
// ============================================================================

mod error_handling {
    use super::*;

    #[tokio::test]
    async fn test_loading_error_cleans_up_entry() {
        let mut store = test_store();
        let mut input = test_input();

        let result = store.get_loaded_or_insert_loading(
            1,
            &input,
            failing_loader(TestError("test error".to_string())),
        );

        let err = result.wait_until_loaded().await;
        assert!(err.is_err());

        // Entry should be removed after error
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(store.is_fully_absent(&1));

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    #[tokio::test]
    async fn test_reload_error_removes_entry() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // Load entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request drop BEFORE releasing guard
        let _drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Release guard in a spawned task so we can do operations while the drop_fn is running
        let guard_drop = tokio::spawn(async move {
            let mut guard = guard;
            guard.async_drop().await.unwrap()
        });
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Request reload that will fail
        let reload_result = store.get_loaded_or_insert_loading(
            1,
            &input,
            failing_loader(TestError("reload error".to_string())),
        );

        // Signal drop to complete, starting the reload
        drop_notify.notify_one();

        // Reload should fail
        let err = reload_result.wait_until_loaded().await;
        assert!(err.is_err());

        // Entry should be removed
        tokio::time::sleep(Duration::from_millis(10)).await;
        assert!(store.is_fully_absent(&1));

        guard_drop.await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}

// ============================================================================
// Category 9: Atomicity Tests - State Changes Immediately Visible
// ============================================================================

mod atomicity {
    //! Tests verifying that state changes from one operation are immediately
    //! visible to subsequent operations, even before awaiting futures.
    //!
    //! This is a critical invariant: all public methods are synchronous and
    //! update state under a mutex, so sequential calls on the same thread
    //! must see prior state changes.

    use super::*;

    /// After calling get_loaded_or_insert_loading(), get_if_loading_or_loaded()
    /// should immediately see the Loading state, even before awaiting the future.
    #[tokio::test]
    async fn test_get_insert_immediately_visible_to_get_if() {
        let mut store = test_store();
        let mut input = test_input();
        let notify = Arc::new(Notify::new());

        // Start loading with a signaled loader (that blocks until we signal it)
        let result1 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(1, notify.clone()));

        // Do NOT await result1 - immediately call get_if_loading_or_loaded
        let result2 = store.get_if_loading_or_loaded(1);

        // result2 should see the Loading state and return a waiter, not NotFound
        // We can verify this by checking that awaiting result2 gives us the same value
        // as result1 (not None)

        // Signal the loader to complete
        notify.notify_one();

        // Both should resolve to the same loaded value
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        assert_eq!(guard1.value().id, 1);
        assert_eq!(guard2.value().id, 1);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// After calling get_loaded_or_insert_loading(), a second call should immediately
    /// see the Loading state and not call the loader again.
    #[tokio::test]
    async fn test_get_insert_immediately_visible_to_second_get_insert() {
        let mut store = test_store();
        let mut input = test_input();
        let notify = Arc::new(Notify::new());
        let load_count = Arc::new(AtomicUsize::new(0));

        // Start loading with a signaled loader
        let result1 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(1, notify.clone()));

        // Do NOT await result1 - immediately call get_loaded_or_insert_loading again
        let result2 =
            store.get_loaded_or_insert_loading(1, &input, counting_loader(999, load_count.clone()));

        // The second loader should NOT have been called - state was already Loading
        assert_eq!(load_count.load(Ordering::SeqCst), 0);

        // Signal the first loader to complete
        notify.notify_one();

        // Both should resolve to the same value (from the first loader)
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        assert_eq!(guard1.value().id, 1);
        assert_eq!(guard2.value().id, 1);
        // The counting loader should still not have been called
        assert_eq!(load_count.load(Ordering::SeqCst), 0);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// After calling request_immediate_drop(), is_fully_absent() should immediately
    /// return false (entry is Dropping), even before the drop completes.
    #[tokio::test]
    async fn test_request_drop_immediately_visible_to_is_fully_absent() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // Load an entry and release the guard
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        guard.async_drop().await.unwrap();

        // Wait briefly for the automatic cleanup to potentially start
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Request immediate drop with a waiting drop function
        let drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Do NOT await drop_result - immediately check is_fully_absent
        // The entry should be in Dropping state, so is_fully_absent should return false
        assert!(
            !store.is_fully_absent(&1),
            "Entry should be in Dropping state, not fully absent"
        );

        // Signal the drop to complete
        drop_notify.notify_one();

        // Wait for drop to complete
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result {
            drop_result.await;
        }

        // Now the entry should be fully absent
        assert!(
            store.is_fully_absent(&1),
            "Entry should be fully absent after drop completes"
        );

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// After calling try_insert_loading(), get_if_loading_or_loaded() should immediately
    /// see the Loading state.
    #[tokio::test]
    async fn test_try_insert_loading_immediately_visible_to_get_if() {
        let mut store = test_store();
        let notify = Arc::new(Notify::new());

        // Insert via try_insert_loading with a signaled loader
        let notify_clone = notify.clone();
        let inserting = store
            .try_insert_loading(1, move || {
                let notify = notify_clone;
                async move {
                    notify.notified().await;
                    Ok(AsyncDropGuard::new(TestValue::new(42)))
                }
            })
            .unwrap();

        // Do NOT await inserting - immediately call get_if_loading_or_loaded
        let result = store.get_if_loading_or_loaded(1);

        // result should see the Loading state (not NotFound)
        // Signal the loader to complete
        notify.notify_one();

        // Both should resolve to the same value
        let mut guard1 = inserting.wait_until_inserted().await.unwrap();
        let guard2_result = result.wait_until_loaded().await.unwrap();

        assert!(
            guard2_result.is_some(),
            "get_if_loading_or_loaded should have found the Loading entry"
        );
        let mut guard2 = guard2_result.unwrap();

        assert_eq!(guard1.value().id, 42);
        assert_eq!(guard2.value().id, 42);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// After calling try_insert_loaded(), get_if_loading_or_loaded() should immediately
    /// see the Loaded state.
    #[tokio::test]
    async fn test_try_insert_loaded_immediately_visible_to_get_if() {
        let mut store = test_store();

        // Insert a pre-loaded value
        let value = AsyncDropGuard::new(TestValue::new(42));
        let mut guard1 = store.try_insert_loaded(1, value).unwrap();

        // Immediately call get_if_loading_or_loaded (no async operations in between)
        let result = store.get_if_loading_or_loaded(1);

        // result should see the Loaded state
        let guard2_result = result.wait_until_loaded().await.unwrap();
        assert!(
            guard2_result.is_some(),
            "get_if_loading_or_loaded should have found the Loaded entry"
        );
        let mut guard2 = guard2_result.unwrap();

        assert_eq!(guard1.value().id, 42);
        assert_eq!(guard2.value().id, 42);

        guard1.async_drop().await.unwrap();
        guard2.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// Comprehensive test that chains multiple operations and verifies each sees
    /// the state changes from prior operations.
    #[tokio::test]
    async fn test_sequential_operations_see_prior_state_changes() {
        let mut store = test_store();
        let mut input = test_input();
        let load_notify = Arc::new(Notify::new());
        let drop_notify = Arc::new(Notify::new());

        // Step 1: Start loading
        let result1 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(1, load_notify.clone()));

        // Step 2: Verify get_if_loading_or_loaded sees Loading state
        let result2 = store.get_if_loading_or_loaded(1);
        // We'll verify this resolved correctly after signaling

        // Step 3: Signal loader to complete
        load_notify.notify_one();

        // Verify both results resolve
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard1.value().id, 1);
        assert_eq!(guard2.value().id, 1);

        // Step 4: Release one guard
        guard2.async_drop().await.unwrap();

        // Step 5: get_if_loading_or_loaded should still see Loaded state
        let result3 = store.get_if_loading_or_loaded(1);
        let mut guard3 = result3.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard3.value().id, 1);

        // Step 6: Release remaining guards
        guard1.async_drop().await.unwrap();
        guard3.async_drop().await.unwrap();

        // Wait briefly for automatic drop to potentially start
        tokio::time::sleep(Duration::from_millis(5)).await;

        // Step 7: Request immediate drop
        let drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Step 8: is_fully_absent should return false (Dropping state)
        assert!(
            !store.is_fully_absent(&1),
            "Entry should be in Dropping state after request_immediate_drop"
        );

        // Step 9: Signal drop to complete
        drop_notify.notify_one();
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result {
            drop_result.await;
        }

        // Step 10: is_fully_absent should return true
        assert!(
            store.is_fully_absent(&1),
            "Entry should be fully absent after drop completes"
        );

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// After calling request_immediate_drop() on a Loaded entry, get_if_loading_or_loaded()
    /// should immediately return None (treating it as dropped), even before drop completes.
    #[tokio::test]
    async fn test_request_drop_makes_get_if_return_none() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // Load an entry
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();

        // Request immediate drop with a waiting drop function
        let drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Do NOT await drop_result - immediately call get_if_loading_or_loaded
        // The entry has an intent to drop, so it should be treated as dropped
        let get_result = store.get_if_loading_or_loaded(1);
        let loaded = get_result.wait_until_loaded().await.unwrap();

        assert!(
            loaded.is_none(),
            "get_if_loading_or_loaded should return None for entry with drop intent"
        );

        // Signal drop to complete
        drop_notify.notify_one();

        // Wait for drop to complete
        guard.async_drop().await.unwrap();
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result {
            drop_result.await;
        }

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// After calling request_immediate_drop() on a Loaded entry, get_loaded_or_insert_loading()
    /// should schedule a new load (queued after the drop completes).
    #[tokio::test]
    async fn test_request_drop_makes_get_insert_schedule_new_load() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());
        let load_count = Arc::new(AtomicUsize::new(0));

        // Load an entry with value id=1
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard.value().id, 1);

        // Request immediate drop with a waiting drop function
        let drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // Do NOT await drop_result - immediately call get_loaded_or_insert_loading with a new loader
        // This should schedule a reload with value id=2
        let result2 =
            store.get_loaded_or_insert_loading(1, &input, counting_loader(2, load_count.clone()));

        // The new loader should be called (it's scheduled as a reload)
        // Note: The loader might be called lazily when we await, so we check after signaling

        // Signal drop to complete
        drop_notify.notify_one();

        // Wait for drop to complete
        guard.async_drop().await.unwrap();
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result {
            drop_result.await;
        }

        // Await the reload result - should get the new value (id=2)
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard2.value().id, 2, "Should get the reloaded value");
        assert_eq!(
            load_count.load(Ordering::SeqCst),
            1,
            "The new loader should have been called"
        );

        guard2.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// After the last guard's async_drop().await completes, the entry is fully absent.
    #[tokio::test]
    async fn test_last_guard_async_drop_makes_fully_absent() {
        let mut store = test_store();
        let mut input = test_input();

        // Load an entry and get a guard
        let result1 = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard1 = result1.wait_until_loaded().await.unwrap().unwrap();

        // Get a second guard for the same entry
        let result2 = store.get_if_loading_or_loaded(1);
        let mut guard2 = result2.wait_until_loaded().await.unwrap().unwrap();

        // async_drop first guard - entry should NOT be fully absent yet
        guard1.async_drop().await.unwrap();
        assert!(
            !store.is_fully_absent(&1),
            "Entry should not be fully absent while second guard exists"
        );

        // async_drop second (last) guard - entry should NOW be fully absent
        guard2.async_drop().await.unwrap();

        // Wait briefly for the drop to complete
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(
            store.is_fully_absent(&1),
            "Entry should be fully absent after last guard is dropped"
        );

        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// When an entry has a drop intent but also has a reload scheduled (via get_loaded_or_insert_loading),
    /// get_if_loading_or_loaded should return a waiter for that reload, not None.
    #[tokio::test]
    async fn test_get_if_sees_reload_in_intent_chain() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // 1. Load an entry with value id=1
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard.value().id, 1);

        // 2. Request immediate drop (sets intent on the entry)
        let drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // 3. Call get_loaded_or_insert_loading with new loader (sets reload in intent)
        let reload_result = store.get_loaded_or_insert_loading(1, &input, simple_loader(2));

        // 4. Call get_if_loading_or_loaded - this should see the reload and return a waiter
        let get_if_result = store.get_if_loading_or_loaded(1);

        // 5. Signal drop to complete
        drop_notify.notify_one();

        // Wait for drop to complete
        guard.async_drop().await.unwrap();
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result {
            drop_result.await;
        }

        // 6. get_if_loading_or_loaded should have returned a waiter (Some), not None
        let get_if_loaded = get_if_result.wait_until_loaded().await.unwrap();
        assert!(
            get_if_loaded.is_some(),
            "get_if_loading_or_loaded should return Some when there's a reload in the intent chain"
        );
        let mut get_if_guard = get_if_loaded.unwrap();

        // Should get the reloaded value (id=2)
        assert_eq!(
            get_if_guard.value().id, 2,
            "Should see the reloaded value"
        );

        // Also verify reload_result gets the same value
        let mut reload_guard = reload_result.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(reload_guard.value().id, 2);

        get_if_guard.async_drop().await.unwrap();
        reload_guard.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }

    /// When an entry has a drop intent but also has a reload scheduled (via get_loaded_or_insert_loading),
    /// all_loading_or_loaded should include a waiter for that reload, not exclude the entry.
    #[tokio::test]
    async fn test_all_loading_or_loaded_sees_reload_in_intent_chain() {
        let mut store = test_store();
        let mut input = test_input();
        let drop_notify = Arc::new(Notify::new());

        // 1. Load an entry with value id=1
        let result = store.get_loaded_or_insert_loading(1, &input, simple_loader(1));
        let mut guard = result.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(guard.value().id, 1);

        // 2. Request immediate drop (sets intent on the entry)
        let drop_result = store.request_immediate_drop(1, waiting_drop_fn(drop_notify.clone()));

        // 3. Call get_loaded_or_insert_loading with new loader (sets reload in intent)
        let reload_result = store.get_loaded_or_insert_loading(1, &input, simple_loader(2));

        // 4. Call all_loading_or_loaded - this should see the reload and include it
        let all_entries = store.all_loading_or_loaded();

        // 5. Should return 1 entry (the reload), not empty
        assert_eq!(
            all_entries.len(),
            1,
            "all_loading_or_loaded should return 1 entry when there's a reload in the intent chain"
        );

        // 6. Signal drop to complete
        drop_notify.notify_one();

        // Wait for drop to complete
        guard.async_drop().await.unwrap();
        if let RequestImmediateDropResult::ImmediateDropRequested { drop_result } = drop_result {
            drop_result.await;
        }

        // 7. The entry from all_loading_or_loaded should resolve to the reloaded value
        let mut all_guard = all_entries
            .into_iter()
            .next()
            .unwrap()
            .wait_until_loaded()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(all_guard.value().id, 2, "Should see the reloaded value");

        // Also verify reload_result gets the same value
        let mut reload_guard = reload_result.wait_until_loaded().await.unwrap().unwrap();
        assert_eq!(reload_guard.value().id, 2);

        all_guard.async_drop().await.unwrap();
        reload_guard.async_drop().await.unwrap();
        input.async_drop().await.unwrap();
        store.async_drop().await.unwrap();
    }
}
