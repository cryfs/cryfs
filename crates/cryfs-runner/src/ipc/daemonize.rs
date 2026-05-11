use anyhow::Result;
use daemonize::{Daemonize, Stdio};
use serde::{Serialize, de::DeserializeOwned};
use tokio::runtime::Handle;

use crate::ipc::RpcConnection;

use super::{RpcClient, RpcServer};

// TODO Refactor to fork+exec instead of plain fork.
//
// The current implementation uses the `daemonize` crate, which does `fork()` without
// `exec()`. Two problems follow from this:
//
// 1. Inherited fds aren't closed. The `daemonize` crate only dup2s stdio; it leaves
//    fds >= 3 alone. `interprocess` creates pipes without `O_CLOEXEC`. The daemonized
//    child therefore holds copies of every fd open in the parent at fork time. In
//    production this leaks shell-inherited fds into the daemon. In parallel `cargo
//    test` runs it causes flaky failures (~5% rate) because sibling tests' pipes get
//    inherited into each others' daemonized children, preventing EOF/EPIPE delivery to
//    the rightful pipe owner — observed in `dropped_recver` and the
//    `test_child_*_{before,after}_request` daemonize tests.
//
// 2. Fork-after-multithread hazard. POSIX restricts post-fork code in a multithreaded
//    program to async-signal-safe operations only, because any mutex held by another
//    thread at fork time stays locked forever in the child. The cargo test harness is
//    multithreaded, so `background_main` (which calls `init_tokio()` and allocates
//    heavily) can rarely deadlock the child. Production cryfs is fine here because it
//    daemonizes single-threaded, before tokio.
//
// Switching to fork+exec fixes both:
//   - `execve()` closes every `O_CLOEXEC` fd in the kernel. Rust stdlib sets CLOEXEC
//     by default; we'd additionally set CLOEXEC on the `interprocess` pipes and clear
//     it only on the two rpc fds we explicitly pass via argv to the re-exec'd child.
//   - The new process is a fresh image: single-threaded, fresh allocator, no
//     inherited mutex state.
//
// Sketch: `Command::new(env::current_exe()).arg("--background-child")
// .arg(format!("{fd_in}:{fd_out}")).spawn()`. `cryfs-cli`'s `main` would detect the
// flag and call `background_main` with the passed fds. See systemd `daemon(7)` and
// rust-lang/rust#24034 for context.
pub fn start_background_process<Request, Response>(
    // TODO Once the `!` type is stabilized, we can use `FnOnce` instead of `fn` here.
    background_main: fn(RpcServer<Request, Response>) -> !,
) -> Result<RpcClient<Request, Response>>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    if Handle::try_current().is_ok() {
        panic!(
            "Cannot daemonize a process if tokio is running. Please daemonize before initializing tokio. See https://github.com/tokio-rs/tokio/issues/4301"
        );
    }

    let rpc_pipes = RpcConnection::new_pipe()?;

    // get current umask value because `daemonize` force overwrites it but we don't really want it to change, so we give it the old value
    let umask = unsafe { libc::umask(0) };
    #[cfg(target_os = "macos")]
    let umask = u32::from(umask);
    match Daemonize::new()
        .umask(umask)
        // We're keeping stdout and stderr bound to the parent at first, but will close them in the child after mounting was successful
        .stdout(Stdio::keep())
        .stderr(Stdio::keep())
        .execute()
    {
        daemonize::Outcome::Parent(parent) => {
            parent?;

            Ok(rpc_pipes.into_client())
        }
        daemonize::Outcome::Child(child) => {
            child.expect("Daemonization failed in child");

            let pipe = rpc_pipes.into_server();
            background_main(pipe);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Request {
        request: i32,
    }
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Response {
        response: i32,
    }

    #[test]
    fn test_start_background_process() {
        fn background_main(mut rpc: RpcServer<Request, Response>) -> ! {
            loop {
                let request = rpc.next_request().unwrap();
                rpc.send_response(&Response {
                    response: request.request + 1,
                })
                .unwrap();
            }
        }
        let mut rpc = start_background_process(background_main).unwrap();
        rpc.send_request(&Request { request: 42 }).unwrap();
        let response = rpc
            .recv_response(std::time::Duration::from_secs(2))
            .unwrap();
        assert_eq!(response, Response { response: 43 });
    }

    #[test]
    fn test_child_panicking_after_request() {
        fn background_main(mut rpc: RpcServer<Request, Response>) -> ! {
            let _request = rpc.next_request().unwrap();
            panic!("Child is panicking");
        }
        let mut rpc = start_background_process(background_main).unwrap();
        rpc.send_request(&Request { request: 42 }).unwrap();
        let response = rpc
            .recv_response(std::time::Duration::from_secs(2))
            .unwrap_err();
        assert_eq!("Sender closed the pipe", response.to_string());
    }

    #[test]
    fn test_child_panicking_before_request() {
        fn background_main(_rpc: RpcServer<Request, Response>) -> ! {
            panic!("Child is panicking");
        }
        let mut rpc = start_background_process(background_main).unwrap();
        let response = rpc
            .recv_response(std::time::Duration::from_secs(2))
            .unwrap_err();
        assert_eq!("Sender closed the pipe", response.to_string());
    }

    #[test]
    fn test_child_exiting_after_request() {
        fn background_main(mut rpc: RpcServer<Request, Response>) -> ! {
            let _request = rpc.next_request().unwrap();
            std::process::exit(0);
        }
        let mut rpc = start_background_process(background_main).unwrap();
        rpc.send_request(&Request { request: 42 }).unwrap();
        let response = rpc
            .recv_response(std::time::Duration::from_secs(2))
            .unwrap_err();
        assert_eq!("Sender closed the pipe", response.to_string());
    }

    #[test]
    fn test_child_exiting_before_request() {
        fn background_main(_rpc: RpcServer<Request, Response>) -> ! {
            std::process::exit(0);
        }
        let mut rpc = start_background_process(background_main).unwrap();
        let response = rpc
            .recv_response(std::time::Duration::from_secs(2))
            .unwrap_err();
        assert_eq!("Sender closed the pipe", response.to_string());
    }
}
