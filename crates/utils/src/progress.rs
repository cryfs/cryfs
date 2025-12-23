//! Progress bar and spinner abstractions.
//!
//! This module provides traits and implementations for showing progress to users.
//! It includes:
//!
//! - [`ProgressBarManager`]: Factory trait for creating spinners and progress bars
//! - [`Spinner`]: A progress indicator for tasks with unknown duration
//! - [`Progress`]: A progress bar for tasks with known total steps
//!
//! Two implementations are provided:
//! - Console implementations that display visual progress using the `indicatif` crate
//! - Silent implementations that do nothing (useful for non-interactive contexts or tests)

use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;
use std::time::Duration;

const AUTOTICK_INTERVAL: Duration = Duration::from_millis(50);

/// Overarching manager trait that allows creating new progress bars and spinners.
pub trait ProgressBarManager: Clone + Copy + Send + Sync {
    type Spinner: Spinner;
    type Progress: Progress;

    fn new_spinner_autotick(&self, message: &'static str) -> Self::Spinner;
    fn new_progress_bar(&self, message: &'static str, total: u64) -> Self::Progress;
}

/// A [ProgressBarManager] that shows progress bars and spinners on the console
#[derive(Clone, Copy)]
pub struct ConsoleProgressBarManager;
impl ProgressBarManager for ConsoleProgressBarManager {
    type Spinner = ConsoleSpinner;
    type Progress = ConsoleProgress;

    fn new_spinner_autotick(&self, message: &'static str) -> Self::Spinner {
        ConsoleSpinner::new_autotick(message)
    }

    fn new_progress_bar(&self, message: &'static str, total: u64) -> Self::Progress {
        ConsoleProgress::new(message, total)
    }
}

/// A [ProgressBarManager] that doesn't show any progress bars or spinners
#[derive(Clone, Copy)]
pub struct SilentProgressBarManager;
impl ProgressBarManager for SilentProgressBarManager {
    type Spinner = SilentSpinner;
    type Progress = SilentProgress;

    fn new_spinner_autotick(&self, _message: &'static str) -> Self::Spinner {
        SilentSpinner
    }

    fn new_progress_bar(&self, _message: &'static str, _total: u64) -> Self::Progress {
        SilentProgress
    }
}

/// A [Spinner] is a progress bar with an unknown duration / end point.
/// It doesn't know when it's going to be finished or how long it will take
/// and will just show a general spinning animation.
///
/// It can be cloned and the clones will all refer to the same spinner.
pub trait Spinner: Clone + Send + Sync {
    fn finish(self);
}

#[derive(Clone)]
pub struct ConsoleSpinner {
    // [indicatif::ProgressBar] is itself an [Arc] and can be cloned, but we still need our own
    // [Arc] so that our [Drop] behavior only drops after the last clone is dropped.
    pb: Arc<ConsoleProgressImpl>,
}

impl ConsoleSpinner {
    fn new_autotick(message: &'static str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_message(format!("{message}..."));
        pb.set_style(
            ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg}").unwrap(),
        );
        pb.enable_steady_tick(AUTOTICK_INTERVAL);
        Self {
            pb: Arc::new(ConsoleProgressImpl::new(message, pb)),
        }
    }
}

impl Spinner for ConsoleSpinner {
    /// Will panic if there are still other clones referencing the same spinner.
    fn finish(self) {
        let pb = Arc::into_inner(self.pb)
            .expect("Called `Spinner.finish` while other instances of the spinner still exist");
        std::mem::drop(pb);
    }
}

#[derive(Copy, Clone)]
pub struct SilentSpinner;
impl Spinner for SilentSpinner {
    fn finish(self) {}
}

/// A [Progress] is a progress bar with a clear end point and current state, i.e.
/// it always knows that x/total steps are already finished and can show a progress
/// bar including time estimates to the user.
///
/// It can be cloned and the clones will all refer to the same progress bar.
pub trait Progress: Clone + Send + Sync {
    fn inc(&self, delta: u64);
    fn inc_length(&self, delta: u64);

