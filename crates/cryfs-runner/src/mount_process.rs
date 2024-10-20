use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::ipc::{start_background_process, RpcClient, RpcServer};

pub struct MountProcess {
    rpc: RpcClient<Request, Response>,
}

impl MountProcess {
    pub fn daemonize() -> Result<Self> {
        println!("Mounting in background mode. CryFS will continue to run in the background.");
        let rpc = start_background_process(background_main)?;
        let mut mount_process = Self { rpc };
        mount_process.status_check()?;
        Ok(mount_process)
    }

    fn status_check(&mut self) -> Result<()> {
        self.rpc.send_request(&Request::StatusCheck)?;
        let response: Response = self.rpc.recv_response(Duration::from_secs(2))?;
        println!("Received {response:?}");
        match response {
            Response::IsRunning => Ok(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    StatusCheck,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    IsRunning,
}

pub fn background_main(mut rpc_server: RpcServer<Request, Response>) -> ! {
    while let Ok(request) = rpc_server.next_request() {
        match request {
            Request::StatusCheck => {
                rpc_server
                    .send_response(&Response::IsRunning)
                    .expect("Failed to send response. Maybe the parent process exited.");
            }
        }
    }
    // TODO Should we make this into a panic and introduce a clean shutdown where Client Drop drops the Server?
    // Error getting request, parent process probably exited or closed the pipe
    std::process::exit(0);
}
