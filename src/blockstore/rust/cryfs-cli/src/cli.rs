use anyhow::Result;
use clap::Parser;

use super::env;
use crate::args::Args;
use cryfs_cryfs::config::{Console, PasswordProvider};
use cryfs_version::VersionInfo;

const CRYFS_VERSION: VersionInfo = cryfs_cryfs::CRYFS_VERSION;

pub struct Cli {
    is_noninteractive: bool,
    // console: Box<dyn Console>,
    // password_provider: Box<dyn PasswordProvider>,
}

impl Cli {
    pub fn new() -> Self {
        let is_noninteractive = env::is_noninteractive();
        // let console = if is_noninteractive { todo!() } else { todo!() };
        // let password_provider = if is_noninteractive { todo!() } else { todo!() };
        Self {
            is_noninteractive,
            // console,
            // password_provider,
        }
    }

    pub fn main(&self) -> Result<()> {
        _show_version();

        let args = Args::parse();
        Ok(())
    }
}

// TODO (manually) test this
fn _show_version() {
    println!("CryFS Version {}", CRYFS_VERSION);
    if let Some(gitinfo) = CRYFS_VERSION.gitinfo() {
        if gitinfo.commits_since_tag > 0 {
            println!(
                "WARNING! This is a development version based on git commit {}. Please don't use in production.",
                gitinfo.commit_id,
            );
        }
        if gitinfo.modified {
            println!("WARNING! There were uncommitted changes in the repository when building this version.");
        }
    }
    if CRYFS_VERSION.version().prerelease.is_some() {
        println!("WARNING! This is a prerelease version. Please backup your data frequently!");
    }

    #[cfg(debug_assertions)]
    println!("WARNING! This is a debug build. Performance might be slow.");

    #[cfg(feature = "check_for_updates")]
    _check_for_updates();
}

#[cfg(feature = "check_for_updates")]
fn _check_for_updates() {
    if env::no_update_check() {
        println!("Automatic checking for security vulnerabilities and updates is disabled.");
    } else if env::is_noninteractive() {
        println!("Automatic checking for security vulnerabilities and updates is disabled in noninteractive mode.");
    } else {
        todo!()
    }
}

// TODO Tests
