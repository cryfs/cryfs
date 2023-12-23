use cryfs_recover::CorruptedError;

mod common;
use common::fixture::FilesystemFixture;

#[tokio::test(flavor = "multi_thread")]
async fn fs_with_only_root_dir() {
    let fs_fixture = FilesystemFixture::new().await;

    let errors = fs_fixture.run_cryfs_recover().await;
    assert_eq!(Vec::<CorruptedError>::new(), errors);
}
