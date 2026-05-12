//! Integration version of the daemonize `test_child_*` lifecycle tests.
//!
//! These previously lived as unit tests in `ipc/daemonize.rs` and forked an
//! in-process fn pointer. That spawn shape suffered an ~5% flake rate in
//! parallel `cargo test` runs: sibling tests' pipe fds were inherited into
//! each others' daemonized children, preventing EOF/EPIPE delivery on the
//! rightful pipe owners.
//!
//! Reworked here to spawn a dedicated helper binary via fork+exec, so:
//! - The daemon child is a clean single-threaded process image (no inherited
//!   libtest threads or mutexes).
//! - Inherited fds are limited to the two we explicitly `dup2` onto fds 3 and
//!   4 — everything else dies under `FD_CLOEXEC` during `execve`.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;

use cryfs_runner::{RpcClient, start_background_process_with_exe};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Request {
    request: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Response {
    response: i32,
}

fn helper_exe() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_cryfs-runner-test-background"))
}

fn spawn_daemon(behavior: &str) -> RpcClient<Request, Response> {
    let env: [(&OsStr, &OsStr); 1] = [(OsStr::new("CRYFS_TEST_BEHAVIOR"), OsStr::new(behavior))];
    start_background_process_with_exe(&helper_exe(), &env)
        .expect("start_background_process_with_exe failed")
}

#[test]
fn test_child_echo_roundtrip() {
    let mut rpc = spawn_daemon("echo");
    rpc.send_request(&Request { request: 42 }).unwrap();
    let response = rpc.recv_response(Duration::from_secs(2)).unwrap();
    assert_eq!(response, Response { response: 43 });
}

#[test]
fn test_child_panicking_after_request() {
    let mut rpc = spawn_daemon("panic_after_request");
    rpc.send_request(&Request { request: 42 }).unwrap();
    let response = rpc.recv_response(Duration::from_secs(2)).unwrap_err();
    assert_eq!("Sender closed the pipe", response.to_string());
}

#[test]
fn test_child_panicking_before_request() {
    let mut rpc = spawn_daemon("panic_before_request");
    let response = rpc.recv_response(Duration::from_secs(2)).unwrap_err();
    assert_eq!("Sender closed the pipe", response.to_string());
}

#[test]
fn test_child_exiting_after_request() {
    let mut rpc = spawn_daemon("exit_after_request");
    rpc.send_request(&Request { request: 42 }).unwrap();
    let response = rpc.recv_response(Duration::from_secs(2)).unwrap_err();
    assert_eq!("Sender closed the pipe", response.to_string());
}

#[test]
fn test_child_exiting_before_request() {
    let mut rpc = spawn_daemon("exit_before_request");
    let response = rpc.recv_response(Duration::from_secs(2)).unwrap_err();
    assert_eq!("Sender closed the pipe", response.to_string());
}
