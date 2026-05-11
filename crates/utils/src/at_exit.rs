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

// Tests that exercise actual signal delivery — i.e. anything that calls
// `libc::raise(SIGTERM/SIGINT/SIGQUIT)` — do NOT live here. They are in
// `crates/utils/tests/at_exit_signals.rs`, which compiles to its own
// integration-test binary so that signals raised by those tests cannot be
// observed by — or interfere with — any other test in the cryfs-utils
// unit-test binary.
//
// The tests that moved out:
//   - test_signal_handler (SIGTERM / SIGINT / SIGQUIT)
//   - test_multiple_signals
//   - test_handler_with_complex_callback
//   - multiple_handlers
//   - test_handler_drop_before_signal
//   - test_thread_name
//
// Only signal-free tests belong in this module.

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn test_create_and_drop() {
        // Test that we can create and drop the handler without panicking.
        // No signals are raised, so this is safe to run in the unit-test binary.
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let handler = AtExitHandler::new("test", move || {
            called_clone.store(true, Ordering::SeqCst);
        });

        drop(handler);

        assert!(
            !called.load(Ordering::SeqCst),
            "Handler should not be called on drop"
        );
    }
}
