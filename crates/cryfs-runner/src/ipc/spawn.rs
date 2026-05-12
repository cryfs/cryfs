use std::ffi::OsStr;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use command_fds::{CommandFdExt, FdMapping};
use serde::{Serialize, de::DeserializeOwned};
use tokio::runtime::Handle;

use crate::ipc::RpcConnection;

use super::{RpcClient, RpcServer};

/// Fd numbers the fork+exec child receives its inherited pipe ends on.
/// Matches `sd_listen_fds(3)`-style convention (parent-provided fds start at 3).
const CHILD_REQUEST_RECV_FD: i32 = 3;
const CHILD_RESPONSE_SEND_FD: i32 = 4;

/// Sentinel argv flag identifying a re-exec'd cryfs binary as the daemon
/// child. Hidden+exclusive at the clap level so users can't reach it by
/// accident; `Cli::main` dispatches to `run_as_background_daemon` when present.
const DAEMON_FLAG: &str = "--daemon";

/// How long the parent CLI will wait for the daemon to send its build-id
/// handshake after fork+exec. Generous because a healthy daemon only has to
/// `fstat` two fds and call `setsid` before sending — sub-millisecond on any
/// real hardware. The timeout matters when the parent accidentally exec'd a
/// wrong binary that opens fd 4 but never writes (or hangs); without a
/// bound the CLI would hang forever in that case.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Resolve the path to exec for the daemon child.
///
/// On Linux we pass the **literal string** `"/proc/self/exe"` to `execve`.
/// The kernel's magic-link resolver for that path returns `mm->exe_file` of
/// the calling process (via `proc_exe_link` / `nd_jump_link`), so the child
/// is loaded from the exact same inode the parent was loaded from — even if
/// `/usr/bin/cryfs` was replaced on disk between parent startup and this
/// `execve`. This is **not** the same as `std::env::current_exe()`, which
/// `readlink`s the symlink into a path string and then re-resolves it through
/// the filesystem; a future reader who "simplifies" this to `current_exe`
/// everywhere will silently lose the apt-upgrade-mid-run guarantee.
///
/// On non-Linux, `current_exe()` is the best we have. The build-id handshake
/// covers the gap.
fn daemon_exe_path() -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        Ok(PathBuf::from("/proc/self/exe"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        std::env::current_exe().context("env::current_exe() failed")
    }
}

/// Spawn the cryfs daemon as a separate process via fork+exec.
///
/// The current binary is re-execed with the [`DAEMON_FLAG`] sentinel argument
/// so its `main` can dispatch to [`crate::run_as_background_daemon`]. The
/// child receives the two pipe ends as fds [`CHILD_REQUEST_RECV_FD`] (3) and
/// [`CHILD_RESPONSE_SEND_FD`] (4). Every other parent fd is CLOEXEC (see
/// [`super::pipe::pipe`]) so the kernel closes them during `execve`.
///
/// Waits for a raw build-id handshake (not postcard-encoded) *from* the
/// daemon child before returning, bounded by [`HANDSHAKE_TIMEOUT`]. Rejects
/// the spawn if the bytes don't match this process's own compile-time build
/// id. This catches three classes of mistake at once:
///   - macOS-style binary replacement during the spawn window (the daemon
///     binary is a different cryfs build than the parent),
///   - a parent that accidentally exec'd a non-cryfs binary (no handshake
///     arrives → EOF or timeout),
///   - any future operator mistake exec'ing a stranger binary that happens
///     to write something to fd 4 (handshake bytes won't match build_id).
pub fn start_background_process<Request, Response>() -> Result<RpcClient<Request, Response>>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    if Handle::try_current().is_ok() {
        panic!(
            "Cannot daemonize a process if tokio is running. Please daemonize \
             before initializing tokio. See https://github.com/tokio-rs/tokio/issues/4301"
        );
    }

    let exe = daemon_exe_path()?;
    // The execve path is `/proc/self/exe` on Linux (kernel magic-link, see
    // `daemon_exe_path`), but that string would also become argv[0] by
    // default, so `ps`/`top` would show "/proc/self/exe --daemon" instead of
    // the actual binary name. Override argv[0] to the resolved path so
    // operators see a recognizable command line. Falls back to the exec
    // path if `current_exe()` fails — preserves correctness over cosmetics.
    let argv0 = std::env::current_exe().unwrap_or_else(|_| exe.clone());
    let client = start_background_process_inner::<Request, Response>(
        &exe,
        Some(argv0.as_path()),
        &[DAEMON_FLAG],
        &[],
    )?;
    validate_handshake_and_build_client(client)
}

