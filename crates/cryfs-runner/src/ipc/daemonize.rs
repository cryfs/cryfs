use anyhow::Result;
use daemonize::{Daemonize, Stdio};
use serde::{Serialize, de::DeserializeOwned};
use tokio::runtime::Handle;

use crate::ipc::RpcConnection;

use super::{RpcClient, RpcServer};

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
    fn test_child_panicking() {
        fn background_main(_rpc: RpcServer<Request, Response>) -> ! {
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
    fn test_child_exiting() {
        fn background_main(_rpc: RpcServer<Request, Response>) -> ! {
            std::process::exit(0);
        }
        let mut rpc = start_background_process(background_main).unwrap();
        rpc.send_request(&Request { request: 42 }).unwrap();
        let response = rpc
            .recv_response(std::time::Duration::from_secs(2))
            .unwrap_err();
        assert_eq!("Sender closed the pipe", response.to_string());
    }
}
