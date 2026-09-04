//! Daemon-side and parent-side helpers for the cryfs mount RPC. The
//! daemonizable framework owns the spawn / handshake; this module defines
//! cryfs's own typed [`Request`] / [`Response`] and provides the daemon's
//! request loop ([`background_main`]) plus a thin parent-side wrapper for
//! sending the mount request and translating the reply back into a `CliError`.
//!
//! The daemon's logging config travels in [`Request::MountRequest`]: the
//! re-exec'd daemon has empty argv and can't re-parse `--log`, so the parent
//! resolves it and ships it over RPC, and the daemon installs it before
//! mounting (there is no framework bootstrap payload anymore).

use std::sync::Arc;

use anyhow::anyhow;
use clap_logflag::LoggingConfig;
use cryfs_cli_utils::{CliError, CliErrorKind, CliResultExtFn, DEFAULT_LOG_LEVEL};
use daemonizable::{RpcClient, RpcServer};
use serde::{Deserialize, Serialize};

use crate::MountArgs;

/// Mount-side error shipped over RPC. The daemon converts its internal
/// `CliError` into this Serialize-friendly shape; the parent reconstitutes
/// a `CliError` from it.
#[derive(Serialize, Deserialize, Debug)]
pub struct MountError {
    pub cli_error_kind: CliErrorKind,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    MountRequest {
        mount_args: MountArgs,
        /// Logging config the daemon installs before mounting. Resolved on the
        /// parent side (the daemon's argv is empty, so it can't parse `--log`)
        /// and shipped here because there is no framework bootstrap payload.
        log_config: LoggingConfig,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    MountResponse(Result<(), MountError>),
}

/// Daemon-side entry point. The daemonizable framework hands us an
/// `rpc_server` whose build-id handshake is already validated. Initialize
/// tokio inside this clean process image and drive the typed request loop
/// until the parent drops its end of the channel. Logging is not yet installed:
/// the daemon installs it from the first request's `log_config` before it
/// mounts (see [`Request::MountRequest`]).
pub fn background_main(rpc_server: RpcServer<Request, Response>) -> ! {
    let runtime = crate::init_tokio();
    runtime.block_on(background_async_main(rpc_server))
}

async fn background_async_main(mut rpc_server: RpcServer<Request, Response>) -> ! {
    while let Ok(request) = rpc_server.next_request() {
        match request {
            Request::MountRequest {
                mount_args,
                log_config,
            } => {
                // Install the parent-resolved logging config before anything
                // logs. The daemon's argv is empty, so this is the only way it
                // learns the user's `--log` choice. Nothing logs before this
                // point (the framework startup uses eprintln, not the facade).
                clap_logflag::init_logging!(log_config, DEFAULT_LOG_LEVEL);
                let on_successfully_mounted = || {
                    // Report success to the parent. If this fails, the parent is
                    // gone (e.g. it was Ctrl+C'd mid-mount); returning the error
                    // makes `mount_filesystem` unmount instead of serving a mount
                    // nobody is waiting on — do NOT panic or keep it alive.
                    rpc_server.send_response(&Response::MountResponse(Ok(())))?;
                    // Detach inherited stdin/stdout/stderr from the user's
                    // terminal now that the user-visible operation succeeded.
                    // A failure here is non-fatal — the daemon keeps serving,
                    // it just stays attached to the (now-backgrounded) terminal.
                    if let Err(err) = daemonizable::detach_stdio() {
                        log::warn!("Failed to detach daemon stdio from the terminal: {err}");
                    }
                    Ok(())
                };
                let mount_result =
                    super::runner::mount_filesystem(mount_args, on_successfully_mounted).await;
                match mount_result {
                    Ok(()) => {
                        // Normal path: the filesystem was mounted, served, and
                        // later unmounted (`mount_filesystem` blocks until
                        // then), with the success response already sent inside
                        // `on_successfully_mounted`. The notification-failed
                        // path (parent gone) also lands here after unmounting —
                        // there's likewise nothing to send. Either way, nothing
                        // to do; the loop below then hits EOF and exits.
                    }
                    Err(err) => {
                        let mount_error = MountError {
                            cli_error_kind: err.kind,
                            message: format!("{:?}", err.error),
                        };
                        // Ignore errors because the parent process likely
                        // has exited if the file system was already mounted
                        // for some time.
                        let _ =
                            rpc_server.send_response(&Response::MountResponse(Err(mount_error)));
                    }
                }
            }
        }
    }

    // TODO Should we make this into a panic and introduce a clean shutdown
    // where Client Drop drops the Server? Error getting request, parent
    // process probably exited or closed the channel.
    std::process::exit(0);
}

/// Parent-side helper: send the mount request to the daemon, wait for the
/// response, translate the result back into a `CliError`.
pub fn parent_mount_filesystem(
    rpc: &mut RpcClient<Request, Response>,
    mount_args: MountArgs,
    log_config: LoggingConfig,
) -> Result<(), CliError> {
    rpc.send_request(&Request::MountRequest {
        mount_args,
        log_config,
    })
    .map_cli_error(|_| CliErrorKind::UnspecifiedError)?;
    // Block until the daemon reports the mount result. No timeout: a mount can
    // legitimately take a long time (large vault, slow/networked storage), and
    // a fixed deadline would spuriously fail a healthy-but-slow daemon. If the
    // daemon dies instead, its channel closes and this returns an error at once.
    let response = rpc
        .recv_response_blocking()
        .map_cli_error(|_| CliErrorKind::UnspecifiedError)?;
    match response {
        Response::MountResponse(Ok(())) => Ok(()),
        Response::MountResponse(Err(err)) => Err(CliError {
            kind: err.cli_error_kind,
            error: Arc::new(anyhow!("{}", err.message)),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use byte_unit::Byte;
    use clap_logflag::{LogDestination, LogDestinationConfig};
    use cryfs_blockstore::{AllowIntegrityViolations, ClientId};
    use cryfs_config::{
        config::{CryConfig, FilesystemId},
        localstate::LocalStateDir,
    };
    use cryfs_rustfs::AtimeUpdateBehavior;
    use daemonizable::in_process_rpc_pair;
    use log::LevelFilter;
    use std::num::NonZeroU32;
    use std::path::PathBuf;

    use crate::CreateOrLoad;

    fn test_mount_args() -> MountArgs {
        MountArgs {
            vaultdir: PathBuf::from("/some/vaultdir"),
            mountdir: PathBuf::from("/some/mountdir"),
            config: CryConfig {
                root_blob: "4a7a231be5055939468cb4a17087053e".to_string(),
                enc_key: "4e4f500b608039d5385f9f977f785288522c7f2f7e1af18a1974dce9c454720e"
                    .to_string(),
                cipher: "aes-256-gcm".to_string(),
                format_version: "0.10".to_string(),
                created_with_version: "1.0.0".to_string(),
                last_opened_with_version: "1.0.0".to_string(),
                blocksize: Byte::from_u64(32 * 1024),
                filesystem_id: FilesystemId::from_hex("8de43828c75c9bb10cac251eaf4ad9bd").unwrap(),
                exclusive_client_id: None,
            },
            allow_integrity_violations: AllowIntegrityViolations::DontAllowViolations,
            create_or_load: CreateOrLoad::LoadExistingFilesystem,
            my_client_id: ClientId {
                id: NonZeroU32::new(10).unwrap(),
            },
            local_state_dir: LocalStateDir::new(PathBuf::from("/some/statedir")),
            unmount_idle: None,
            fuse_options: Box::new([]),
            atime_behavior: AtimeUpdateBehavior::Relatime,
        }
    }

    /// A logging config with every destination shape the daemon can be asked
    /// to install. `LoggingConfig::disabled()` is the empty-vec case and would
    /// prove nothing about the wire.
    fn test_log_config() -> LoggingConfig {
        LoggingConfig::new(vec![
            LogDestinationConfig {
                destination: LogDestination::File(PathBuf::from("/var/log/cryfs.log")),
                level: Some(LevelFilter::Debug),
            },
            LogDestinationConfig {
                destination: LogDestination::Syslog,
                level: None,
            },
            LogDestinationConfig {
                destination: LogDestination::Stderr,
                level: Some(LevelFilter::Warn),
            },
        ])
    }

    #[test]
    fn parent_mount_filesystem_returns_ok_on_success_response() {
        let (mut server, mut client) =
            in_process_rpc_pair::<Request, Response>().expect("create in-process rpc pair");

        let daemon = std::thread::spawn(move || {
            let Request::MountRequest {
                mount_args,
                log_config,
            } = server
                .next_request()
                .expect("daemon: receive mount request");
            // The typed request must round-trip the args, not just a marker.
            // postcard is not self-describing, so a field whose serde shape it
            // cannot carry fails HERE at runtime rather than at compile time —
            // which is the whole reason this goes through a real channel.
            // Compared through Debug because MountArgs has no PartialEq: the
            // derived Debug prints every field, so a field that fails to
            // survive the wire still shows up here.
            assert_eq!(
                format!("{:?}", test_mount_args()),
                format!("{mount_args:?}")
            );
            // The daemon installs this before mounting; if it does not survive
            // the wire the user silently gets no logging.
            assert_eq!(test_log_config(), log_config);
            server
                .send_response(&Response::MountResponse(Ok(())))
                .expect("daemon: send response");
        });

        let result = parent_mount_filesystem(&mut client, test_mount_args(), test_log_config());
        daemon.join().expect("daemon thread panicked");
        result.expect("expected Ok from parent_mount_filesystem");
    }

    #[test]
    fn parent_mount_filesystem_reconstructs_cli_error_from_mount_error() {
        let (mut server, mut client) =
            in_process_rpc_pair::<Request, Response>().expect("create in-process rpc pair");

        let daemon = std::thread::spawn(move || {
            let Request::MountRequest { .. } = server
                .next_request()
                .expect("daemon: receive mount request");
            server
                .send_response(&Response::MountResponse(Err(MountError {
                    cli_error_kind: CliErrorKind::WrongPasswordOrCorruptedConfigFile,
                    message: "simulated mount failure".to_string(),
                })))
                .expect("daemon: send response");
        });

        let err =
            parent_mount_filesystem(&mut client, test_mount_args(), LoggingConfig::disabled())
                .expect_err("expected Err from parent_mount_filesystem");
        daemon.join().expect("daemon thread panicked");
        assert_eq!(CliErrorKind::WrongPasswordOrCorruptedConfigFile, err.kind);
        let message = format!("{}", err.error);
        assert!(
            message.contains("simulated mount failure"),
            "expected the daemon's error message to survive the RPC translation, got: {message}"
        );
    }
}
