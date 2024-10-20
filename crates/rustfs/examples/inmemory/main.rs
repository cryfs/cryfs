mod device;
mod dir;
mod file;
mod inode_metadata;
mod node;
mod symlink;

use device::InMemoryDevice;

const USAGE: &str = "Usage: inmemoryfs [mountdir]";

fn main() {
    // TODO Use clap for argument parsing

    env_logger::init();

    let mut args = std::env::args();
    let _executable = args.next().unwrap();
    let mountdir = args.next().expect(USAGE);
    assert!(args.next().is_none(), "{}", USAGE);

    let fs = |uid, gid| InMemoryDevice::new(uid, gid);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("rustfs")
        .build()
        .unwrap();
    cryfs_rustfs::backend::fuser::mount(fs, mountdir, runtime.handle().clone(), || {}).unwrap();
}
