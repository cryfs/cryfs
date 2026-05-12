//! End-to-end regression test for the daemon spawn + RPC roundtrip.
//!
//! Spawns the `cryfs-runner-test-background` helper binary as a daemon via
//! fork+exec, exchanges request/response pairs with varying payload sizes,
//! drops the client, and verifies the daemon exits cleanly (i.e. its receive
//! loop terminates on EOF rather than hanging).
//!
//! The helper binary's `echo` behavior expects `i32`-payload Request/Response,
//! so this test uses those rather than free-form byte payloads. Variation
//! comes from the request values themselves; sizes are exercised separately
//! by the existing in-process `ipc::pipe::tests::recv_timeout::large_message`
//! and `multiple_sequential_messages` unit tests.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::time::Duration;

use cryfs_runner::start_background_process_with_exe;
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

#[test]
fn roundtrip_many_requests() {
    let env: [(&OsStr, &OsStr); 1] = [(OsStr::new("CRYFS_TEST_BEHAVIOR"), OsStr::new("echo"))];
    let mut client =
        start_background_process_with_exe::<Request, Response>(&helper_exe(), &env).unwrap();

    for i in 0..10i32 {
        client
            .send_request(&Request { request: i })
            .expect("client failed to send request");
        let response = client
            .recv_response(Duration::from_secs(5))
            .expect("client failed to receive response");
        assert_eq!(response, Response { response: i + 1 });
    }

    // Dropping the client closes its end of both pipes; the daemon's
    // `next_request` should observe EOF and the daemon should `exit(0)`.
    drop(client);
}
