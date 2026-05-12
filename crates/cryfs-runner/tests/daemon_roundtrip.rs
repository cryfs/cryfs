//! End-to-end regression test for the daemon spawn + RPC roundtrip.
//!
//! Spawns a daemon via `start_background_process`, exchanges a series of
//! request/response pairs with varying payload sizes, drops the client, and
//! verifies the daemon exits cleanly (i.e. its receive loop terminates on EOF
//! rather than hanging).
//!
//! Today this exercises the fork-only `daemonize`-crate path. After the
//! fork+exec refactor it will be adjusted to spawn the daemon via a helper
//! binary instead of an injected fn-pointer.

use std::time::Duration;

use cryfs_runner::{RpcServer, start_background_process};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Request {
    payload: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Response {
    echoed: Vec<u8>,
    len: u64,
}

fn echo_daemon(mut rpc: RpcServer<Request, Response>) -> ! {
    loop {
        match rpc.next_request() {
            Ok(request) => {
                let len = request.payload.len() as u64;
                rpc.send_response(&Response {
                    echoed: request.payload,
                    len,
                })
                .expect("daemon failed to send response");
            }
            Err(err) => {
                // Parent dropped the client → EOF → clean exit. Any other error
                // is a real daemon-side failure; surface it on stderr so the
                // parent's recv_response timeout isn't the only diagnostic.
                let is_eof = err
                    .downcast_ref::<std::io::Error>()
                    .is_some_and(|e| e.kind() == std::io::ErrorKind::UnexpectedEof);
                if !is_eof {
                    eprintln!("echo daemon: receive failed: {err:#}");
                    std::process::exit(1);
                }
                std::process::exit(0);
            }
        }
    }
}

#[test]
fn roundtrip_varying_payload_sizes() {
    let mut client = start_background_process::<Request, Response>(echo_daemon).unwrap();

    // Mix of sizes: empty, small, several KB, near the 1 MiB pipe limit.
    let sizes = [0, 1, 16, 1024, 8 * 1024, 64 * 1024, 256 * 1024, 900 * 1024];

    for (i, &size) in sizes.iter().enumerate() {
        let payload: Vec<u8> = (0..size).map(|n| ((n + i) % 256) as u8).collect();
        let expected = payload.clone();

        client
            .send_request(&Request { payload })
            .expect("client failed to send request");
        let response = client
            .recv_response(Duration::from_secs(5))
            .expect("client failed to receive response");

        assert_eq!(response.len, size as u64, "wrong length for size {size}");
        assert_eq!(response.echoed, expected, "wrong payload for size {size}");
    }

    // Dropping the client closes its end of both pipes; the daemon's
    // `next_request` should observe EOF and the daemon should `exit(0)`.
    drop(client);
}
