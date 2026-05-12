use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use anyhow::anyhow;
use cryfs_cli_utils::{CliError, CliErrorKind, CliResultExt};
use serde::{Deserialize, Serialize};

use crate::{
    MountArgs,
    ipc::{RpcClient, RpcServer, start_background_process},
};

pub struct BackgroundProcess {
    rpc: RpcClient<Request, Response>,
}

impl BackgroundProcess {
    pub fn daemonize() -> Result<Self> {
        let rpc = start_background_process::<Request, Response>()?;
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
                error: Arc::new(anyhow!("{}", err.message)),
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

pub fn background_main(rpc_server: RpcServer<Request, Response>) -> ! {
    // Now we're post-daemonization, so we can initialize tokio.
    let runtime = crate::init_tokio();
    runtime.block_on(background_async_main(rpc_server))
}

async fn background_async_main(mut rpc_server: RpcServer<Request, Response>) -> ! {
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
                    super::runner::mount_filesystem(mount_args, on_successfully_mounted).await;
                match mount_result {
                    Ok(()) => {
                        // `mount_filesystem` only returns with `Ok` if the filesystem was correctly mounted **and then later unmounted**.
                        // It blocks until the unmount. No need to send a response here because it wa already sent in `on_successfully_mounted` above.
                    }
                    Err(err) => {
                        let mount_error = MountError {
                            cli_error_kind: err.kind,
                            message: format!("{:?}", err.error),
                        };
                        // Ignore errors because the parent process likely has exited if the file system was already mounted for some time
                        let _ =
                            rpc_server.send_response(&Response::MountResponse(Err(mount_error)));
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
    // Redirect stdin/stdout/stderr at /dev/null. We don't bare-close fds 0/1/2
    // because a later allocation could re-grab those numbers and produce
    // garbage in unrelated files. dup2-over-`/dev/null` keeps the fd numbers
    // valid but neutralized.
    let devnull = match std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/null")
    {
        Ok(f) => f,
        Err(err) => {
            log::warn!("failed to open /dev/null while detaching daemon stdio: {err}");
            return;
        }
    };
    let fd = std::os::fd::AsRawFd::as_raw_fd(&devnull);
    for target in [libc::STDIN_FILENO, libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        if unsafe { libc::dup2(fd, target) } < 0 {
            log::warn!(
                "dup2(/dev/null, {target}) failed while detaching daemon stdio: {}",
                std::io::Error::last_os_error(),
            );
        }
    }
    // `devnull` drop closes the temp fd; targets 0/1/2 keep their dup'd copies.
}
