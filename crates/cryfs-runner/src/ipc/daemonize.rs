use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use command_fds::{CommandFdExt, FdMapping};
use daemonize::{Daemonize, Stdio};
use serde::{Serialize, de::DeserializeOwned};
use tokio::runtime::Handle;

use crate::ipc::RpcConnection;

use super::{RpcClient, RpcServer};

/// Fd numbers the fork+exec child receives its inherited pipe ends on.
/// Matches `sd_listen_fds(3)`-style convention (parent-provided fds start at 3).
const CHILD_REQUEST_RECV_FD: i32 = 3;
const CHILD_RESPONSE_SEND_FD: i32 = 4;

// Legacy fork-without-exec daemon spawn (uses the `daemonize` crate). Two known issues:
//
// 1. Inherited fds aren't closed. The `daemonize` crate only dup2s stdio; it leaves
//    fds >= 3 alone, and `interprocess` creates pipes without `O_CLOEXEC` (we patch
//    that up in `pipe()` but the race is documented there). The daemonized child
//    therefore can hold copies of fds open in the parent at fork time.
//
// 2. Fork-after-multithread hazard. POSIX restricts post-fork code in a multithreaded
//    program to async-signal-safe operations only; any mutex held by another thread
//    at fork time stays locked forever in the child. This is why we panic if tokio
//    is already running. See https://github.com/tokio-rs/tokio/issues/4301.
//
// New code should prefer `start_background_process_with_exe`, which does fork+exec
// and gets a fresh process image. We keep this entry point for code paths that still
// run an in-process daemon body.
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

/// Spawn a separate process as the background daemon via fork+exec.
///
/// Unlike [`start_background_process`], this re-execs a binary at `exe` rather
/// than running an in-process function in a forked child. The child sees the
/// two pipe ends as fds [`CHILD_REQUEST_RECV_FD`] (3) and
/// [`CHILD_RESPONSE_SEND_FD`] (4). Every other fd in the parent is CLOEXEC
/// (see [`super::pipe::pipe`]), so `execve` closes them.
///
/// `extra_env` is forwarded as additional environment variables on the child.
/// Used by tests to drive the helper binary's behavior; not used in production.
///
/// The returned [`RpcClient`] is the parent's end of the bidirectional pipe.
/// The child handle is dropped on purpose: this function spawns a long-lived
/// daemon that outlives its parent (see the `daemon_survives_parent_exit`
/// regression test).
pub fn start_background_process_with_exe<Request, Response>(
    exe: &Path,
    extra_env: &[(&OsStr, &OsStr)],
) -> Result<RpcClient<Request, Response>>
where
    Request: Serialize + DeserializeOwned,
    Response: Serialize + DeserializeOwned + Send,
{
    // Note: we don't guard against tokio being running here. Pipe creation
    // (`RpcConnection::new_pipe` → `pipe()`) panics if tokio is up; see the
    // CLOEXEC-race discussion in `pipe()`'s docs. The `Command::spawn` below
    // would *also* suffer the fork-after-multithread hazard under tokio
    // because `command-fds` uses `pre_exec` internally (see TODO below), but
    // the pipe guard catches the unsupported case before we get here.
    let rpc_pipes = RpcConnection::<Request, Response>::new_pipe()?;
    let (client, child_in_fd, child_out_fd) = rpc_pipes.into_client_and_child_fds();

    let mut cmd = Command::new(exe);
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

/// Helper for the daemon side of [`start_background_process_with_exe`]: parse
/// the conventional fds 3 and 4 into an [`RpcServer`]. Aborts with a
/// human-readable message if the fds are not pipes — almost always the result
/// of a curious user invoking the daemon entry point manually from a shell.
///
/// The check is best-effort: it catches the common case (TTY, regular file)
/// without going overboard. We accept anything `fstat` reports as a FIFO.
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
    Ok(unsafe {
        RpcServer::from_raw_fds(CHILD_REQUEST_RECV_FD, CHILD_RESPONSE_SEND_FD)
    })
}

// Lifecycle/flake-prone tests previously lived here as unit tests with
// in-process fn-pointer daemons. They now live in
// `tests/daemon_child_lifecycle.rs` and spawn the dedicated test helper
// binary `cryfs-runner-test-background` via fork+exec, so the daemon child
// runs in a clean process image with only fds 3/4 inherited.
