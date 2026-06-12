//#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod background_process;
mod ipc;
mod mounter;
mod runner;
mod unmount_trigger;

pub use cryfs_rustfs::AtimeUpdateBehavior;
pub use mounter::Mounter;
pub use runner::{CreateOrLoad, FuseOption, MountArgs, make_device};

// Exposed for integration tests in `tests/` (daemon_roundtrip,
// daemon_child_lifecycle, daemon_survives_parent_exit, spawn_fd_isolation).
// Not stable API; everything here may change without notice.
#[doc(hidden)]
pub use ipc::{
    RpcClient, RpcServer, rpc_server_from_inherited_fds, start_background_process,
    start_background_process_with_exe,
};

cryfs_version::assert_cargo_version_equals_git_version!();

/// Compile-time build identifier baked into both the parent CLI and the daemon
/// child. Used as the build-id handshake: parent sends this over the pipe
/// immediately after spawning the daemon; daemon refuses to start if its own
/// build id doesn't match. Catches the macOS case where the on-disk binary
/// could be replaced between parent startup and child exec (no kernel
/// `/proc/self/exe` magic-link pin there), and any future operator mistake
/// that exec's a stranger binary.
const BUILD_ID: cryfs_version::VersionInfo<'static, 'static, &'static str> =
    cryfs_version::package_version!();

/// Stringified build id sent over the handshake pipe.
pub fn build_id() -> String {
    BUILD_ID.to_string()
}

/// Entry point for the daemon child of the fork+exec spawn. The cryfs CLI's
/// `main` dispatches here when it sees the hidden `--daemon` flag.
///
/// This function diverges: it either runs the background mount loop forever
/// or aborts the process. It does *not* install a panic hook or initialize
/// logging — `cryfs_cli_utils::run::<Cli>()` already did both before
/// `Cli::main` reached here.
///
/// Order of operations:
/// 1. `fstat` fds 3 and 4 — fail loudly if a curious user invoked `--daemon`
///    by hand. Done first because it's the only reversible step.
/// 2. `setsid()` — without this the daemon dies on SIGHUP when the parent's
///    controlling terminal goes away (e.g. user closes the shell).
/// 3. Send the build-id handshake to the parent. The parent compares it to
///    its own [`build_id`] and rejects the spawn on mismatch; without this
///    write the parent times out and reports a spawn failure. Doing it
///    *here* (rather than parent-side validation in the daemon) means a
///    parent that accidentally exec'd a non-cryfs binary surfaces the
///    mistake — a non-cryfs child won't send the expected bytes.
/// 4. Hand the [`ipc::RpcServer`] to `background_process::background_main`,
///    which initializes tokio inside this clean process image and serves the
///    mount RPC until the parent drops its client.
pub fn run_as_background_daemon() -> ! {
    let mut server: ipc::RpcServer<background_process::Request, background_process::Response> =
        match ipc::rpc_server_from_inherited_fds() {
            Ok(s) => s,
            Err(err) => {
                eprintln!("cryfs --daemon: {err:#}");
                std::process::exit(2);
            }
        };

    // setsid is fatal on failure: without a new session the daemon would die
    // along with the parent's controlling terminal.
    if unsafe { libc::setsid() } < 0 {
        eprintln!(
            "cryfs --daemon: setsid() failed: {}",
            std::io::Error::last_os_error()
        );
        std::process::exit(1);
    }

    if let Err(err) = ipc::send_handshake(&mut server) {
        eprintln!("cryfs --daemon: {err:#}");
        std::process::exit(127);
    }

    background_process::background_main(server)
}

pub fn init_tokio() -> tokio::runtime::Runtime {
    // TODO Test if a different runtime, e.g. monoio, is faster for us because we have heavy file I/O operations with mostly predictable workloads. See https://chesedo.me/blog/monoio-introduction/
    // TODO Runtime settings
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("cryfs")
        .enable_all()
        .build()
        .unwrap()
}
