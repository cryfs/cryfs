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
