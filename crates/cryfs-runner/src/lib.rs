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

cryfs_version::assert_cargo_version_equals_git_version!();

pub fn init_tokio() -> tokio::runtime::Runtime {
    // TODO Test if a different runtime, e.g. monoio, is faster for us because we have heavy file I/O operations with mostly predictable workloads. See https://chesedo.me/blog/monoio-introduction/
    // TODO Runtime settings
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("cryfs")
        .enable_all()
        .build()
        .unwrap()
}
