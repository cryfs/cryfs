//! Mount orchestration for the `cryfs` binary: turning fully resolved
//! [`MountArgs`] into a running FUSE mount, either in this process (foreground
//! mode) or inside the daemon that `daemonizable` spawns (background mode).
//!
//! Everything in here runs *after* argument parsing, password prompting and
//! config loading are done (that is `cli.rs`'s job). In background mode this
//! module is the only cryfs code the detached daemon executes: the parent
//! ships [`MountArgs`] over RPC and [`background_main`] drives the mount.

mod background_process;
mod mount;
mod unmount_trigger;

// The RPC types (`Request`, `Response` and everything reachable through their
// fields, e.g. `MountArgs` and `MountError`) must stay nominally `pub` even
// though this module is private: `Request`/`Response` are the associated types
// of `impl Daemonizable for CryfsApp`, and `CryfsApp` is public, so rustc
// rejects `pub(crate)` there with E0446. A `pub` item in a private module is
// fine (reachable but unnameable from outside the crate), same as
// `args::CryfsArgs` in `impl Application for Cli`.
pub use background_process::{Request, Response};
pub use mount::{CreateOrLoad, FuseOption, MountArgs};

// The functions, by contrast, are plain crate internals.
pub(crate) use background_process::{background_main, parent_mount_filesystem};
pub(crate) use mount::mount_filesystem;

pub(crate) fn init_tokio() -> tokio::runtime::Runtime {
    // TODO Test if a different runtime, e.g. monoio, is faster for us because we have heavy file I/O operations with mostly predictable workloads. See https://chesedo.me/blog/monoio-introduction/
    // TODO Runtime settings
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("cryfs")
        .enable_all()
        .build()
        .unwrap()
}
