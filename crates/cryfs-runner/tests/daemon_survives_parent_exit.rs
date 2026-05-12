//! Regression test for daemon detachment.
//!
//! Forks a sub-test-process that calls `start_background_process_with_exe`,
//! spawning the `cryfs-runner-test-background` helper binary in
//! `sentinel_loop` mode (ignores RPC, writes a tick counter to a file
//! forever), then exits immediately. The main test waits for the sub-process
//! to reap, verifies the daemon is in its own session (setsid took effect),
//! and that it's still updating the sentinel. Cleans up via SIGTERM.
//!
//! Locks in the setsid behavior of the fork+exec spawn: without `setsid()`
//! in the daemon, it would die along with the parent's shell session.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};

use cryfs_runner::start_background_process_with_exe;
use nix::sys::signal::{Signal, kill};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{ForkResult, Pid, fork, getsid};

fn helper_exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cryfs-runner-test-background"))
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
    let tmp = tempfile::Builder::new()
        .prefix("cryfs-daemon-survive-test")
        .tempdir()
        .unwrap();
    let sentinel_path = tmp.path().join("sentinel");
    let pid_path = tmp.path().join("daemon.pid");

    // SAFETY: `set_var` is unsafe because it races with concurrent env
    // reads on other threads. This integration test is its own binary with
    // a single `#[test]`, so no sibling test thread is reading env at the
    // same time. The values are inherited through fork + execve into the
    // helper daemon.
    unsafe {
        std::env::set_var("CRYFS_TEST_SENTINEL", &sentinel_path);
        std::env::set_var("CRYFS_TEST_PID", &pid_path);
    }

    match unsafe { fork() }.expect("fork failed") {
        ForkResult::Child => {
            // Simulate the cryfs parent CLI process: spawn the daemon, then
            // exit immediately. The daemon must keep running. `exit(0)`
            // skips destructors, matching what the real cryfs parent CLI
            // does after a successful mount.
            let env: [(&OsStr, &OsStr); 1] = [(
                OsStr::new("CRYFS_TEST_BEHAVIOR"),
                OsStr::new("sentinel_loop"),
            )];
            let _client = start_background_process_with_exe::<(), ()>(&helper_exe(), &env)
                .expect("start_background_process_with_exe failed in child");
            std::process::exit(0);
        }
        ForkResult::Parent { child } => {
            let status = waitpid(child, None).expect("waitpid on sub-process");
            assert!(
                matches!(status, WaitStatus::Exited(_, 0)),
                "sub-test-process did not exit cleanly: {status:?}",
            );

            // Daemon is a grandchild of this test; discover its PID through
            // the file it writes on startup.
            //
            // Poll on parseable content rather than just file existence:
            // `std::fs::write` (used by the daemon) creates the file before
            // it writes, so a naive `pid_path.exists()` check can win the
            // race and read an empty file. macOS exposes this race more
            // often than Linux.
            let pid_deadline = Instant::now() + Duration::from_secs(5);
            let daemon_pid = loop {
                if let Ok(contents) = std::fs::read_to_string(&pid_path) {
                    if let Ok(pid) = contents.trim().parse::<i32>() {
                        break Pid::from_raw(pid);
                    }
                }
                assert!(
                    Instant::now() < pid_deadline,
                    "daemon did not publish a parseable PID within 5s",
                );
                thread::sleep(Duration::from_millis(20));
            };
            // Installed *before* any assertion below, so the daemon gets
            // killed even if a check panics.
            let _guard = DaemonGuard(daemon_pid);

            // setsid moved the daemon into its own session. Without this,
            // the daemon would die on SIGHUP when the parent's controlling
            // terminal closes (e.g. when the user closes the shell).
            let daemon_sid = getsid(Some(daemon_pid)).expect("getsid(daemon)");
            let test_sid = getsid(None).expect("getsid(test)");
            assert_ne!(
                daemon_sid, test_sid,
                "daemon and test share a session — setsid did not take effect",
            );

            // Daemon must keep writing the sentinel even though its parent
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
