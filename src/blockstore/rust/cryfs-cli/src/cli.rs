use anyhow::Result;
use clap::Parser;
use std::fmt::Display;
use std::path::Path;

use super::console::InteractiveConsole;
use super::env;
use super::password_provider::{InteractivePasswordProvider, NoninteractivePasswordProvider};
use crate::args::CryfsArgs;
use cryfs_cryfs::{
    config::{
        CommandLineFlags, ConfigLoadError, ConfigLoadResult, Console, CryConfig, PasswordProvider,
    },
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

    pub fn main(&self) -> Result<()> {
        self.run_filesystem()?;

        println!(
            "Basedir: {:?}\nMountdir: {:?}",
            self.args.mount.as_ref().unwrap().basedir,
            self.args.mount.as_ref().unwrap().mountdir,
        );
        Ok(())
    }

    fn run_filesystem(&self) -> Result<()> {
        let config = self.load_or_create_config()?;
        print_config(&config);

        // unique_ptr<fspp::fuse::Fuse> fuse = nullptr;
        // bool stoppedBecauseOfIntegrityViolation = false;

        // auto onIntegrityViolation = [&fuse, &stoppedBecauseOfIntegrityViolation] () {
        //     if (fuse.get() != nullptr) {
        //     LOG(ERR, "Integrity violation detected after mounting. Unmounting.");
        //     stoppedBecauseOfIntegrityViolation = true;
        //     fuse->stop();
        //     } else {
        //     // Usually on an integrity violation, the file system is unmounted.
        //     // Here, the file system isn't initialized yet, i.e. we failed in the initial steps when
        //     // setting up _device before running initFilesystem.
        //     // We can't unmount a not-mounted file system, but we can make sure it doesn't get mounted.
        //     LOG(ERR, "Integrity violation detected before mounting. Not mounting.");
        //     }
        // };
        // const bool missingBlockIsIntegrityViolation = config.configFile->config()->missingBlockIsIntegrityViolation();
        // _device = optional<unique_ref<CryDevice>>(make_unique_ref<CryDevice>(std::move(config.configFile), options.baseDir(), std::move(localStateDir), config.myClientId, options.allowIntegrityViolations(), missingBlockIsIntegrityViolation, std::move(onIntegrityViolation)));
        // _sanityCheckFilesystem(_device->get());

        // auto initFilesystem = [&] (fspp::fuse::Fuse *fs){
        //     ASSERT(_device != none, "File system not ready to be initialized. Was it already initialized before?");

        //     //TODO Test auto unmounting after idle timeout
        //     const boost::optional<double> idle_minutes = options.unmountAfterIdleMinutes();
        //     _idleUnmounter = _createIdleCallback(idle_minutes, [fs, idle_minutes] {
        //         LOG(INFO, "Unmounting because file system was idle for {} minutes", *idle_minutes);
        //         fs->stop();
        //     });
        //     if (_idleUnmounter != none) {
        //         (*_device)->onFsAction(std::bind(&CallAfterTimeout::resetTimer, _idleUnmounter->get()));
        //     }

        //     return make_shared<fspp::FilesystemImpl>(std::move(*_device));
        // };

        // fuse = make_unique<fspp::fuse::Fuse>(initFilesystem, std::move(onMounted), "cryfs", "cryfs@" + options.baseDir().string());

        // _initLogfile(options);

        // std::cout << "\nMounting filesystem. To unmount, call:\n$ cryfs-unmount " << options.mountDir() << "\n"
        //             << std::endl;

        // if (options.foreground()) {
        //     fuse->runInForeground(options.mountDir(), options.fuseOptions());
        // } else {
        //     fuse->runInBackground(options.mountDir(), options.fuseOptions());
        // }

        // if (stoppedBecauseOfIntegrityViolation) {
        //     throw CryfsException("Integrity violation detected. Unmounting.", ErrorCode::IntegrityViolation);
        // }

        todo!()
    }

    // TODO Test the console flows for opening an existing/creating a new file system
    fn load_or_create_config(&self) -> Result<ConfigLoadResult, ConfigLoadError> {
        // TODO Allow changing config file using args as C++ did
        let config_file_location = self.basedir();
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
