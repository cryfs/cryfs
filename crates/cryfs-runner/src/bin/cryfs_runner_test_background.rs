//! Helper binary used by `start_background_process_with_exe` integration
//! tests. Reads `CRYFS_TEST_BEHAVIOR` from the environment and replays one of
//! a few canned daemon behaviors against the inherited fds 3 and 4.
//!
//! This binary is what the test_child_* daemon-lifecycle tests now spawn
//! instead of forking an in-process fn pointer, so they no longer suffer the
//! parallel-test fd-inheritance flake.

use cryfs_runner::{rpc_server_from_inherited_fds, RpcServer};
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
    let behavior =
        std::env::var("CRYFS_TEST_BEHAVIOR").unwrap_or_else(|_| "echo".to_string());

    let mut rpc: RpcServer<Request, Response> = rpc_server_from_inherited_fds()
        .expect("daemon: failed to rebuild RpcServer from inherited fds");

    match behavior.as_str() {
        "echo" => loop {
            let request = match rpc.next_request() {
                Ok(r) => r,
                Err(_) => std::process::exit(0),
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
        other => {
            panic!("daemon: unknown CRYFS_TEST_BEHAVIOR={other:?}");
        }
    }
}
