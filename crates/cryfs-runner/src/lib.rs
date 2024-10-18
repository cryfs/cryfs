#![forbid(unsafe_code)]
// TODO #![deny(missing_docs)]

use std::path::PathBuf;

use cryfs_blockstore::{
    AllowIntegrityViolations, ClientId, IntegrityConfig, MissingBlockIsIntegrityViolation,
    OnDiskBlockStore,
};
use cryfs_cli_utils::{setup_blockstore_stack, CliError};
use cryfs_filesystem::{config::CryConfig, localstate::LocalStateDir};

mod runner;
pub use runner::CreateOrLoad;
use runner::FilesystemRunner;

cryfs_version::assert_cargo_version_equals_git_version!();

#[derive(Debug)]
pub struct MountArgs {
    pub basedir: PathBuf,
    pub mountdir: PathBuf,
    pub allow_integrity_violations: AllowIntegrityViolations,
    pub create_or_load: CreateOrLoad,
}

pub fn mount_filesystem(
    config: CryConfig,
    my_client_id: ClientId,
    local_state_dir: LocalStateDir,
    mount_args: MountArgs,
) -> Result<(), CliError> {
    // TODO Runtime settings
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("cryfs")
        .enable_all()
        .build()
        .unwrap();
    let missing_block_is_integrity_violation = if config.missingBlockIsIntegrityViolation() {
        MissingBlockIsIntegrityViolation::IsAViolation
    } else {
        MissingBlockIsIntegrityViolation::IsNotAViolation
    };
    runtime.block_on(setup_blockstore_stack(
        OnDiskBlockStore::new(mount_args.basedir.to_owned()),
        &config,
        my_client_id,
        &local_state_dir,
        IntegrityConfig {
            allow_integrity_violations: mount_args.allow_integrity_violations,
            missing_block_is_integrity_violation,
            on_integrity_violation: Box::new(|err| {
                // TODO
            }),
        },
        FilesystemRunner {
            mountdir: &mount_args.mountdir,
            config: &config,
            create_or_load: mount_args.create_or_load,
        },
    ))??;

    Ok(())
}

// TODO Tests
