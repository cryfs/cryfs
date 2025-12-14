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
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
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
    fn test_signal_handler(#[values(SIGTERM, SIGINT, SIGQUIT)] signal: i32) {
        signal_test(move || {
            let called = Arc::new(AtomicBool::new(false));
            let called_clone = called.clone();

            let _handler = AtExitHandler::new("test", move || {
                called_clone.store(true, Ordering::SeqCst);
            });

            // Send signal to ourselves
            unsafe {
                libc::raise(signal);
            }

            // Wait for the signal to be processed
            thread::sleep(Duration::from_millis(100));

            assert!(called.load(Ordering::SeqCst), "handler was not called",);
        });
    }

    #[test]
    fn test_multiple_signals() {
        signal_test(|| {
            let call_count = Arc::new(AtomicUsize::new(0));
            let call_count_clone = call_count.clone();

            let _handler = AtExitHandler::new("test", move || {
                call_count_clone.fetch_add(1, Ordering::SeqCst);
            });

            // Send multiple signals
            unsafe {
                libc::raise(SIGTERM);
            }
            thread::sleep(Duration::from_millis(50));

            sleep_to_not_trigger_double_signal_handler();

            unsafe {
                libc::raise(SIGINT);
            }
            thread::sleep(Duration::from_millis(50));

            let count = call_count.load(Ordering::SeqCst);
            assert!(
                count >= 2,
                "Handler should be called multiple times, got {}",
                count
            );
        });
    }

    #[test]
    fn test_handler_with_complex_callback() {
        signal_test(|| {
            let messages = Arc::new(std::sync::Mutex::new(Vec::new()));
            let messages_clone = messages.clone();

            let _handler = AtExitHandler::new("test", move || {
                messages_clone
                    .lock()
                    .unwrap()
                    .push("Signal received".to_string());
            });

            unsafe {
                libc::raise(SIGTERM);
            }
            thread::sleep(Duration::from_millis(100));

            let msgs = messages.lock().unwrap();
            assert_eq!(msgs.len(), 1);
            assert_eq!(msgs[0], "Signal received");
        });
    }

    #[test]
    fn multiple_handlers() {
        signal_test(|| {
            let call_count1 = Arc::new(AtomicUsize::new(0));
            let call_count1_clone = call_count1.clone();

            let call_count2 = Arc::new(AtomicUsize::new(0));
            let call_count2_clone = call_count2.clone();

            let call_count3 = Arc::new(AtomicUsize::new(0));
            let call_count3_clone = call_count3.clone();

            let _handler1 = AtExitHandler::new("test", move || {
                call_count1_clone.fetch_add(1, Ordering::SeqCst);
            });

            let _handler2 = AtExitHandler::new("test", move || {
                call_count2_clone.fetch_add(1, Ordering::SeqCst);
            });

            let _handler3 = AtExitHandler::new("test", move || {
                call_count3_clone.fetch_add(1, Ordering::SeqCst);
            });

            unsafe {
                libc::raise(SIGINT);
            }
            thread::sleep(Duration::from_millis(100));

            assert_eq!(
                call_count1.load(Ordering::SeqCst),
                1,
                "First handler was not called"
            );
            assert_eq!(
                call_count2.load(Ordering::SeqCst),
                1,
                "Second handler was not called"
            );
            assert_eq!(
                call_count3.load(Ordering::SeqCst),
                1,
                "Third handler was not called"
            );
        });
    }

    #[test]
    fn test_handler_drop_before_signal() {
        signal_test(|| {
            let _handler = AtExitHandler::new("test", || {
                // Extra handler to ensure that the process doesn't crash
                // even after the main handler is dropped
            });

            let called = Arc::new(AtomicUsize::new(0));
            let called_clone = called.clone();
            let handler = AtExitHandler::new("test", move || {
                called.fetch_add(1, Ordering::SeqCst);
            });

            unsafe {
                libc::raise(SIGINT);
            }
            thread::sleep(Duration::from_millis(100));
            assert_eq!(
                called_clone.load(Ordering::SeqCst),
                1,
                "Handler was not called before drop"
            );

            // Wait a bit to ensure we don't trigger the double signal handler
            sleep_to_not_trigger_double_signal_handler();

            drop(handler);
            unsafe {
                libc::raise(SIGINT);
            }
            thread::sleep(Duration::from_millis(100));
            assert_eq!(
                called_clone.load(Ordering::SeqCst),
                1,
                "Handler was called after drop"
            );
        });
    }

    #[test]
    fn test_thread_name() {
        signal_test(|| {
            let thread_name = Arc::new(std::sync::Mutex::new(None));
            let thread_name_clone = thread_name.clone();

            let _handler = AtExitHandler::new("my-custom-handler", move || {
                let name = thread::current().name().map(|s| s.to_string());
                *thread_name_clone.lock().unwrap() = name;
            });

            unsafe {
                libc::raise(SIGINT);
            }
            thread::sleep(Duration::from_millis(100));

            let captured_name = thread_name.lock().unwrap();
            assert_eq!(
                captured_name.as_deref(),
                Some("atexit:my-custom-handler"),
                "Thread name should be 'atexit:my-custom-handler'"
            );
        });
    }
}
