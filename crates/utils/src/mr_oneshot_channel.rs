use derive_more::{Display, Error};
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

/// Create a multi-receiver oneshot channel. There can only be one sender who can only send one time,
/// but there can be multiple receivers who will all get the sent value.
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner {
        state: Mutex::new(State::Empty),
        notify: Notify::new(),
    });

    let sender = Sender {
        inner: Arc::clone(&inner),
    };
    let receiver = Receiver { inner };

    (sender, receiver)
}

/// Sender side of a multi-receiver oneshot channel.
/// Not cloneable and consumes itself when sending to ensure single use.
pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

/// Receiver side of a multi-receiver oneshot channel.
/// Can be cloned to create multiple receivers that all receive the same value.
#[derive(Clone)]
pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

struct Inner<T> {
    state: Mutex<State<T>>,
    notify: Notify,
}

enum State<T> {
    /// No value has been sent yet
    Empty,
    /// A value has been sent and is available to all receivers
    Filled(T),
    /// The sender was dropped without sending a value
    Closed,
}

/// Error returned when the sender is dropped without sending a value
#[derive(Debug, Display, Error, Clone, Copy, PartialEq, Eq)]
#[display("Channel closed without a value being sent")]
pub struct RecvError;

impl<T> Sender<T> {
    /// Send a value to all receivers.
    /// Consumes the sender to ensure it can only be used once.
    /// Returns an error with the value if the channel is already closed.
    pub fn send(self, value: T) {
        let mut state = self.inner.state.lock().unwrap();

        // Check if we're in the Empty state (the only valid state for sending)
        if !matches!(*state, State::Empty) {
            panic!("Sender can only send once and only if not closed");
        }

        // Store the value so all receivers can clone it
        *state = State::Filled(value);

        // Release the lock before notifying to avoid holding it during notification
        drop(state);

        // Wake up all waiting receivers
        self.inner.notify.notify_waiters();
    }

    /// Create a new receiver that can receive the sent value.
    pub fn subscribe(&self) -> Receiver<T> {
        Receiver {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // If the sender is dropped without sending, mark the channel as closed
        // so receivers don't wait forever
        let mut state = self.inner.state.lock().unwrap();
        if matches!(*state, State::Empty) {
            *state = State::Closed;
            drop(state);
            self.inner.notify.notify_waiters();
        }
    }
}

impl<T: Clone> Receiver<T> {
    /// Receive the value sent by the sender.
    /// Asynchronously waits until a value is available or the sender is dropped.
    /// Returns a clone of the value for this receiver.
    pub async fn recv(&self) -> Result<T, RecvError> {
        loop {
            // IMPORTANT: Must call notified() BEFORE checking state to avoid race condition.
            // This ensures we don't miss notifications that happen between checking state
            // and waiting.
            let notified = self.inner.notify.notified();

            // Check current state in a scope to ensure MutexGuard is dropped before await
            let should_wait = {
                let state = self.inner.state.lock().unwrap();
                match &*state {
                    State::Filled(value) => {
                        // Value is available, return a clone
                        return Ok(value.clone());
                    }
                    State::Closed => {
                        // Sender was dropped without sending
                        return Err(RecvError);
                    }
                    State::Empty => {
                        // No value yet, need to wait
                        true
                    }
                }
            }; // MutexGuard is dropped here

            if should_wait {
                // Wait for notification
                notified.await;
                // Loop back to check state again
            }
        }
    }

    /// Try to receive the value without blocking.
    /// Returns None if no value is available yet.
    pub fn try_recv(&self) -> Result<Option<T>, RecvError> {
        let state = self.inner.state.lock().unwrap();

        match &*state {
            State::Filled(value) => Ok(Some(value.clone())),
            State::Closed => Err(RecvError),
            State::Empty => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_basic_send_recv() {
        let (sender, receiver) = channel();
        sender.send(42);
        assert_eq!(receiver.recv().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_multiple_receivers() {
        let (sender, receiver1) = channel();
        let receiver2 = receiver1.clone();
        let receiver3 = receiver1.clone();

        sender.send(100);

        assert_eq!(receiver1.recv().await.unwrap(), 100);
        assert_eq!(receiver2.recv().await.unwrap(), 100);
        assert_eq!(receiver3.recv().await.unwrap(), 100);
    }

    #[tokio::test]
    async fn test_recv_before_send() {
        let (sender, receiver) = channel();

        let handle = tokio::spawn(async move { receiver.recv().await.unwrap() });

        sleep(Duration::from_millis(10)).await;
        sender.send(42);

        assert_eq!(handle.await.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_sender_dropped_without_sending() {
        let (sender, receiver) = channel::<i32>();
        drop(sender);
        assert_eq!(receiver.recv().await, Err(RecvError));
    }

    #[tokio::test]
    async fn test_try_recv() {
        let (sender, receiver) = channel();

        // No value yet
        assert_eq!(receiver.try_recv(), Ok(None));

        sender.send(42);

        // Value available
        assert_eq!(receiver.try_recv().unwrap().unwrap(), 42);
    }

    #[tokio::test]
    async fn test_multiple_receivers_concurrent() {
        let (sender, receiver) = channel();

        let mut handles = vec![];
        for _ in 0..10 {
            let rx = receiver.clone();
            handles.push(tokio::spawn(async move { rx.recv().await.unwrap() }));
        }

        sleep(Duration::from_millis(10)).await;
        sender.send(42);

        for handle in handles {
            assert_eq!(handle.await.unwrap(), 42);
        }
    }

    #[tokio::test]
    async fn test_sender_not_cloneable() {
        let (sender, _receiver) = channel::<i32>();
        // This should not compile:
        // let _sender2 = sender.clone();
        drop(sender);
    }

    #[tokio::test]
    async fn test_recv_after_value_sent() {
        let (sender, receiver) = channel();
        sender.send(42);

        // Multiple receivers can all receive after the value was sent
        let receiver2 = receiver.clone();
        assert_eq!(receiver.recv().await.unwrap(), 42);
        assert_eq!(receiver2.recv().await.unwrap(), 42);
    }
}
