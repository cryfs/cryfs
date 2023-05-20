// TODO Go through all API calls we're doing (e.g. std::fs, tokio::fs, nix::) and make sure we're using the API correctly
//      and handle errors that can happen.

mod device;
mod dir;
mod errors;
mod file;
mod node;
mod openfile;
mod symlink;
mod utils;

use device::PassthroughDevice;

const USAGE: &str = "Usage: passthroughfs [basedir] [mountdir]";

fn main() {
    // TODO Use clap for argument parsing

    env_logger::init();

    let mut args = std::env::args();
    let _executable = args.next().unwrap();
    let basedir = args.next().expect(USAGE);
    let mountdir = args.next().expect(USAGE);
    assert!(args.next().is_none(), "{}", USAGE);

    let device = PassthroughDevice::new(basedir.into());

    cryfs_rustfs::backend::fuse_mt::mount(|_uid, _gid| device, mountdir).unwrap();
}
