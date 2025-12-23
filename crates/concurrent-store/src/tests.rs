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

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
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

    fn with_drop_tracker(id: usize, dropped: Arc<AtomicBool>) -> Self {
        Self { id, dropped }
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
) -> futures::future::BoxFuture<'static, Result<Option<AsyncDropGuard<TestValue>>, TestError>>
{
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
) -> futures::future::BoxFuture<'static, Result<Option<AsyncDropGuard<TestValue>>, TestError>>
{
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
) -> futures::future::BoxFuture<'static, Result<Option<AsyncDropGuard<TestValue>>, TestError>>
{
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
) -> futures::future::BoxFuture<'static, Result<Option<AsyncDropGuard<TestValue>>, TestError>>
{
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
) -> futures::future::BoxFuture<'static, Result<Option<AsyncDropGuard<TestValue>>, TestError>>
{
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
fn noop_drop_fn(
) -> impl FnOnce(Option<AsyncDropGuard<TestValue>>) -> futures::future::BoxFuture<'static, ()>
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
        let result1 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(10, reload_notify.clone()));

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
        let result1 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(10, reload1_notify.clone()));

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
        let result1 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(10, reload1_notify.clone()));

        // Drop request 2 (new_intent on reload)
        let _drop2 = store.request_immediate_drop(1, waiting_drop_fn(drop2_notify.clone()));

        // Get 2 (reload on new_intent) - this tests deep chain walking
        let result2 =
            store.get_loaded_or_insert_loading(1, &input, signaled_loader(20, reload2_notify.clone()));

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
