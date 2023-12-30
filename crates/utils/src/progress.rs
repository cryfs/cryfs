use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

// TODO Use [https://docs.rs/indicatif-log-bridge/latest/indicatif_log_bridge/] so log messages don't destroy the progress bar

const AUTOTICK_INTERVAL: Duration = Duration::from_millis(50);

/// A [Spinner] is a progress bar with an unknown duration / end point.
/// It doesn't know when it's going to be finished or how long it will take
/// and will just show a general spinning animation.
///
/// It can be cloned and the clones will all refer to the same spinner.
#[derive(Clone)]
pub struct Spinner {
    // [indicatif::ProgressBar] is itself an [Arc] and can be cloned, but we still need our own
    // [Arc] so that our [Drop] behavior only drops after the last clone is dropped.
    pb: Arc<ProgressImpl>,
}

impl Spinner {
    pub fn new_autotick(message: &'static str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_message(format!("{message}..."));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}").unwrap(),
        );
        pb.enable_steady_tick(AUTOTICK_INTERVAL);
        Self {
            pb: Arc::new(ProgressImpl::new(message, pb)),
        }
    }

    /// Will panic if there are still other clones referencing the same spinner.
    pub fn finish(self) {
        let pb = Arc::into_inner(self.pb)
            .expect("Called `Spinner.finish` while other instances of the spinner still exist");
        std::mem::drop(pb);
    }
}

/// A [Progress] is a progress bar with a clear end point and current state, i.e.
/// it always knows that x/total steps are already finished and can show a progress
/// bar including time estimates to the user.
///
/// It can be cloned and the clones will all refer to the same progress bar.
#[derive(Clone)]
pub struct Progress {
    // [indicatif::ProgressBar] is itself an [Arc] and can be cloned, but we still need our own
    // [Arc] so that our [Drop] behavior only drops after the last clone is dropped.
    pb: Arc<ProgressImpl>,
}

impl Progress {
    pub fn new(message: &'static str, total: u64) -> Self {
        let pb = ProgressBar::new(total);
        pb.set_message(format!("{message}..."));
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg} [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
        );
        pb.tick();
        Self {
            pb: Arc::new(ProgressImpl::new(message, pb)),
        }
    }

    pub fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }

    /// Will panic if there are still other clones referencing the same progress bar.
    pub fn finish(self) {
        let pb = Arc::into_inner(self.pb).expect(
            "Called `Progress.finish` while other instances of the progress bar still exist",
        );
        std::mem::drop(pb);
    }
}

struct ProgressImpl {
    message: &'static str,
    pb: ProgressBar,
}

impl ProgressImpl {
    pub fn new(message: &'static str, pb: ProgressBar) -> Self {
        Self { message, pb }
    }

    pub fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }
}

impl Drop for ProgressImpl {
    fn drop(&mut self) {
        self.pb
            .finish_with_message(format!("{}...done", self.message));
    }
}
