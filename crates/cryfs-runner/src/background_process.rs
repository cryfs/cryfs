use std::time::Duration;

use anyhow::anyhow;
use anyhow::Result;
use cryfs_cli_utils::{CliError, CliErrorKind, CliResultExt};
use serde::{Deserialize, Serialize};

use crate::{
    ipc::{start_background_process, RpcClient, RpcServer},
    MountArgs,
};

pub struct BackgroundProcess {
    rpc: RpcClient<Request, Response>,
}

impl BackgroundProcess {
    pub fn daemonize() -> Result<Self> {
        let rpc = start_background_process(background_main)?;
        let mut mount_process = Self { rpc };
        mount_process.status_check()?;
        Ok(mount_process)
    }

    fn status_check(&mut self) -> Result<()> {
        self.rpc.send_request(&Request::StatusCheckRequest)?;
        let response: Response = self.rpc.recv_response(Duration::from_secs(2))?;
        match response {
            Response::StatusCheckResponse => Ok(()),
            response => panic!("Unexpected response: {response:?}"),
        }
    }

    pub fn mount_filesystem(&mut self, mount_args: MountArgs) -> Result<(), CliError> {
        self.rpc
            .send_request(&Request::MountRequest(mount_args))
            .map_cli_error(CliErrorKind::UnspecifiedError)?;
        let response: Response = self
            .rpc
            .recv_response(Duration::from_secs(10))
            .map_cli_error(CliErrorKind::UnspecifiedError)?;
        match response {
            Response::MountResponse(Ok(())) => Ok(()),
            Response::MountResponse(Err(err)) => Err(CliError {
                kind: err.cli_error_kind,
                // TODO Should we report more error info than just the message to the parent process?
                error: anyhow!("{}", err.message),
            }),
            response => panic!("Unexpected response: {response:?}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MountError {
    pub cli_error_kind: CliErrorKind,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    StatusCheckRequest,
    MountRequest(MountArgs),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    StatusCheckResponse,
    MountResponse(Result<(), MountError>),
}

pub fn background_main(mut rpc_server: RpcServer<Request, Response>) -> ! {
    while let Ok(request) = rpc_server.next_request() {
        match request {
            Request::StatusCheckRequest => {
                rpc_server
                    .send_response(&Response::StatusCheckResponse)
                    .expect("Failed to send response. Maybe the parent process exited.");
            }
            Request::MountRequest(mount_args) => {
                let on_successfully_mounted = || {
                    rpc_server
                        .send_response(&Response::MountResponse(Ok(())))
                        .expect("Failed to send response. Maybe the parent process exited.");
                    close_stdout_stderr();
                };
                let mount_result =
                    super::runner::mount_filesystem(mount_args, on_successfully_mounted);
                match mount_result {
                    Ok(()) => {
                        // `mount_filesystem` only returns with `Ok` if the filesystem was correctly mounted **and then later unmounted**.
                        // It blocks until the unmount. No need to send a response here because it wa already sent in `on_successfully_mounted` above.
                    }
                    Err(err) => {
                        let mount_error = MountError {
                            cli_error_kind: err.kind,
                            // TODO Should we report more error info than just the message to the parent process?
                            message: err.to_string(),
                        };
                        rpc_server
                            .send_response(&Response::MountResponse(Err(mount_error)))
                            .expect("Failed to send response. Maybe the parent process exited.");
                    }
                }
            }
        }
    }

    // TODO Should we make this into a panic and introduce a clean shutdown where Client Drop drops the Server?
    // Error getting request, parent process probably exited or closed the pipe
    std::process::exit(0);
}

fn close_stdout_stderr() {
    // TODO We should probably redirect them to the logfile if there is a logfile argument, otherwise /dev/null
    // See https://docs.rs/daemonize/latest/src/daemonize/lib.rs.html#454
}
