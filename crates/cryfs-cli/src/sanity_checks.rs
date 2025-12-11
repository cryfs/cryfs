use anyhow::{Context, Result, bail};
use std::path::Path;

use cryfs_utils::tempfile::TempFile;

pub async fn check_dir_accessible(
    path: &Path,
    name: &str,
    create_if_missing: bool,
    ask_create_fn: impl FnOnce(&Path) -> Result<bool>,
    // TODO error_if_missing:
) -> Result<()> {
    if !tokio::fs::try_exists(path).await? {
        let mut create = create_if_missing;
        if create {
            log::info!("Creating {name} at {}", path.display());
        } else {
            create = ask_create_fn(path)?;
        }
        if create {
            tokio::fs::create_dir_all(path)
                .await
                .with_context(|| format!("Error creating {name}"))?;
        } else {
            // TODO Return error with error code
            bail!("{name} '{}' does not exist", path.display())
        }
    }
    if !tokio::fs::metadata(path).await?.is_dir() {
        // TODO Return error with error code
        bail!("{name} '{}' is not a directory", path.display())
    }
    check_dir_writeable_and_readable(path, name).await?;
    Ok(())
}

async fn check_dir_writeable_and_readable(path: &Path, name: &str) -> Result<()> {
    // TODO return cli error codes
    const TEST_CONTENT: &str = "test content";
    const TESTFILE_NAME: &str = ".cryfs_testfile";

    // Write and read a file
    let testfile = TempFile::create_async(path.join(TESTFILE_NAME)).await?;
    tokio::fs::write(testfile.path(), TEST_CONTENT)
        .await
        .with_context(|| format!("Error writing to {name}"))?;
    let read_back_content = tokio::fs::read_to_string(testfile.path())
        .await
        .with_context(|| format!("Error reading from {name}"))?;
    if read_back_content != TEST_CONTENT {
        // TODO Return error with error code
        bail!("Error reading from {name}")
    }

    // Read the directory entries
    let dir_entries = tokio::fs::read_dir(path)
        .await
        .with_context(|| format!("Error reading from {name}"))?;
    if !dir_contains_file(dir_entries, TESTFILE_NAME).await? {
        // TODO Return error with error code
        bail!("Error reading from {name}")
    }

    Ok(())
}

async fn dir_contains_file(
    mut dir_entries: tokio::fs::ReadDir,
    expected_filename: &str,
) -> Result<bool> {
    while let Some(entry) = dir_entries.next_entry().await? {
        if let Some(filename) = entry.path().file_name()
            && filename == expected_filename {
                return Ok(true);
            }
    }
    Ok(false)
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
