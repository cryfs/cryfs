use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM, TERM_SIGNALS},
    iterator::{Handle, Signals},
};
use std::{
    sync::LazyLock,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

const DOUBLE_SIGNAL_THRESHOLD: Duration = Duration::from_secs(1);

/// A global [AtExitHandler] that exits the process immediately if a second
/// termination signal (e.g. ctrl+c) is received within 1 second of the first.
static DOUBLE_SIGNAL_HANDLER: LazyLock<AtExitHandler> = LazyLock::new(|| {
    let mut last_term_signal_time: Option<Instant> = None;

    AtExitHandler::_new("double-signal-handler", move || {
        let now = Instant::now();
        if let Some(last_term_signal_time) = last_term_signal_time {
            let elapsed = now.duration_since(last_term_signal_time);
            if elapsed < DOUBLE_SIGNAL_THRESHOLD {
                log::warn!("Received double signal. Exiting immediately.");
                std::process::exit(1);
            }
        }
        last_term_signal_time = Some(now);
    })
});

/// Creating an instance of [AtExitHandler] registers a function to be run
/// when the process receives a SIGTERM, SIGINT, or SIGQUIT signal.
/// The function is run in a separate thread.
///
/// Dropping the [AtExitHandler] instance will unregister the signal handler.
pub struct AtExitHandler {
    // Always Some except during drop
    join_handle: Option<JoinHandle<()>>,

    signals_handle: Handle,
}

impl AtExitHandler {
    pub fn new(name: &str, func: impl FnMut() + Send + 'static) -> AtExitHandler {
        // Ensure the double signal handler is initialized
        LazyLock::force(&DOUBLE_SIGNAL_HANDLER);
        Self::_new(name, func)
    }

    fn _new(name: &str, mut func: impl FnMut() + Send + 'static) -> AtExitHandler {
        let mut signals = Signals::new(TERM_SIGNALS).unwrap();
        let signals_handle = signals.handle();

        let join_handle = thread::Builder::new()
            .name(format!("atexit:{name}"))
            .spawn(move || {
                while !signals.is_closed() {
                    for signal in signals.wait() {
                        let signal_name = match signal {
                            SIGTERM => "SIGTERM".to_string(),
                            SIGINT => "SIGINT".to_string(),
                            SIGQUIT => "SIGQUIT".to_string(),
                            _ => format!("signal {}", signal),
                        };
                        log::warn!("Received {signal_name}");
                        func();
                    }
                }
            })
            .expect("Failed to spawn AtExitHandler thread");
        AtExitHandler {
            join_handle: Some(join_handle),
            signals_handle,
        }
    }
}

impl Drop for AtExitHandler {
    fn drop(&mut self) {
        self.signals_handle.close();
        self.join_handle
            .take()
            .expect("Already destructed")
            .join()
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    use super::*;
    use rstest::rstest;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    fn sleep_to_not_trigger_double_signal_handler() {
        std::thread::sleep(Duration::from_secs_f32(
            DOUBLE_SIGNAL_THRESHOLD.as_secs_f32() * 1.5,
        ));
    }

    fn signal_test(test_fn: impl FnOnce() + Send + 'static) {
        // Ensure only one signal test is running at a time
        let _guard = LOCK.lock().unwrap();

        test_fn();

        // Wait a bit to ensure we don't trigger the double signal handler in the very next test
        sleep_to_not_trigger_double_signal_handler();
    }

    #[test]
    fn test_create_and_drop() {
        signal_test(|| {
            // Test that we can create and drop the handler without panicking
            let called = Arc::new(AtomicBool::new(false));
            let called_clone = called.clone();

            let handler = AtExitHandler::new("test", move || {
                called_clone.store(true, Ordering::SeqCst);
            });

            drop(handler);
            // Handler should be cleanly dropped

            assert!(
                !called.load(Ordering::SeqCst),
                "Handler should not be called on drop"
            );
        });
    }

    #[rstest]
    #[ignore] // Temporarily disabled: sends process-wide signals that interfere with other tests
    fn test_signal_handler(#[values(SIGTERM, SIGINT, SIGQUIT)] signal: i32) {
        signal_test(move || {
            let (tx, rx) = std::sync::mpsc::channel();

            let _handler = AtExitHandler::new("test", move || {
                tx.send(()).unwrap();
            });

            // Send signal to ourselves
            unsafe {
                libc::raise(signal);
            }

            // Wait for the handler to be called
            rx.recv_timeout(Duration::from_secs(10))
                .expect("Handler was not called within timeout");
        });
    }

    #[test]
    #[ignore] // Temporarily disabled: sends process-wide signals that interfere with other tests
    fn test_multiple_signals() {
        signal_test(|| {
            let (tx, rx) = std::sync::mpsc::channel();

            let _handler = AtExitHandler::new("test", move || {
                tx.send(()).unwrap();
            });

            // Send first signal
            unsafe {
                libc::raise(SIGTERM);
            }

            sleep_to_not_trigger_double_signal_handler();

            // Send second signal
            unsafe {
                libc::raise(SIGINT);
            }

            // Wait for first signal to be processed
            rx.recv_timeout(Duration::from_secs(10))
                .expect("First signal was not handled");

            // Wait for second signal to be processed
            rx.recv_timeout(Duration::from_secs(10))
                .expect("Second signal was not handled");
        });
    }

    #[test]
    fn test_handler_with_complex_callback() {
        signal_test(|| {
            let (tx, rx) = std::sync::mpsc::channel();

            let _handler = AtExitHandler::new("test", move || {
                tx.send("Signal received".to_string()).unwrap();
            });

            unsafe {
                libc::raise(SIGTERM);
            }

            let msg = rx
                .recv_timeout(Duration::from_secs(10))
                .expect("Handler was not called");
            assert_eq!(msg, "Signal received");
        });
    }

    #[test]
    fn multiple_handlers() {
        signal_test(|| {
            use std::sync::Barrier;

            let barrier = Arc::new(Barrier::new(4)); // 3 handlers + 1 main thread

            let barrier1 = barrier.clone();
            let _handler1 = AtExitHandler::new("test", move || {
                barrier1.wait();
            });

            let barrier2 = barrier.clone();
            let _handler2 = AtExitHandler::new("test", move || {
                barrier2.wait();
            });

            let barrier3 = barrier.clone();
            let _handler3 = AtExitHandler::new("test", move || {
                barrier3.wait();
            });

            unsafe {
                libc::raise(SIGINT);
            }

            // Wait for all handlers to be called with a timeout
            let barrier_clone = barrier.clone();
            let result = std::thread::spawn(move || barrier_clone.wait())
                .join()
                .expect("Barrier wait failed");

            // If we get here, all handlers were called
            assert!(result.is_leader() || !result.is_leader()); // Just to use the result
        });
    }

    #[test]
    fn test_handler_drop_before_signal() {
        signal_test(|| {
            let (dummy_tx, _dummy_rx) = std::sync::mpsc::channel();
            let _handler = AtExitHandler::new("test", move || {
                // Extra handler to ensure that the process doesn't crash
                // even after the main handler is dropped
                let _ = dummy_tx.send(());
            });

            let (tx, rx) = std::sync::mpsc::channel();
            let handler = AtExitHandler::new("test", move || {
                tx.send(()).unwrap();
            });

            unsafe {
                libc::raise(SIGINT);
            }

            // Wait for handler to be called
            rx.recv_timeout(Duration::from_secs(10))
                .expect("Handler was not called before drop");

            // Wait to avoid triggering double signal handler
            sleep_to_not_trigger_double_signal_handler();

            drop(handler);
            unsafe {
                libc::raise(SIGINT);
            }

            // Verify handler is NOT called after drop
            assert!(
                rx.recv_timeout(Duration::from_secs(1)).is_err(),
                "Handler should not be called after drop"
            );
        });
    }

    #[test]
    fn test_thread_name() {
        signal_test(|| {
            let (tx, rx) = std::sync::mpsc::channel();

            let _handler = AtExitHandler::new("my-custom-handler", move || {
                let name = thread::current().name().map(|s| s.to_string());
                tx.send(name).unwrap();
            });

            unsafe {
                libc::raise(SIGINT);
            }

            let thread_name = rx
                .recv_timeout(Duration::from_secs(10))
                .expect("Handler was not called");

            assert_eq!(
                thread_name.as_deref(),
                Some("atexit:my-custom-handler"),
                "Thread name should be 'atexit:my-custom-handler'"
            );
        });
    }
}
