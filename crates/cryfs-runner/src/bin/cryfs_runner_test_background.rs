//! Helper binary used by `start_background_process_with_exe` integration
//! tests. Reads `CRYFS_TEST_BEHAVIOR` from the environment and replays one of
//! a few canned daemon behaviors against the inherited fds 3 and 4.
//!
//! This binary is what the test_child_* daemon-lifecycle tests now spawn
//! instead of forking an in-process fn pointer, so they no longer suffer the
//! parallel-test fd-inheritance flake.

use cryfs_runner::{RpcServer, rpc_server_from_inherited_fds};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Request {
    request: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Response {
    response: i32,
}

fn main() {
    let behavior = std::env::var("CRYFS_TEST_BEHAVIOR").unwrap_or_else(|_| "echo".to_string());

    let mut rpc: RpcServer<Request, Response> = rpc_server_from_inherited_fds()
        .expect("daemon: failed to rebuild RpcServer from inherited fds");

    match behavior.as_str() {
        "echo" => loop {
            let request = match rpc.next_request() {
                Ok(r) => r,
                Err(err) => {
                    // Parent dropped the client → EOF → clean exit. Any
                    // other error is a real daemon-side failure; surface
                    // it on stderr so a hung/failing parent test isn't
                    // the only diagnostic.
                    let is_eof = err
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|e| e.kind() == std::io::ErrorKind::UnexpectedEof);
                    if !is_eof {
                        eprintln!("daemon: echo receive failed: {err:#}");
                        std::process::exit(1);
                    }
                    std::process::exit(0);
                }
            };
            rpc.send_response(&Response {
                response: request.request + 1,
            })
            .expect("daemon: failed to send response");
        },
        "panic_after_request" => {
            let _ = rpc.next_request().expect("daemon: expected a request");
            panic!("daemon: panic_after_request");
        }
        "panic_before_request" => {
            panic!("daemon: panic_before_request");
        }
        "exit_after_request" => {
            let _ = rpc.next_request().expect("daemon: expected a request");
            std::process::exit(0);
        }
        "exit_before_request" => {
            std::process::exit(0);
        }
        "write_to_fd_then_idle" => {
            // Used by spawn_fd_isolation. Attempts to write a sentinel byte
            // to the file descriptor number in CRYFS_TEST_LEAK_FD — under
            // fork+exec + FD_CLOEXEC, this fd should already be closed (no
            // longer inherited), so the write fails with EBADF. The test
            // verifies its parent-side read end gets EOF rather than the
            // sentinel byte. Then writes a PID file and idles so the test
            // can clean up.
            drop(rpc);
            let leak_fd: i32 = std::env::var("CRYFS_TEST_LEAK_FD")
                .expect("CRYFS_TEST_LEAK_FD not set")
                .parse()
                .expect("CRYFS_TEST_LEAK_FD not an int");
            let pid_file = std::path::PathBuf::from(
                std::env::var_os("CRYFS_TEST_PID").expect("CRYFS_TEST_PID not set"),
            );
            // The write is expected to fail with EBADF when the test
            // succeeds. Do it before the PID file write so the parent
            // doesn't observe "pid file present" until we've at least
            // attempted the leak.
            let payload = b"LEAK\n";
            let _ = unsafe { libc::write(leak_fd, payload.as_ptr().cast(), payload.len()) };
            std::fs::write(&pid_file, std::process::id().to_string())
                .expect("daemon: write pid file");
            if unsafe { libc::setsid() } < 0 {
                eprintln!("daemon: setsid failed: {}", std::io::Error::last_os_error());
                std::process::exit(1);
            }
            loop {
                std::thread::sleep(std::time::Duration::from_secs(60));
            }
        }
        "sentinel_loop" => {
            // Used by daemon_survives_parent_exit. Ignore RPC entirely;
            // loop writing the current monotonic timestamp to the path in
            // CRYFS_TEST_SENTINEL, plus the daemon's PID to CRYFS_TEST_PID.
            // The test verifies the file is still being updated after the
            // sub-test parent exits.
            drop(rpc);
            let sentinel = std::path::PathBuf::from(
                std::env::var_os("CRYFS_TEST_SENTINEL").expect("CRYFS_TEST_SENTINEL not set"),
            );
            let pid_file = std::path::PathBuf::from(
                std::env::var_os("CRYFS_TEST_PID").expect("CRYFS_TEST_PID not set"),
            );
            std::fs::write(&pid_file, std::process::id().to_string())
                .expect("daemon: write pid file");
            // setsid for the test daemon so it survives the sub-test-process
            // exit even though we haven't gone through cryfs's
            // run_as_background_daemon (which would have called setsid).
            if unsafe { libc::setsid() } < 0 {
                eprintln!("daemon: setsid failed: {}", std::io::Error::last_os_error());
                std::process::exit(1);
            }
            let mut tick: u64 = 0;
            loop {
                tick += 1;
                if let Err(err) = std::fs::write(&sentinel, tick.to_string()) {
                    eprintln!("daemon: failed to write sentinel: {err}");
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
        other => {
            panic!("daemon: unknown CRYFS_TEST_BEHAVIOR={other:?}");
        }
    }
}