    /// Will panic if there are still other clones referencing the same progress bar.
    fn finish(self);
}

#[derive(Clone)]
pub struct ConsoleProgress {
    // [indicatif::ProgressBar] is itself an [Arc] and can be cloned, but we still need our own
    // [Arc] so that our [Drop] behavior only drops after the last clone is dropped.
    pb: Arc<ConsoleProgressImpl>,
}

impl ConsoleProgress {
    fn new(message: &'static str, total: u64) -> Self {
        let pb = ProgressBar::new(total);
        pb.set_message(format!("{message}..."));
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {msg} [{wide_bar:.cyan/blue}] {human_pos}/{human_len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
        );
        pb.tick();
        Self {
            pb: Arc::new(ConsoleProgressImpl::new(message, pb)),
        }
    }
}

#[derive(Copy, Clone)]
pub struct SilentProgress;
impl Progress for SilentProgress {
    fn inc(&self, _delta: u64) {}
    fn inc_length(&self, _delta: u64) {}
    fn finish(self) {}
}

impl Progress for ConsoleProgress {
    fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }

    fn inc_length(&self, delta: u64) {
        self.pb.inc_length(delta);
    }

    /// Will panic if there are still other clones referencing the same progress bar.
    fn finish(self) {
        let pb = Arc::into_inner(self.pb).expect(
            "Called `Progress.finish` while other instances of the progress bar still exist",
        );
        std::mem::drop(pb);
    }
}

struct ConsoleProgressImpl {
    message: &'static str,
    pb: ProgressBar,
}

impl ConsoleProgressImpl {
    pub fn new(message: &'static str, pb: ProgressBar) -> Self {
        Self { message, pb }
    }

    pub fn inc(&self, delta: u64) {
        self.pb.inc(delta);
    }

    pub fn inc_length(&self, delta: u64) {
        self.pb.inc_length(delta);
    }
}

impl Drop for ConsoleProgressImpl {
    fn drop(&mut self) {
        self.pb
            .finish_with_message(format!("{}...done\n", self.message));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silent_spinner_finish() {
        let spinner = SilentSpinner;
        // Should not panic
        spinner.finish();
    }

    #[test]
    fn test_silent_spinner_clone() {
        let spinner1 = SilentSpinner;
        let spinner2 = spinner1;
        // Both should work independently
        spinner1.finish();
        spinner2.finish();
    }

    #[test]
    fn test_silent_progress_inc() {
        let progress = SilentProgress;
        // Should not panic
        progress.inc(1);
        progress.inc(100);
    }

    #[test]
    fn test_silent_progress_inc_length() {
        let progress = SilentProgress;
        // Should not panic
        progress.inc_length(50);
    }

    #[test]
    fn test_silent_progress_finish() {
        let progress = SilentProgress;
        // Should not panic
        progress.finish();
    }

    #[test]
    fn test_silent_progress_clone() {
        let progress1 = SilentProgress;
        let progress2 = progress1;
        // Both should work independently
        progress1.inc(1);
        progress2.inc(2);
        progress1.finish();
        progress2.finish();
    }

    #[test]
    fn test_silent_progress_bar_manager_creates_silent_spinner() {
        let manager = SilentProgressBarManager;
        let spinner = manager.new_spinner_autotick("test");
        spinner.finish();
    }

    #[test]
    fn test_silent_progress_bar_manager_creates_silent_progress() {
        let manager = SilentProgressBarManager;
        let progress = manager.new_progress_bar("test", 100);
        progress.inc(50);
        progress.finish();
    }

    #[test]
    fn test_silent_progress_bar_manager_clone() {
        let manager1 = SilentProgressBarManager;
        let manager2 = manager1;
        // Both should create working spinners
        let spinner1 = manager1.new_spinner_autotick("test1");
        let spinner2 = manager2.new_spinner_autotick("test2");
        spinner1.finish();
        spinner2.finish();
    }
}