/// Test-only variant: spawn an arbitrary helper binary instead of re-exec'ing
/// cryfs. Used by `tests/daemon_child_lifecycle.rs` and `spawn_fd_isolation`
/// to drive the spawn machinery against a controlled daemon. Does not send a
/// build-id handshake — the helper bin doesn't expect one.
pub fn start_background_process_with_exe<Request, Response>(
    exe: &Path,
    extra_env: &[(&OsStr, &OsStr)],
) -> Result<RpcClient<Request, Response>>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    if Handle::try_current().is_ok() {
        panic!(
            "Cannot spawn a background process if tokio is running. \
             Spawn before initializing tokio."
        );
    }
    start_background_process_inner(exe, None, &[], extra_env)
}

/// Common fork+exec machinery shared by [`start_background_process`] and
/// [`start_background_process_with_exe`].
fn start_background_process_inner<Request, Response>(
    exe: &Path,
    argv0: Option<&Path>,
    args: &[&str],
    extra_env: &[(&OsStr, &OsStr)],
) -> Result<RpcClient<Request, Response>>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    let rpc_pipes = RpcConnection::<Request, Response>::new_pipe()?;
    let (client, child_in_fd, child_out_fd) = rpc_pipes.into_client_and_child_fds();

    let mut cmd = Command::new(exe);
    if let Some(argv0) = argv0 {
        cmd.arg0(argv0);
    }
    cmd.args(args);
    for (k, v) in extra_env {
        cmd.env(k, v);
    }

    // TODO Replace `command-fds` with stdlib `CommandExt::fd` once
    // https://github.com/rust-lang/rust/pull/145687 lands and stabilizes.
    // The stdlib version is expected to route through `posix_spawn` +
    // `posix_spawn_file_actions_adddup2` when possible, which would close
    // the fork-after-multithread hazard (see https://github.com/tokio-rs/tokio/issues/4301).
    // `command-fds` itself uses `pre_exec` (it has no way around that
    // through `std::process::Command` today), so this switch is a code-shape
    // and edge-case-correctness improvement rather than a safety improvement
    // — but it puts the migration one line away when the stdlib API lands.
    cmd.fd_mappings(vec![
        FdMapping {
            parent_fd: child_in_fd,
            child_fd: CHILD_REQUEST_RECV_FD,
        },
        FdMapping {
            parent_fd: child_out_fd,
            child_fd: CHILD_RESPONSE_SEND_FD,
        },
    ])
    .context("failed to set up fd mappings for daemon child")?;

    let _child = cmd
        .spawn()
        .with_context(|| format!("failed to spawn daemon binary at {}", exe.display()))?;

    Ok(client)
}

/// Parent-side counterpart to [`send_handshake`]: read the build-id the
/// daemon sent and reject the spawn if it doesn't match this process's own
/// compile-time build id. Returns the client unchanged on match.
///
/// Must run before any postcard-typed RPC: a mismatch would otherwise let
/// the parent deserialize structured data from a daemon whose
/// Request/Response schemas may not agree.
fn validate_handshake_and_build_client<Request, Response>(
    mut client: RpcClient<Request, Response>,
) -> Result<RpcClient<Request, Response>>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    let received = client
        .recv_raw_handshake_with_timeout(HANDSHAKE_TIMEOUT)
        .context("failed to receive build-id handshake from daemon")?;
    let received_str = std::str::from_utf8(&received)
        .context("daemon sent a build-id that isn't valid UTF-8")?;
    let expected = crate::build_id();
    if received_str != expected {
        bail!(
            "parent and daemon binaries don't match (parent={expected}, \
             daemon={received_str}). Refusing to start."
        );
    }
    Ok(client)
}

/// Daemon-side counterpart to [`validate_handshake_and_build_client`]: send
/// this process's build id to the parent so the parent can confirm it
/// exec'd the binary it intended to. Must be called before any
/// postcard-typed RPC on `server`.
pub fn send_handshake<Request, Response>(
    server: &mut RpcServer<Request, Response>,
) -> Result<()>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned,
{
    server
        .send_raw_handshake(crate::build_id().as_bytes())
        .context("failed to send build-id handshake to parent")
}

