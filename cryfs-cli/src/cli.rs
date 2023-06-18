use anyhow::Result;
use clap::Parser;
use std::fmt::Display;
use std::path::Path;

use super::console::InteractiveConsole;
use super::env;
use super::password_provider::{InteractivePasswordProvider, NoninteractivePasswordProvider};
use super::runner::FilesystemRunner;
use crate::args::CryfsArgs;
use cryfs_blockstore::{
    AllowIntegrityViolations, IntegrityConfig, MissingBlockIsIntegrityViolation,
};
use cryfs_cryfs::{
    config::ciphers::lookup_cipher_async,
    config::{CommandLineFlags, ConfigLoadError, ConfigLoadResult, Console, PasswordProvider},
    localstate::LocalStateDir,
};
use cryfs_version::VersionInfo;

const CRYFS_VERSION: VersionInfo = cryfs_cryfs::CRYFS_VERSION;

// TODO Check (and add tests for) error messages make sense, e.g. when
//   - wrong password
//   - basedir/mountdir don't exist
//   - ...

pub struct Cli {
    args: CryfsArgs,
    is_noninteractive: bool,
    local_state_dir: LocalStateDir,
}

impl Cli {
    // Returns None if the program should exit immediately with a success error code
    pub fn new() -> Result<Option<Self>> {
        let is_noninteractive = env::is_noninteractive();

        _show_version();

        let args = CryfsArgs::parse();

        if args.version {
            // No need to show version because we've already shown it, let's just exit
            return Ok(None);
        }

        if args.show_ciphers {
            for cipher in cryfs_cryfs::config::ALL_CIPHERS {
                println!("{}", cipher);
            }
            return Ok(None);
        }

        // TODO Make sure we have tests for the local_state_dir location
        let local_state_dir = cryfs_cryfs::localstate::LocalStateDir::new(env::local_state_dir()?);

        Ok(Some(Self {
            is_noninteractive,
            args,
            local_state_dir,
        }))
    }

    pub async fn main(&self) -> Result<()> {
        self.run_filesystem().await?;

        println!(
            "Basedir: {:?}\nMountdir: {:?}",
            self.args.mount.as_ref().unwrap().basedir,
            self.args.mount.as_ref().unwrap().mountdir,
        );
        Ok(())
    }

    async fn run_filesystem(&self) -> Result<()> {
        // TODO C++ code has lots more logic here, migrate that.
        let basedir = self.basedir().to_owned();
        if !basedir.exists() {
            std::fs::create_dir(&basedir)?;
        }
        let mountdir = self.mountdir();
        if !mountdir.exists() {
            std::fs::create_dir(mountdir)?;
        }

        let config = self.load_or_create_config()?;
        print_config(&config);

        lookup_cipher_async(
            &config.config.config().cipher,
            FilesystemRunner {
                basedir,
                mountdir,
                config: &config,
                local_state_dir: &self.local_state_dir,
                // TODO Setup IntegrityConfig correctly
                integrity_config: IntegrityConfig {
                    allow_integrity_violations: AllowIntegrityViolations::DontAllowViolations,
                    missing_block_is_integrity_violation:
                        MissingBlockIsIntegrityViolation::IsNotAViolation,
                    on_integrity_violation: Box::new(|err| {}),
                },
            },
        )
        .await??;

        Ok(())
    }

    // TODO Test the console flows for opening an existing/creating a new file system
    fn load_or_create_config(&self) -> Result<ConfigLoadResult, ConfigLoadError> {
        // TODO Allow changing config file using args as C++ did
        let config_file_location = self.basedir().join("cryfs.config");
        cryfs_cryfs::config::load_or_create(
            config_file_location.to_owned(),
            self.password_provider(),
            self.console(),
            // TODO Set CommandLineFlags correctly
            &CommandLineFlags {
                missing_block_is_integrity_violation: None,
                expected_cipher: None,
            },
            &self.local_state_dir,
        )
    }

    fn basedir(&self) -> &Path {
        &self.args.mount.as_ref().expect("Basedir not set").basedir
    }

    fn mountdir(&self) -> &Path {
        &self.args.mount.as_ref().expect("Basedir not set").mountdir
    }

    fn password_provider(&self) -> &'static dyn PasswordProvider {
        if self.is_noninteractive {
            // TODO Make sure we have tests for noninteractive mode
            &NoninteractivePasswordProvider
        } else {
            &InteractivePasswordProvider
        }
    }

    fn console(&self) -> &'static dyn Console {
        // TODO Implement NoninteractiveConsole
        &InteractiveConsole
    }
}

// TODO (manually?) test this
fn _show_version() {
    eprintln!("CryFS Version {}", CRYFS_VERSION);
    if let Some(gitinfo) = CRYFS_VERSION.gitinfo() {
        if let Some(tag_info) = gitinfo.tag_info {
            if tag_info.commits_since_tag > 0 {
                eprintln!(
                    "WARNING! This is a development version based on git commit {}. Please don't use in production.",
                    gitinfo.commit_id,
                );
            }
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
        // TODO
        // todo!()
    }
}

// TODO Integration test the outputs of print_config
fn print_config(config: &ConfigLoadResult) {
    fn print_value<T: Display + Eq>(old_value: T, new_value: T) {
        if old_value == new_value {
            print!("{}", old_value);
        } else {
            print!("{} -> {}", old_value, new_value);
        }
    }

    println!("----------------------------------------------------");
    println!("Filesystem configuration:");
    println!("----------------------------------------------------");
    print!("- Filesystem format version: ");
    print_value(
        &config.old_config.format_version,
        &config.config.config().format_version,
    );
    print!("\n- Created with: CryFS ");
    print_value(
        &config.old_config.created_with_version,
        &config.config.config().created_with_version,
    );
    print!("\n- Last opened with: CryFS ");
    print_value(
        &config.old_config.last_opened_with_version,
        &config.config.config().last_opened_with_version,
    );
    print!("\n- Cipher: ");
    print_value(&config.old_config.cipher, &config.config.config().cipher);
    print!("\n- Blocksize: ");
    print_value(
        config.old_config.blocksize_bytes,
        config.config.config().blocksize_bytes,
    );
    print!(" bytes");
    print!("\n- Filesystem Id: ");
    print_value(
        config.old_config.filesystem_id.to_hex(),
        config.config.config().filesystem_id.to_hex(),
    );

    println!("\n----------------------------------------------------");
}

// TODO Tests
