use std::os::fd::{FromRawFd, OwnedFd, RawFd};
use std::time::Duration;

use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

use super::pipe::{Receiver, Sender, pipe};

pub struct RpcConnection<Request, Response>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned,
{
    request_sender: Sender<Request>,
    request_receiver: Receiver<Request>,
    response_sender: Sender<Response>,
    response_receiver: Receiver<Response>,
}

impl<Request, Response> RpcConnection<Request, Response>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    pub fn new_pipe() -> Result<Self> {
        let (request_sender, request_receiver) = pipe::<Request>()?;
        let (response_sender, response_receiver) = pipe::<Response>()?;
        Ok(Self {
            request_sender,
            request_receiver,
            response_sender,
            response_receiver,
        })
    }

    pub fn into_server(self) -> RpcServer<Request, Response> {
        RpcServer {
            sender: self.response_sender,
            receiver: self.request_receiver,
        }
    }

    pub fn into_client(self) -> RpcClient<Request, Response> {
        RpcClient {
            sender: self.request_sender,
            receiver: self.response_receiver,
        }
    }

    /// Split for fork+exec: keep the parent-side `RpcClient` and surrender
    /// the two child-side raw file descriptors. The caller is expected to
    /// `dup2` the returned fds onto `CHILD_REQUEST_RECV_FD` and
    /// `CHILD_RESPONSE_SEND_FD` (3 and 4) in a `pre_exec` closure, then drop
    /// the originals after `Command::spawn` returns.
    pub fn into_client_and_child_fds(self) -> (RpcClient<Request, Response>, OwnedFd, OwnedFd) {
        let client = RpcClient {
            sender: self.request_sender,
            receiver: self.response_receiver,
        };
        let child_request_recv = self.request_receiver.into_owned_fd();
        let child_response_send = self.response_sender.into_owned_fd();
        (client, child_request_recv, child_response_send)
    }

    #[cfg(test)]
    pub fn into_server_and_client(
        self,
    ) -> (RpcServer<Request, Response>, RpcClient<Request, Response>) {
        (
            RpcServer {
                sender: self.response_sender,
                receiver: self.request_receiver,
            },
            RpcClient {
                sender: self.request_sender,
                receiver: self.response_receiver,
            },
        )
    }
}

pub struct RpcServer<Request, Response>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned,
{
    sender: Sender<Response>,
    receiver: Receiver<Request>,
}

impl<Request, Response> RpcServer<Request, Response>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned,
{
    /// Reconstruct an `RpcServer` from inherited raw file descriptors. The
    /// fork+exec daemon child receives its pipe ends as fds 3 (request-recv)
    /// and 4 (response-send) and calls this to rebuild its typed RPC handle.
    ///
    /// # Safety
    /// `in_fd` must be the read end of a pipe whose write end is held by the
    /// parent's `RpcClient`. `out_fd` must be the corresponding write end.
    /// Both fds must be owned (not shared) — calling this twice on the same
    /// fd numbers is a use-after-free.
    pub unsafe fn from_raw_fds(in_fd: RawFd, out_fd: RawFd) -> Self {
        let receiver = unsafe { Receiver::from_owned_fd(OwnedFd::from_raw_fd(in_fd)) };
        let sender = unsafe { Sender::from_owned_fd(OwnedFd::from_raw_fd(out_fd)) };
        Self { sender, receiver }
    }

    pub fn next_request(&mut self) -> Result<Request> {
        self.receiver.recv()
    }

    pub fn send_response(&mut self, response: &Response) -> Result<()> {
        self.sender.send(response)
    }
}

pub struct RpcClient<Request, Response>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    sender: Sender<Request>,
    receiver: Receiver<Response>,
}

impl<Request, Response> RpcClient<Request, Response>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    pub fn send_request(&mut self, request: &Request) -> Result<()> {
        self.sender.send(request)
    }

    pub fn recv_response(&mut self, timeout: Duration) -> Result<Response> {
        self.receiver.recv_timeout(timeout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[test]
    fn rpc() {
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        struct Request {
            v: u32,
        }
        #[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
        struct Response {
            v: u32,
        }

        let connection = RpcConnection::<Request, Response>::new_pipe().unwrap();
        let (mut server, mut client) = connection.into_server_and_client();

        client.send_request(&Request { v: 42 }).unwrap();
        assert_eq!(Request { v: 42 }, server.next_request().unwrap());

        server.send_response(&Response { v: 10 }).unwrap();
        assert_eq!(
            Response { v: 10 },
            client.recv_response(Duration::from_secs(2)).unwrap()
        );
    }
}