#[cfg(test)]
mod handshake_tests {
    use super::*;
    use crate::ipc::RpcConnection;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize)]
    struct Req(u32);
    #[derive(Debug, Serialize, Deserialize)]
    struct Resp(u32);

    #[test]
    fn accepts_matching_build_id() {
        let (mut server, client) = RpcConnection::<Req, Resp>::new_pipe()
            .unwrap()
            .into_server_and_client();
        send_handshake(&mut server).unwrap();
        validate_handshake_and_build_client(client).expect("matching build_id");
    }

    #[test]
    fn rejects_mismatched_build_id() {
        let (mut server, client) = RpcConnection::<Req, Resp>::new_pipe()
            .unwrap()
            .into_server_and_client();
        server
            .send_raw_handshake(b"some-other-version-1.2.3")
            .unwrap();
        let err = validate_handshake_and_build_client(client)
            .err()
            .expect("mismatched build_id should be rejected");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("don't match"),
            "expected mismatch message, got: {msg}",
        );
        assert!(
            msg.contains("daemon=some-other-version-1.2.3"),
            "expected received string in message, got: {msg}",
        );
    }

    #[test]
    fn rejects_non_utf8_build_id() {
        let (mut server, client) = RpcConnection::<Req, Resp>::new_pipe()
            .unwrap()
            .into_server_and_client();
        // 0xff is never valid as a leading UTF-8 byte.
        server.send_raw_handshake(&[0xff, 0xfe]).unwrap();
        let err = validate_handshake_and_build_client(client)
            .err()
            .expect("non-UTF-8 should be rejected");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("isn't valid UTF-8"),
            "expected UTF-8 message, got: {msg}",
        );
    }

    #[test]
    fn rejects_when_daemon_closes_before_handshake() {
        // Daemon dies (or was a non-cryfs binary that just exited) before
        // writing the handshake. Parent's `recv_raw_timeout` sees EOF and
        // bails — must surface as an error rather than hang.
        let (server, client) = RpcConnection::<Req, Resp>::new_pipe()
            .unwrap()
            .into_server_and_client();
        drop(server);
        let err = validate_handshake_and_build_client(client)
            .err()
            .expect("missing handshake should be rejected");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("failed to receive build-id handshake"),
            "expected handshake-receive message, got: {msg}",
        );
    }

    #[test]
    fn rejects_when_daemon_hangs_without_sending() {
        // Daemon (or a wrong binary like a hung `/bin/cat`) holds fd 4 open
        // but never writes. Without a timeout the parent would hang forever;
        // bounded `recv_raw_handshake_with_timeout` surfaces a timeout error
        // instead. Tiny timeout so the test doesn't actually wait 10s.
        let (_server_keepalive, mut client) = RpcConnection::<Req, Resp>::new_pipe()
            .unwrap()
            .into_server_and_client();
        let err = client
            .recv_raw_handshake_with_timeout(Duration::from_millis(50))
            .err()
            .expect("hung daemon should be rejected via timeout");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("Timeout"),
            "expected timeout message, got: {msg}",
        );
    }
}

/// Helper for the daemon side of [`start_background_process_with_exe`]: parse
/// the conventional fds 3 and 4 into an [`RpcServer`]. Aborts with a
/// human-readable message if the fds are not pipes — almost always the result
/// of a curious user invoking the daemon entry point manually from a shell.
///
/// Used by the test helper binary. Production cryfs goes through
/// [`crate::run_as_background_daemon`], which additionally validates the
/// build-id handshake before returning the server.
pub fn rpc_server_from_inherited_fds<Request, Response>() -> Result<RpcServer<Request, Response>>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned,
{
    for (label, fd) in [
        ("request-recv", CHILD_REQUEST_RECV_FD),
        ("response-send", CHILD_RESPONSE_SEND_FD),
    ] {
        let mut statbuf: libc::stat = unsafe { std::mem::zeroed() };
        if unsafe { libc::fstat(fd, &mut statbuf) } < 0 {
            bail!(
                "fd {fd} ({label}) is not open. This entry point is internal to cryfs; \
                 do not invoke it directly. ({})",
                std::io::Error::last_os_error()
            );
        }
        if statbuf.st_mode & libc::S_IFMT != libc::S_IFIFO {
            bail!(
                "fd {fd} ({label}) is not a pipe (st_mode={:#o}). This entry point \
                 is internal to cryfs; do not invoke it directly.",
                statbuf.st_mode
            );
        }
    }
    Ok(unsafe { RpcServer::from_raw_fds(CHILD_REQUEST_RECV_FD, CHILD_RESPONSE_SEND_FD) })
}
