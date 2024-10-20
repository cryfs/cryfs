//#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

mod background_process;
mod ipc;
mod mounter;
mod runner;

pub use mounter::Mounter;
pub use runner::{CreateOrLoad, MountArgs};

cryfs_version::assert_cargo_version_equals_git_version!();

pub fn init_tokio() -> tokio::runtime::Runtime {
    // TODO Runtime settings
    tokio::runtime::Builder::new_multi_thread()
        .thread_name("cryfs")
        .enable_all()
        .build()
        .unwrap()
}
