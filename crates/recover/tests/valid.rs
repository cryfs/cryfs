mod common;
use common::fixture::FilesystemFixture;

#[tokio::test(flavor = "multi_thread")]
async fn bla() {
    let fs_fixture = FilesystemFixture::new().await;
}
