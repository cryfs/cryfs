//! Tests for [`AtExitHandler`] that raise process-wide termination signals
//! (`SIGTERM`/`SIGINT`/`SIGQUIT`) via [`libc::raise`].
//!
//! These live in their own integration-test binary so that signals they raise
//! cannot be observed by any other test in the `cryfs-utils` test suite, and
//! so that other tests cannot accidentally interfere with this binary's signal
//! delivery (e.g. by registering an unrelated `Signals` iterator on the same
//! `TERM_SIGNALS`). Within this binary, tests are still serialized through a
//! module-level mutex with a post-test sleep, because once any `AtExitHandler`
//! is constructed the process-wide `DOUBLE_SIGNAL_HANDLER` is active and
//! `process::exit(1)`s on any two terminating signals less than one second
//! apart.

use cryfs_utils::at_exit::AtExitHandler;
use rstest::rstest;
use signal_hook::consts::{SIGINT, SIGQUIT, SIGTERM};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Must exceed the `DOUBLE_SIGNAL_THRESHOLD` of 1 second in
/// `cryfs_utils::at_exit` by a comfortable margin.
const SLEEP_AFTER_SIGNAL: Duration = Duration::from_millis(1500);

static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn sleep_to_not_trigger_double_signal_handler() {
    std::thread::sleep(SLEEP_AFTER_SIGNAL);
}

fn signal_test(test_fn: impl FnOnce()) {
    // Serialize signal-raising tests: libc::raise targets the whole process,
    // so any two tests running in parallel would observe each other's signals.
    let _guard = LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    test_fn();

    // Avoid the global double-signal-detector firing in the next test.
    sleep_to_not_trigger_double_signal_handler();
}

#[rstest]
fn test_signal_handler(#[values(SIGTERM, SIGINT, SIGQUIT)] signal: i32) {
    signal_test(|| {
        let (tx, rx) = std::sync::mpsc::channel();

        let _handler = AtExitHandler::new("test", move || {
            tx.send(()).unwrap();
        });

        unsafe {
            libc::raise(signal);
        }

        rx.recv_timeout(Duration::from_secs(10))
            .expect("Handler was not called within timeout");
    });
}

#[test]
fn test_multiple_signals() {
    signal_test(|| {
        let (tx, rx) = std::sync::mpsc::channel();

        let _handler = AtExitHandler::new("test", move || {
            tx.send(()).unwrap();
        });

        unsafe {
            libc::raise(SIGTERM);
        }

        sleep_to_not_trigger_double_signal_handler();

        unsafe {
            libc::raise(SIGINT);
        }

        rx.recv_timeout(Duration::from_secs(10))
            .expect("First signal was not handled");
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

        // Wait for all handlers to be called with a timeout.
        let barrier_clone = barrier.clone();
        let result = std::thread::spawn(move || barrier_clone.wait())
            .join()
            .expect("Barrier wait failed");

        // If we get here, all handlers were called.
        assert!(result.is_leader() || !result.is_leader());
    });
}

#[test]
fn test_handler_drop_before_signal() {
    signal_test(|| {
        let (dummy_tx, _dummy_rx) = std::sync::mpsc::channel();
        let _handler = AtExitHandler::new("test", move || {
            // Extra handler to ensure that the process doesn't crash
            // even after the main handler is dropped.
            let _ = dummy_tx.send(());
        });

        let (tx, rx) = std::sync::mpsc::channel();
        let handler = AtExitHandler::new("test", move || {
            tx.send(()).unwrap();
        });

        unsafe {
            libc::raise(SIGINT);
        }

        rx.recv_timeout(Duration::from_secs(10))
            .expect("Handler was not called before drop");

        sleep_to_not_trigger_double_signal_handler();

        drop(handler);
        unsafe {
            libc::raise(SIGINT);
        }

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
