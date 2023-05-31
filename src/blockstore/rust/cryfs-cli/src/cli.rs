use anyhow::Result;
use clap::Parser;

use super::env;
use crate::args::Args;
use cryfs_cryfs::config::{Console, PasswordProvider};
use cryfs_version::VersionInfo;

const CRYFS_VERSION: VersionInfo = cryfs_cryfs::CRYFS_VERSION;

pub struct Cli {
    is_noninteractive: bool,
}

impl Cli {
    pub fn new() -> Self {
        let is_noninteractive = env::is_noninteractive();
        Self { is_noninteractive }
    }

    pub fn main(&self) -> Result<()> {
        _show_version();

        let args = Args::parse();

        if args.version {
            // No need to show version because we've already shown it, let's just exit
            return Ok(());
        }

        if args.show_ciphers {
            for cipher in cryfs_cryfs::config::ALL_CIPHERS {
                println!("{}", cipher);
            }
            return Ok(());
        }

        println!("Basedir: {:?}\nMountdir: {:?}", args.basedir, args.mountdir,);
        Ok(())
    }
}

// TODO (manually) test this
fn _show_version() {
    eprintln!("CryFS Version {}", CRYFS_VERSION);
    if let Some(gitinfo) = CRYFS_VERSION.gitinfo() {
        if gitinfo.commits_since_tag > 0 {
            eprintln!(
                "WARNING! This is a development version based on git commit {}. Please don't use in production.",
                gitinfo.commit_id,
            );
        }
        if gitinfo.modified {
            eprintln!("WARNING! There were uncommitted changes in the repository when building this version.");
        }
    }
    if CRYFS_VERSION.version().prerelease.is_some() {
        eprintln!("WARNING! This is a prerelease version. Please backup your data frequently!");
    }

    #[cfg(debug_assertions)]
    eprintln!("WARNING! This is a debug build. Performance might be slow.");

    #[cfg(feature = "check_for_updates")]
    _check_for_updates();
}

#[cfg(feature = "check_for_updates")]
fn _check_for_updates() {
    if env::no_update_check() {
        eprintln!("Automatic checking for security vulnerabilities and updates is disabled.");
    } else if env::is_noninteractive() {
        eprintln!("Automatic checking for security vulnerabilities and updates is disabled in noninteractive mode.");
    } else {
        // todo!()
    }
}

// TODO Tests
