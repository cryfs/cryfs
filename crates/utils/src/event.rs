//! One-time async event for task synchronization.
//!
//! This module provides [`Event`], an async primitive that allows multiple tasks
//! to wait until a single event is triggered. Once triggered, all waiting tasks
//! are notified and future waiters return immediately.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// An async event that multiple tasks can wait on and be notified when it is triggered.
/// This is a one-time event. Once triggered, it cannot be reset.
/// It can be cloned and shared across multiple tasks.
#[derive(Clone)]
pub struct Event {
    inner: Arc<EventImpl>,
}

impl Default for Event {
    fn default() -> Self {
        Self::new()
    }
}

impl Event {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(EventImpl {
                triggered: AtomicBool::new(false),
                notify: tokio::sync::Notify::new(),
            }),
        }
    }

    /// Trigger the event, notifying all tasks waiting on it.
    /// If the event was already triggered before, this will do nothing.
    pub fn trigger(&self) {
        if !self.inner.triggered.swap(true, Ordering::Release) {
            self.inner.notify.notify_waiters();
        }
    }

    /// Wait until the event is triggered.
    /// If the event was already triggered before, this will return immediately.
    pub async fn wait(&self) {
        // We need to create the notifier before checking `triggered` to avoid a race condition
        // where we check triggered, it's false, then another task sets it to true, notifies all waiters,
        // and then we start waiting, missing the notification.
        let notifier = self.inner.notify.notified();
        if !self.inner.triggered.load(Ordering::Acquire) {
            notifier.await;
        }
    }
}

struct EventImpl {
    triggered: AtomicBool,
    notify: tokio::sync::Notify,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::time::Duration;

    #[tokio::test]
    async fn test_trigger_then_wait_returns_immediately() {
        let event = Event::new();

        event.trigger();
        // Should return immediately since already triggered
        event.wait().await;
    }

    #[tokio::test]
    async fn test_wait_then_trigger_wakes_waiter() {
        let event = Event::new();
        let event_clone = event.clone();

        let handle = tokio::spawn(async move {
            event_clone.wait().await;
            true
        });

        // Give the task time to start waiting
        tokio::time::sleep(Duration::from_millis(10)).await;

        event.trigger();

        let result = tokio::time::timeout(Duration::from_millis(100), handle)
            .await
            .expect("Timed out waiting for task")
            .expect("Task panicked");

        assert!(result);
    }

    #[tokio::test]
    async fn test_multiple_waiters_all_notified() {
        let event = Event::new();
        let counter = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..5 {
            let event_clone = event.clone();
            let counter_clone = Arc::clone(&counter);
            handles.push(tokio::spawn(async move {
                event_clone.wait().await;
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }));
        }

        // Give tasks time to start waiting
        tokio::time::sleep(Duration::from_millis(10)).await;

        event.trigger();

        for handle in handles {
            tokio::time::timeout(Duration::from_millis(100), handle)
                .await
                .expect("Timed out")
                .expect("Task panicked");
        }

        assert_eq!(5, counter.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_trigger_twice_is_idempotent() {
        let event = Event::new();

        event.trigger();
        event.trigger(); // Should not panic or cause issues

        event.wait().await; // Should still work
    }

    #[tokio::test]
    async fn test_clone_shares_same_event() {
        let event1 = Event::new();
        let event2 = event1.clone();

        event1.trigger();

        // Both clones should see the triggered state
        event2.wait().await;
    }

    #[test]
    fn test_default_creates_untriggered_event() {
        let event = Event::default();
        // The event should be created but not triggered
        // We can't easily test this synchronously, but we verify it compiles
        assert!(!event.inner.triggered.load(Ordering::Acquire));
    }
}
