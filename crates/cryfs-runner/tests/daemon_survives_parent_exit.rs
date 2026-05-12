//! Regression test for daemon detachment.
//!
//! Forks a sub-test-process that calls `start_background_process` (spawning a
//! daemon that loops writing a tick counter to a sentinel file), then exits
//! immediately. The main test process waits for the sub-process to reap, then
//! verifies that the daemon is in its own session (setsid took effect) and
//! is still alive and still updating the sentinel file. Cleans up via SIGTERM.
//!
//! Documents and locks in the setsid behavior we get from today's
//! `daemonize` crate, so the fork+exec refactor (which calls `setsid()`
//! explicitly in the daemon child) doesn't regress it.

use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use cryfs_runner::{RpcServer, start_background_process};
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, Pid, fork, getsid};
use tempdir::TempDir;

const SENTINEL_ENV: &str = "CRYFS_TEST_DAEMON_SENTINEL";
const PID_ENV: &str = "CRYFS_TEST_DAEMON_PID";

fn env_path(key: &str) -> PathBuf {
    std::env::var_os(key)
        .unwrap_or_else(|| panic!("env var {key} not set"))
        .into()
}

/// Loop forever, writing an incrementing tick counter to the sentinel file
/// every 50 ms. Ignores the RPC server entirely — this test is about the
/// daemon's *existence* surviving the parent's exit, not its RPC behavior.
fn daemon_main(_rpc: RpcServer<(), ()>) -> ! {
    let sentinel = env_path(SENTINEL_ENV);
    let pid_file = env_path(PID_ENV);

    std::fs::write(&pid_file, std::process::id().to_string()).expect("daemon: write pid");

    let mut tick: u64 = 0;
    loop {
        tick += 1;
        if let Err(err) = std::fs::write(&sentinel, tick.to_string()) {
            eprintln!("daemon: failed to write sentinel: {err}");
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// RAII handle that kills the daemon on drop, so an assertion failure in the
/// test doesn't leak the (init-parented, detached) daemon process. SIGTERM
/// first; SIGKILL after a 2 s grace period. Never panics from Drop.
struct DaemonGuard(Pid);

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = kill(self.0, Signal::SIGTERM);
        let term_deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match kill(self.0, None) {
                Ok(()) if Instant::now() >= term_deadline => {
                    eprintln!(
                        "daemon {} did not exit on SIGTERM within 2s; sending SIGKILL",
                        self.0,
                    );
                    let _ = kill(self.0, Signal::SIGKILL);
                    break;
                }
                Ok(()) => thread::sleep(Duration::from_millis(20)),
                // ESRCH: gone already. Anything else: stop probing — we're
                // in Drop and can't usefully react.
                Err(_) => break,
            }
        }
    }
}

#[test]
fn daemon_survives_parent_exit() {
    let tmp = TempDir::new("cryfs-daemon-survive-test").unwrap();
    let sentinel_path = tmp.path().join("sentinel");
    let pid_path = tmp.path().join("daemon.pid");

    // SAFETY: `set_var` is unsafe because it races with concurrent env
    // reads on other threads. This integration test is its own binary with
    // a single `#[test]`, so no sibling test thread is reading env at the
    // same time. The values are inherited across `fork()` into both the
    // sub-test-process and the daemon.
    unsafe {
        std::env::set_var(SENTINEL_ENV, &sentinel_path);
        std::env::set_var(PID_ENV, &pid_path);
    }

    match unsafe { fork() }.expect("fork failed") {
        ForkResult::Child => {
            // Simulate the cryfs parent CLI process: spawn the daemon, then
            // exit immediately. The daemon must keep running. `exit(0)`
            // skips destructors, matching what the real cryfs parent CLI
            // does after a successful mount.
            let _client = start_background_process::<(), ()>(daemon_main)
                .expect("start_background_process failed in child");
            std::process::exit(0);
        }
        ForkResult::Parent { child } => {
            let status = waitpid(child, None).expect("waitpid on sub-process");
            assert!(
                matches!(status, WaitStatus::Exited(_, 0)),
                "sub-test-process did not exit cleanly: {status:?}",
            );

            // Wait for the daemon to publish its PID. The daemon is a
            // grandchild of this test, not a direct child, so we discover it
            // via the PID file rather than waitpid.
            let pid_deadline = Instant::now() + Duration::from_secs(5);
            while !pid_path.exists() {
                assert!(
                    Instant::now() < pid_deadline,
                    "daemon did not write PID file within 5s",
                );
                thread::sleep(Duration::from_millis(20));
            }
            let daemon_pid = Pid::from_raw(
                std::fs::read_to_string(&pid_path)
                    .expect("read pid file")
                    .trim()
                    .parse::<i32>()
                    .expect("parse pid"),
            );
            // Installed *before* any assertion below, so the daemon gets
            // killed even if a check panics.
            let _guard = DaemonGuard(daemon_pid);

            // setsid moved the daemon into its own session. Without this,
            // the daemon would die on SIGHUP when its parent's controlling
            // terminal closes (e.g. when the user closes the shell).
            let daemon_sid = getsid(Some(daemon_pid)).expect("getsid(daemon)");
            let test_sid = getsid(None).expect("getsid(test)");
            assert_ne!(
                daemon_sid, test_sid,
                "daemon and test share a session — setsid did not take effect",
            );

            // Daemon should be updating the sentinel even though its parent
            // (the sub-test-process) has exited. Wait for the file to appear,
            // then poll until its contents change. Daemon writes every 50 ms,
            // so observing a change normally takes <100 ms; 5 s is a generous
            // ceiling that fails fast if the daemon has actually stopped.
            let sentinel_appear_deadline = Instant::now() + Duration::from_secs(5);
            while !sentinel_path.exists() {
                assert!(
                    Instant::now() < sentinel_appear_deadline,
                    "daemon did not create sentinel file within 5s",
                );
                thread::sleep(Duration::from_millis(20));
            }
            let first = std::fs::read_to_string(&sentinel_path).expect("read sentinel");
            let change_deadline = Instant::now() + Duration::from_secs(5);
            loop {
                thread::sleep(Duration::from_millis(20));
                let next = std::fs::read_to_string(&sentinel_path).expect("read sentinel");
                if next != first {
                    break; // observed a change → daemon is alive
                }
                assert!(
                    Instant::now() < change_deadline,
                    "daemon stopped writing sentinel after parent exited (no change in 5s)",
                );
            }

            // Cleanup happens via DaemonGuard's Drop.
        }
    }
}
