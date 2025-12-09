use signal_hook::{
    consts::{SIGINT, SIGQUIT, SIGTERM, TERM_SIGNALS},
    iterator::{Handle, Signals},
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

    signals_handle: Handle,
}

impl AtExitHandler {
    pub fn new(func: impl Fn() + Send + 'static) -> AtExitHandler {
        let mut signals = Signals::new(TERM_SIGNALS).unwrap();
        let signals_handle = signals.handle();
        let join_handle = thread::spawn(move || {
            while !signals.is_closed() {
                for signal in signals.wait() {
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

// TODO Tests
