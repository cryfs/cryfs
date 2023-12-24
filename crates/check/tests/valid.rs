use cryfs_check::CorruptedError;
use cryfs_cryfs::filesystem::fsblobstore::FsBlob;
use cryfs_cryfs::utils::fs_types::{Gid, Mode, Uid};
use std::time::SystemTime;

mod common;
use common::entry_helpers::{create_dir, create_some_files_directories_and_symlinks};
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
    let root_id = fs_fixture.root_blob_id();
    fs_fixture
        .update_fsblobstore(move |blobstore| async move {
            let mut root = FsBlob::into_dir(blobstore.load(&root_id).await.unwrap().unwrap())
                .await
                .unwrap();
            create_some_files_directories_and_symlinks(blobstore, &mut root).await;
            root.async_drop().await.unwrap();
        })
        .await;

    let errors = fs_fixture.run_cryfs_check().await;
    assert_eq!(Vec::<CorruptedError>::new(), errors);
}
