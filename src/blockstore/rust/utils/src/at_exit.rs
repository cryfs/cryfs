use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM, TERM_SIGNALS},
    iterator::Signals,
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

/// Creating an instance of [AtExitHandler] registers a function to be run
/// when the process receives a SIGTERM, SIGINT, or SIGQUIT signal.
/// The function is run in a separate thread.
///
/// Dropping the [AtExitHandler] instance will unregister the signal handler.
pub struct AtExitHandler {
    // Always Some except during drop
    join_handle: Option<JoinHandle<()>>,

    cancellation_token: CancellationToken,
}

impl AtExitHandler {
    pub fn new(func: impl Fn() + Send + 'static) -> AtExitHandler {
        let cancellation_token = CancellationToken::new();
        let cancellation_token_clone = cancellation_token.clone();
        let join_handle = thread::spawn(move || {
            let mut signals = Signals::new(TERM_SIGNALS).unwrap();
            while !cancellation_token_clone.is_cancelled() {
                for signal in signals.pending() {
                    let signal_name = match signal {
                        SIGTERM => "SIGTERM".to_string(),
                        SIGINT => "SIGINT".to_string(),
                        SIGQUIT => "SIGQUIT".to_string(),
                        _ => format!("signal {}", signal),
                    };
                    log::info!("Received {signal_name}");
                    func();
                }
            }
        });
        AtExitHandler {
            join_handle: Some(join_handle),
            cancellation_token,
        }
    }
}

impl Drop for AtExitHandler {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
        self.join_handle
            .take()
            .expect("Already destructed")
            .join()
            .unwrap();
    }
}

#[derive(Clone)]
struct CancellationToken {
    should_cancel: Arc<AtomicBool>,
}

impl CancellationToken {
    fn new() -> Self {
        Self {
            should_cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    fn cancel(&self) {
        self.should_cancel.store(true, Ordering::Relaxed);
    }

    fn is_cancelled(&self) -> bool {
        self.should_cancel.load(Ordering::Relaxed)
    }
}

// TODO Tests
