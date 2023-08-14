use super::utils::{make_mock_filesystem, MockAsyncFilesystemLL, Runner};

#[tokio::test]
async fn simple() {
    let mock_filesystem = make_mock_filesystem();
    let runner = Runner::start(mock_filesystem);
    let driver = runner.driver();
    // TODO It seems mock expectations don't actually get enforced because they throw panics in another thread. How to fix this?
}
