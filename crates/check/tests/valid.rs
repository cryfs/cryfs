//! Tests where the filesystem doesn't have errors

use cryfs_check::CorruptedError;

mod common;
use common::fixture::FilesystemFixture;

#[tokio::test(flavor = "multi_thread")]
async fn fs_with_only_root_dir() {
    let fs_fixture = FilesystemFixture::new().await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(Vec::<CorruptedError>::new(), errors);
}

#[tokio::test(flavor = "multi_thread")]
async fn fs_with_some_files_and_directories_and_symlinks() {
    let fs_fixture = FilesystemFixture::new().await;
    fs_fixture.create_some_blobs().await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(Vec::<CorruptedError>::new(), errors);
}
