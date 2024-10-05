use anyhow::{bail, Context, Result};
use std::path::Path;

use cryfs_utils::tempfile::TempFile;

pub fn check_dir_accessible(
    path: &Path,
    name: &str,
    create_if_missing: bool,
    ask_create_fn: impl FnOnce(&Path) -> Result<bool>,
    // TODO error_if_missing:
) -> Result<()> {
    if !path.exists() {
        let mut create = create_if_missing;
        if create {
            log::info!("Creating {name} at {}", path.display());
        } else {
            create = ask_create_fn(path)?;
        }
        if create {
            std::fs::create_dir_all(path).with_context(|| format!("Error creating {name}"))?;
        } else {
            // TODO Return error with error code
            bail!("{name} '{}' does not exist", path.display())
        }
    }
    if !path.is_dir() {
        // TODO Return error with error code
        bail!("{name} '{}' is not a directory", path.display())
    }
    check_dir_writeable_and_readable(path, name)?;
    Ok(())
}

fn check_dir_writeable_and_readable(path: &Path, name: &str) -> Result<()> {
    // TODO return cli error codes
    const TEST_CONTENT: &str = "test content";
    const TESTFILE_NAME: &str = ".cryfs_testfile";

    // Write and read a file
    let testfile = TempFile::create(path.join(TESTFILE_NAME))?;
    std::fs::write(testfile.path(), TEST_CONTENT)
        .with_context(|| format!("Error writing to {name}"))?;
    let read_back_content = std::fs::read_to_string(testfile.path())
        .with_context(|| format!("Error reading from {name}"))?;
    if read_back_content != TEST_CONTENT {
        // TODO Return error with error code
        bail!("Error reading from {name}")
    }

    // Read the directory entries
    let mut dir_entries =
        std::fs::read_dir(path).with_context(|| format!("Error reading from {name}"))?;
    let dir_contains_test_file = dir_entries.any(|entry| match entry {
        Err(_) => false,
        Ok(entry) => entry
            .path()
            .file_name()
            .map(|file_name| file_name == TESTFILE_NAME)
            .unwrap_or(false),
    });
    if !dir_contains_test_file {
        // TODO Return error with error code
        bail!("Error reading from {name}")
    }

    Ok(())
}

pub fn check_mountdir_doesnt_contain_basedir(mount_args: &super::args::MountArgs) -> Result<()> {
    if mount_args.basedir.starts_with(&mount_args.mountdir) {
        // TODO Return error with error code
        bail!(
            "Vault directory '{}' cannot be inside of the mountpoint '{}'",
            mount_args.basedir.display(),
            mount_args.mountdir.display(),
        )
    }
    Ok(())
}

// TODO Tests
