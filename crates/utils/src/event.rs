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

// TODO Tests
