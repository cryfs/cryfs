use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, Result, anyhow, bail};
use clap_logflag::LoggingConfig;
use cryfs_cli_utils::{CliError, CliErrorKind, CliResultExt};
use serde::{Deserialize, Serialize};

use crate::{
    MountArgs,
    ipc::{RpcClient, RpcServer, start_background_process},
};

/// How long the parent will wait for the daemon to ack the Bootstrap request.
/// The daemon's ack handler just calls `init_logging` and writes the ack —
/// sub-millisecond on any healthy system. Generous bound so a slow CI doesn't
/// flake.
const BOOTSTRAP_TIMEOUT: Duration = Duration::from_secs(10);

pub struct BackgroundProcess {
    rpc: RpcClient<Request, Response>,
}

impl BackgroundProcess {
    pub fn daemonize(log_config: LoggingConfig) -> Result<Self> {
        let rpc = start_background_process::<Request, Response>()?;
        let mut mount_process = Self { rpc };
        mount_process.send_bootstrap(BootstrapConfig { log_config })?;
        mount_process.status_check()?;
        Ok(mount_process)
    }

    fn send_bootstrap(&mut self, config: BootstrapConfig) -> Result<()> {
        self.rpc.send_request(&Request::Bootstrap(config))?;
        let response: Response = self.rpc.recv_response(BOOTSTRAP_TIMEOUT)?;
        match response {
            Response::BootstrapAck => Ok(()),
            response => panic!("Unexpected response to Bootstrap: {response:?}"),
        }
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

/// Config the parent CLI ships to the daemon child as the first message
/// after the build-id handshake. Carries everything that can't be
/// rederived on the child's side from its own argv (which is just
/// `--daemon`). Currently just logging; new knobs (telemetry, debug
/// flags, runtime tunables) extend this struct rather than the
/// `Request`/`Response` enums.
#[derive(Debug, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub log_config: LoggingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    /// Must be the first request the daemon receives. Carries the user's
    /// resolved logging config (the daemon's own argv doesn't have `--log`).
    /// The daemon initializes logging from this and acks with
    /// [`Response::BootstrapAck`] before any other request is processed.
    Bootstrap(BootstrapConfig),
    StatusCheckRequest,
    MountRequest(MountArgs),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    BootstrapAck,
    StatusCheckResponse,
    MountResponse(Result<(), MountError>),
}

pub fn background_main(rpc_server: RpcServer<Request, Response>) -> ! {
    // Now we're post-daemonization, so we can initialize tokio.
    let runtime = crate::init_tokio();
    runtime.block_on(background_async_main(rpc_server))
}

async fn background_async_main(mut rpc_server: RpcServer<Request, Response>) -> ! {
    // The first request MUST be Bootstrap. We can't initialize logging
    // before this point (the daemon's own argv only has `--daemon` — no
    // `--log` flags) so any error here goes to stderr instead of the log.
    //
    // `init_logging!` runs *before* the ack: if it panics or otherwise
    // fails, the parent never sees `BootstrapAck`, its
    // `recv_response(BOOTSTRAP_TIMEOUT)` surfaces a clean
    // "bootstrap failed" error, and no later RPC requests get mishandled
    // by a half-initialized daemon.
    if let Err(err) = handle_bootstrap(&mut rpc_server, |log_config| {
        clap_logflag::init_logging!(log_config, cryfs_cli_utils::DEFAULT_LOG_LEVEL);
        Ok(())
    }) {
        eprintln!("cryfs --daemon: {err:#}");
        std::process::exit(127);
    }

    while let Ok(request) = rpc_server.next_request() {
        match request {
            Request::Bootstrap(_) => {
                // Re-bootstrap mid-flight isn't supported today; log a
                // warning so an unexpected duplicate is visible, but ack
                // anyway so the parent doesn't hang on `recv_response`.
                log::warn!("received unexpected duplicate Bootstrap; ignoring");
                let _ = rpc_server.send_response(&Response::BootstrapAck);
            }
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

/// Read the first request off `rpc_server`, require it to be
/// [`Request::Bootstrap`], hand its [`LoggingConfig`] to `install_logging`,
/// and only then ack with [`Response::BootstrapAck`].
///
/// The strict receive → install → ack ordering matters: if `install_logging`
/// fails (or panics), the parent never sees an ack and its
/// `recv_response(BOOTSTRAP_TIMEOUT)` surfaces a clean "bootstrap failed"
/// error instead of finding out later via a mysterious EOF on the next
/// typed RPC.
///
/// The callback shape exists so tests can drive the protocol without
/// invoking `clap_logflag::init_logging!`, which globally installs a logger
/// (`log::set_logger` accepts exactly one call per process — and tests share
/// a process).
fn handle_bootstrap<F>(
    rpc_server: &mut RpcServer<Request, Response>,
    install_logging: F,
) -> Result<()>
where
    F: FnOnce(LoggingConfig) -> Result<()>,
{
    let request = rpc_server
        .next_request()
        .context("failed to receive bootstrap request")?;
    let cfg = match request {
        Request::Bootstrap(cfg) => cfg,
        other => bail!("expected Bootstrap as first request, got {other:?}"),
    };
    install_logging(cfg.log_config).context("failed to install logging")?;
    rpc_server
        .send_response(&Response::BootstrapAck)
        .context("failed to send bootstrap ack")?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::RpcConnection;
    use clap_logflag::{LogDestination, LogDestinationConfig};
    use log::LevelFilter;
    use std::path::PathBuf;

    /// Build a non-trivial LoggingConfig that exercises every
    /// [`LogDestination`] variant and a mix of present / absent
    /// per-destination level filters. Used by both the postcard round-trip
    /// test and the over-the-pipe round-trip test so the two cover the same
    /// wire shape.
    fn sample_log_config() -> LoggingConfig {
        LoggingConfig::new(vec![
            LogDestinationConfig {
                destination: LogDestination::Stderr,
                level: Some(LevelFilter::Warn),
            },
            LogDestinationConfig {
                destination: LogDestination::Syslog,
                level: None,
            },
            LogDestinationConfig {
                destination: LogDestination::File(PathBuf::from("/tmp/cryfs-test.log")),
                level: Some(LevelFilter::Info),
            },
        ])
    }

    /// Round-trip a `BootstrapConfig` through postcard (the encoding used on
    /// the IPC pipe). Catches regressions where clap-logflag's serde feature
    /// changes representation or where a new `LogDestination` variant gets
    /// added without matching serde derives.
    #[test]
    fn bootstrap_config_postcard_round_trips() {
        let original = BootstrapConfig {
            log_config: sample_log_config(),
        };
        let bytes = postcard::to_stdvec(&original).expect("postcard encode");
        let restored: BootstrapConfig =
            postcard::from_bytes(&bytes).expect("postcard decode");
        assert_eq!(original.log_config, restored.log_config);
    }

    /// End-to-end protocol test for the bootstrap step: a stand-in parent
    /// sends `Request::Bootstrap(cfg)` over a real `RpcConnection`, the
    /// daemon-side `handle_bootstrap` consumes it, invokes the
    /// `install_logging` callback (which captures the config instead of
    /// calling `clap_logflag::init_logging!`), then acks. The parent
    /// confirms it received `Response::BootstrapAck`. Verifies the bytes
    /// that travel across the pipe carry the config faithfully.
    ///
    /// We don't use the real `init_logging` here because `log::set_logger`
    /// accepts exactly one call per process and tests share a process.
    #[test]
    fn bootstrap_round_trips_over_rpc() {
        use std::sync::{Arc, Mutex};

        let (mut server, mut client) = RpcConnection::<Request, Response>::new_pipe()
            .unwrap()
            .into_server_and_client();
        let sent_config = sample_log_config();
        let captured: Arc<Mutex<Option<LoggingConfig>>> = Arc::new(Mutex::new(None));

        let captured_for_thread = Arc::clone(&captured);
        let server_handle = std::thread::spawn(move || {
            handle_bootstrap(&mut server, |log_config| {
                *captured_for_thread.lock().unwrap() = Some(log_config);
                Ok(())
            })
        });

        client
            .send_request(&Request::Bootstrap(BootstrapConfig {
                log_config: sent_config.clone(),
            }))
            .expect("parent send_request");
        let response = client
            .recv_response(Duration::from_secs(5))
            .expect("parent recv_response");
        assert!(
            matches!(response, Response::BootstrapAck),
            "expected BootstrapAck, got {response:?}"
        );

        server_handle
            .join()
            .expect("daemon thread panicked")
            .expect("handle_bootstrap returned error");
        let received = captured
            .lock()
            .unwrap()
            .take()
            .expect("install_logging callback was never invoked");
        assert_eq!(received, sent_config);
    }

    /// Pins the receive → install → ack ordering: if `install_logging`
    /// fails, the daemon must NOT send `BootstrapAck`. Otherwise the parent
    /// would think bootstrap succeeded and only discover the failure
    /// later via a mysterious EOF / timeout on the next RPC, hiding the
    /// root cause.
    #[test]
    fn handle_bootstrap_does_not_ack_if_install_logging_fails() {
        let (mut server, mut client) = RpcConnection::<Request, Response>::new_pipe()
            .unwrap()
            .into_server_and_client();

        let server_handle = std::thread::spawn(move || {
            handle_bootstrap(&mut server, |_log_config| {
                Err(anyhow!("simulated install_logging failure"))
            })
        });

        client
            .send_request(&Request::Bootstrap(BootstrapConfig {
                log_config: sample_log_config(),
            }))
            .expect("parent send_request");

        // No ack should arrive. The server thread bails before send_response,
        // dropping its end of the pipe; the parent's recv surfaces EOF or
        // a related error — either way, NOT a successful BootstrapAck.
        let response = client.recv_response(Duration::from_millis(500));
        assert!(
            response.is_err(),
            "expected no BootstrapAck when install_logging fails, got {response:?}"
        );

        let err = server_handle
            .join()
            .expect("daemon thread panicked")
            .err()
            .expect("handle_bootstrap should return error when install_logging fails");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("failed to install logging"),
            "expected install-logging error context, got: {msg}"
        );
    }

    /// Defensive: if the parent ever sends a non-Bootstrap as the first
    /// request (a protocol-level bug), the daemon must surface an error
    /// rather than proceed without a logger or silently mishandle the
    /// request. Pins the early-error path in `handle_bootstrap`. The
    /// callback must NOT be invoked in this case.
    #[test]
    fn handle_bootstrap_rejects_non_bootstrap_first_request() {
        let (mut server, mut client) = RpcConnection::<Request, Response>::new_pipe()
            .unwrap()
            .into_server_and_client();

        let server_handle = std::thread::spawn(move || {
            handle_bootstrap(&mut server, |_log_config| {
                panic!("install_logging must not be invoked on a non-Bootstrap first request");
            })
        });

        client
            .send_request(&Request::StatusCheckRequest)
            .expect("parent send_request");

        let err = server_handle
            .join()
            .expect("daemon thread panicked")
            .err()
            .expect("expected handle_bootstrap to reject non-Bootstrap first request");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("expected Bootstrap"),
            "expected 'expected Bootstrap' in error, got: {msg}"
        );
    }
}
